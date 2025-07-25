use crate::memory::Bus;
use crate::ppu::Ppu;
use log::trace;

// DMA transfer modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DmaMode {
    SingleByte,              // 0: A -> B
    TwoRegisters,            // 1: A, A+1 -> B, B+1
    SingleToTwoSame,         // 2: A -> B, B
    TwoToTwoSame,            // 3: A, A+1 -> B, B
    FourRegisters,           // 4: A, A+1, A+2, A+3 -> B, B+1, B+2, B+3
    TwoAlternating,          // 5: A, A+1 -> B, A, A+1 -> B+1 (HDMA only)
    SingleToTwoAlternating,  // 6: A -> B, A -> B+1 (HDMA only)
    TwoToTwoAlternating,     // 7: A, A+1 -> B, A, A+1 -> B+1 (HDMA only)
}

impl From<u8> for DmaMode {
    fn from(value: u8) -> Self {
        match value & 0x07 {
            0 => DmaMode::SingleByte,
            1 => DmaMode::TwoRegisters,
            2 => DmaMode::SingleToTwoSame,
            3 => DmaMode::TwoToTwoSame,
            4 => DmaMode::FourRegisters,
            5 => DmaMode::TwoAlternating,
            6 => DmaMode::SingleToTwoAlternating,
            7 => DmaMode::TwoToTwoAlternating,
            _ => DmaMode::SingleByte,
        }
    }
}

// DMA channel state
#[derive(Debug, Clone)]
pub struct DmaChannel {
    // Control registers
    pub control: u8,         // $43n0 - DMA Control
    pub b_address: u8,       // $43n1 - B Bus Address
    pub a_address: u16,      // $43n2-3 - A Bus Address
    pub a_bank: u8,          // $43n4 - A Bus Bank
    pub transfer_size: u16,  // $43n5-6 - Transfer Size / Indirect Address
    pub indirect_bank: u8,   // $43n7 - Indirect Bank
    pub table_address: u16,  // $43n8-9 - Table Address (HDMA)
    pub line_counter: u8,    // $43nA - Line Counter (HDMA)
    
    // Internal state
    pub hdma_active: bool,
    pub hdma_completed: bool,
    pub hdma_indirect_mode: bool,
    pub hdma_repeat_mode: bool,
}

impl DmaChannel {
    pub fn new() -> Self {
        Self {
            control: 0,
            b_address: 0,
            a_address: 0,
            a_bank: 0,
            transfer_size: 0,
            indirect_bank: 0,
            table_address: 0,
            line_counter: 0,
            hdma_active: false,
            hdma_completed: false,
            hdma_indirect_mode: false,
            hdma_repeat_mode: false,
        }
    }
    
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    pub fn get_mode(&self) -> DmaMode {
        DmaMode::from(self.control)
    }
    
    pub fn is_direction_b_to_a(&self) -> bool {
        (self.control & 0x80) != 0
    }
    
    pub fn is_indirect_hdma(&self) -> bool {
        (self.control & 0x40) != 0
    }
    
    pub fn is_address_increment(&self) -> bool {
        match (self.control >> 3) & 0x03 {
            0 => true,  // Increment
            1 => false, // Fixed
            2 => false, // Decrement
            _ => true,
        }
    }
    
    pub fn is_address_decrement(&self) -> bool {
        ((self.control >> 3) & 0x03) == 2
    }
    
    pub fn get_address_step(&self) -> i16 {
        match (self.control >> 3) & 0x03 {
            0 => 1,   // Increment
            1 => 0,   // Fixed
            2 => -1,  // Decrement
            _ => 1,
        }
    }
}

pub struct DmaController {
    channels: [DmaChannel; 8],
    dma_enable: u8,  // $420B
    hdma_enable: u8, // $420C
}

impl DmaController {
    pub fn new() -> Self {
        Self {
            channels: [
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
                DmaChannel::new(),
            ],
            dma_enable: 0,
            hdma_enable: 0,
        }
    }
    
    pub fn reset(&mut self) {
        for channel in &mut self.channels {
            channel.reset();
        }
        self.dma_enable = 0;
        self.hdma_enable = 0;
    }
    
    // Execute DMA transfers for enabled channels
    pub fn execute_dma(&mut self, bus: &mut Bus, ppu: &mut Ppu) -> u32 {
        let mut total_cycles = 8; // DMA setup overhead
        
        // Process channels in order 0-7
        for channel_num in 0..8 {
            if (self.dma_enable & (1 << channel_num)) != 0 {
                trace!("DMA: Starting transfer on channel {}", channel_num);
                total_cycles += self.execute_channel_dma(channel_num, bus, ppu);
            }
        }
        
        // Clear DMA enable after transfers complete
        self.dma_enable = 0;
        
        total_cycles
    }
    
    fn execute_channel_dma(&mut self, channel: usize, bus: &mut Bus, ppu: &mut Ppu) -> u32 {
        let mode = self.channels[channel].get_mode();
        let mut cycles = 0;
        let mut remaining = self.channels[channel].transfer_size as u32;
        
        // Handle zero length as 65536 bytes
        if remaining == 0 {
            remaining = 0x10000;
        }
        
        let b_to_a = self.channels[channel].is_direction_b_to_a();
        let step = self.channels[channel].get_address_step();
        let b_address = self.channels[channel].b_address;
        let a_bank = self.channels[channel].a_bank;
        
        trace!("DMA {}: Mode {:?}, {} bytes, {} -> {}", 
               channel, mode, remaining,
               if b_to_a { "B" } else { "A" },
               if b_to_a { "A" } else { "B" });
        
        while remaining > 0 {
            let a_address = self.channels[channel].a_address;
            
            match mode {
                DmaMode::SingleByte => {
                    // Transfer single byte
                    if b_to_a {
                        let value = self.read_b_bus(bus, ppu, b_address);
                        self.write_a_bus(bus, a_bank, a_address, value);
                    } else {
                        let value = bus.read8((a_bank as u32) << 16 | a_address as u32);
                        self.write_b_bus(bus, ppu, b_address, value);
                    }
                    
                    self.channels[channel].a_address = (a_address as i32 + step as i32) as u16;
                    remaining -= 1;
                    cycles += 8;
                }
                
                DmaMode::TwoRegisters => {
                    // Transfer two bytes to consecutive B addresses
                    for i in 0..2 {
                        if remaining == 0 { break; }
                        
                        let a_address = self.channels[channel].a_address;
                        
                        if b_to_a {
                            let value = self.read_b_bus(bus, ppu, b_address + i);
                            self.write_a_bus(bus, a_bank, a_address, value);
                        } else {
                            let value = bus.read8((a_bank as u32) << 16 | a_address as u32);
                            self.write_b_bus(bus, ppu, b_address + i, value);
                        }
                        
                        self.channels[channel].a_address = (a_address as i32 + step as i32) as u16;
                        remaining -= 1;
                        cycles += 8;
                    }
                }
                
                DmaMode::SingleToTwoSame => {
                    // Transfer single byte to same B address twice
                    if b_to_a {
                        let value = self.read_b_bus(bus, ppu, b_address);
                        self.write_a_bus(bus, a_bank, a_address, value);
                        self.channels[channel].a_address = (a_address as i32 + step as i32) as u16;
                        if remaining > 1 {
                            let a_address = self.channels[channel].a_address;
                            self.write_a_bus(bus, a_bank, a_address, value);
                            self.channels[channel].a_address = (a_address as i32 + step as i32) as u16;
                        }
                    } else {
                        let value = bus.read8((a_bank as u32) << 16 | a_address as u32);
                        self.write_b_bus(bus, ppu, b_address, value);
                        self.channels[channel].a_address = (a_address as i32 + step as i32) as u16;
                        if remaining > 1 {
                            self.write_b_bus(bus, ppu, b_address, value);
                        }
                    }
                    
                    remaining = remaining.saturating_sub(2);
                    cycles += 16;
                }
                
                DmaMode::TwoToTwoSame => {
                    // Transfer two bytes to same B addresses
                    for _repeat in 0..2 {
                        for i in 0..2 {
                            if remaining == 0 { break; }
                            
                            let a_address = self.channels[channel].a_address;
                            
                            if b_to_a {
                                let value = self.read_b_bus(bus, ppu, b_address + (i & 1));
                                self.write_a_bus(bus, a_bank, a_address, value);
                            } else {
                                let value = bus.read8((a_bank as u32) << 16 | a_address as u32);
                                self.write_b_bus(bus, ppu, b_address + (i & 1), value);
                            }
                            
                            self.channels[channel].a_address = (a_address as i32 + step as i32) as u16;
                            remaining -= 1;
                            cycles += 8;
                        }
                    }
                }
                
                DmaMode::FourRegisters => {
                    // Transfer four bytes to consecutive B addresses
                    for i in 0..4 {
                        if remaining == 0 { break; }
                        
                        let a_address = self.channels[channel].a_address;
                        
                        if b_to_a {
                            let value = self.read_b_bus(bus, ppu, b_address + i);
                            self.write_a_bus(bus, a_bank, a_address, value);
                        } else {
                            let value = bus.read8((a_bank as u32) << 16 | a_address as u32);
                            self.write_b_bus(bus, ppu, b_address + i, value);
                        }
                        
                        self.channels[channel].a_address = (a_address as i32 + step as i32) as u16;
                        remaining -= 1;
                        cycles += 8;
                    }
                }
                
                _ => {
                    // Modes 5-7 are HDMA only, treat as mode 0 for DMA
                    if b_to_a {
                        let value = self.read_b_bus(bus, ppu, b_address);
                        self.write_a_bus(bus, a_bank, a_address, value);
                    } else {
                        let value = bus.read8((a_bank as u32) << 16 | a_address as u32);
                        self.write_b_bus(bus, ppu, b_address, value);
                    }
                    
                    self.channels[channel].a_address = (a_address as i32 + step as i32) as u16;
                    remaining -= 1;
                    cycles += 8;
                }
            }
        }
        
        // Update channel state
        self.channels[channel].transfer_size = 0;
        
        cycles
    }
    
    // Initialize HDMA channels at start of frame
    pub fn init_hdma(&mut self, bus: &mut Bus) {
        for channel_num in 0..8 {
            if (self.hdma_enable & (1 << channel_num)) != 0 {
                let ch = &mut self.channels[channel_num];
                ch.hdma_active = true;
                ch.hdma_completed = false;
                ch.table_address = ch.a_address;
                ch.hdma_indirect_mode = ch.is_indirect_hdma();
                
                // Read first table entry
                self.reload_hdma_channel(channel_num, bus);
            }
        }
    }
    
    // Execute HDMA transfers for current scanline
    pub fn execute_hdma(&mut self, bus: &mut Bus, ppu: &mut Ppu) -> u32 {
        let mut cycles = 18; // HDMA overhead per scanline
        
        for channel_num in 0..8 {
            if (self.hdma_enable & (1 << channel_num)) != 0 {
                let ch = &mut self.channels[channel_num];
                
                if !ch.hdma_completed && ch.hdma_active {
                    cycles += self.execute_channel_hdma(channel_num, bus, ppu);
                }
            }
        }
        
        cycles
    }
    
    fn execute_channel_hdma(&mut self, channel: usize, bus: &mut Bus, ppu: &mut Ppu) -> u32 {
        let mut cycles = 8;
        
        // Check if we need to reload
        if self.channels[channel].line_counter == 0 {
            self.reload_hdma_channel(channel, bus);
            if self.channels[channel].hdma_completed {
                return cycles;
            }
        }
        
        // Extract necessary values before mutable operations
        let hdma_repeat_mode = self.channels[channel].hdma_repeat_mode;
        let line_counter = self.channels[channel].line_counter;
        let mode = self.channels[channel].get_mode();
        let b_to_a = self.channels[channel].is_direction_b_to_a();
        let b_address = self.channels[channel].b_address;
        let a_bank = self.channels[channel].a_bank;
        let a_address = self.channels[channel].a_address;
        
        // Execute transfer for this scanline
        if hdma_repeat_mode || line_counter == 1 {
            match mode {
                DmaMode::SingleByte => {
                    if b_to_a {
                        let value = self.read_b_bus(bus, ppu, b_address);
                        self.write_a_bus(bus, a_bank, a_address, value);
                    } else {
                        let value = bus.read8((a_bank as u32) << 16 | a_address as u32);
                        self.write_b_bus(bus, ppu, b_address, value);
                    }
                    self.channels[channel].a_address = a_address.wrapping_add(1);
                    cycles += 8;
                }
                
                DmaMode::TwoRegisters => {
                    for i in 0..2 {
                        let current_a_address = self.channels[channel].a_address;
                        if b_to_a {
                            let value = self.read_b_bus(bus, ppu, b_address + i);
                            self.write_a_bus(bus, a_bank, current_a_address, value);
                        } else {
                            let value = bus.read8((a_bank as u32) << 16 | current_a_address as u32);
                            self.write_b_bus(bus, ppu, b_address + i, value);
                        }
                        self.channels[channel].a_address = current_a_address.wrapping_add(1);
                        cycles += 8;
                    }
                }
                
                _ => {
                    // TODO: Implement other HDMA modes
                    cycles += 8;
                }
            }
        }
        
        // Decrement line counter
        self.channels[channel].line_counter -= 1;
        
        cycles
    }
    
    fn reload_hdma_channel(&mut self, channel: usize, bus: &mut Bus) {
        // Read line counter and options
        let a_bank = self.channels[channel].a_bank;
        let table_address = self.channels[channel].table_address;
        let table_addr = (a_bank as u32) << 16 | table_address as u32;
        let header = bus.read8(table_addr);
        
        self.channels[channel].table_address = table_address.wrapping_add(1);
        
        if header == 0 {
            // End of HDMA table
            self.channels[channel].hdma_completed = true;
            self.channels[channel].hdma_active = false;
            return;
        }
        
        self.channels[channel].line_counter = header & 0x7F;
        self.channels[channel].hdma_repeat_mode = (header & 0x80) != 0;
        
        let hdma_indirect_mode = self.channels[channel].hdma_indirect_mode;
        
        if hdma_indirect_mode {
            // Read indirect address
            let table_address = self.channels[channel].table_address;
            let low = bus.read8((a_bank as u32) << 16 | table_address as u32);
            self.channels[channel].table_address = table_address.wrapping_add(1);
            
            let table_address = self.channels[channel].table_address;
            let high = bus.read8((a_bank as u32) << 16 | table_address as u32);
            self.channels[channel].table_address = table_address.wrapping_add(1);
            
            self.channels[channel].a_address = ((high as u16) << 8) | (low as u16);
            self.channels[channel].a_bank = self.channels[channel].indirect_bank;
        } else {
            // Direct mode - read data inline
            self.channels[channel].a_address = self.channels[channel].table_address;
            
            // Advance table pointer based on transfer mode
            let mode = self.channels[channel].get_mode();
            let advance = match mode {
                DmaMode::SingleByte => 1,
                DmaMode::TwoRegisters => 2,
                DmaMode::SingleToTwoSame => 1,
                DmaMode::TwoToTwoSame => 2,
                DmaMode::FourRegisters => 4,
                _ => 1,
            };
            
            let hdma_repeat_mode = self.channels[channel].hdma_repeat_mode;
            let line_counter = self.channels[channel].line_counter;
            let table_address = self.channels[channel].table_address;
            
            if hdma_repeat_mode {
                self.channels[channel].table_address = table_address.wrapping_add(advance);
            } else {
                self.channels[channel].table_address = table_address.wrapping_add(advance * line_counter as u16);
            }
        }
    }
    
    // Helper functions for B-Bus access (PPU registers)
    fn read_b_bus(&self, bus: &mut Bus, ppu: &mut Ppu, address: u8) -> u8 {
        let full_address = 0x2100 + address as u16;
        if full_address >= 0x2100 && full_address <= 0x213F {
            ppu.read_register(full_address)
        } else {
            bus.read8(full_address as u32)
        }
    }
    
    fn write_b_bus(&self, bus: &mut Bus, ppu: &mut Ppu, address: u8, value: u8) {
        let full_address = 0x2100 + address as u16;
        if full_address >= 0x2100 && full_address <= 0x213F {
            ppu.write_register(full_address, value);
        } else {
            bus.write8(full_address as u32, value);
        }
    }
    
    // Helper function for A-Bus access (main memory)
    fn write_a_bus(&self, bus: &mut Bus, bank: u8, address: u16, value: u8) {
        let full_address = (bank as u32) << 16 | address as u32;
        bus.write8(full_address, value);
    }
    
    // Register access
    pub fn read_register(&self, address: u16) -> u8 {
        match address {
            0x420B => self.dma_enable,
            0x420C => self.hdma_enable,
            
            // DMA channel registers
            0x4300..=0x437F => {
                let channel = ((address - 0x4300) >> 4) as usize;
                let reg = (address & 0x0F) as usize;
                
                if channel < 8 {
                    match reg {
                        0x0 => self.channels[channel].control,
                        0x1 => self.channels[channel].b_address,
                        0x2 => (self.channels[channel].a_address & 0xFF) as u8,
                        0x3 => (self.channels[channel].a_address >> 8) as u8,
                        0x4 => self.channels[channel].a_bank,
                        0x5 => (self.channels[channel].transfer_size & 0xFF) as u8,
                        0x6 => (self.channels[channel].transfer_size >> 8) as u8,
                        0x7 => self.channels[channel].indirect_bank,
                        0x8 => (self.channels[channel].table_address & 0xFF) as u8,
                        0x9 => (self.channels[channel].table_address >> 8) as u8,
                        0xA => self.channels[channel].line_counter,
                        _ => 0xFF,
                    }
                } else {
                    0xFF
                }
            }
            
            _ => 0xFF,
        }
    }
    
    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x420B => {
                self.dma_enable = value;
                trace!("DMA: Enable register set to ${:02X}", value);
            }
            
            0x420C => {
                self.hdma_enable = value;
                trace!("HDMA: Enable register set to ${:02X}", value);
            }
            
            // DMA channel registers
            0x4300..=0x437F => {
                let channel = ((address - 0x4300) >> 4) as usize;
                let reg = (address & 0x0F) as usize;
                
                if channel < 8 {
                    match reg {
                        0x0 => self.channels[channel].control = value,
                        0x1 => self.channels[channel].b_address = value,
                        0x2 => self.channels[channel].a_address = 
                               (self.channels[channel].a_address & 0xFF00) | value as u16,
                        0x3 => self.channels[channel].a_address = 
                               (self.channels[channel].a_address & 0x00FF) | ((value as u16) << 8),
                        0x4 => self.channels[channel].a_bank = value,
                        0x5 => self.channels[channel].transfer_size = 
                               (self.channels[channel].transfer_size & 0xFF00) | value as u16,
                        0x6 => self.channels[channel].transfer_size = 
                               (self.channels[channel].transfer_size & 0x00FF) | ((value as u16) << 8),
                        0x7 => self.channels[channel].indirect_bank = value,
                        0x8 => self.channels[channel].table_address = 
                               (self.channels[channel].table_address & 0xFF00) | value as u16,
                        0x9 => self.channels[channel].table_address = 
                               (self.channels[channel].table_address & 0x00FF) | ((value as u16) << 8),
                        0xA => self.channels[channel].line_counter = value,
                        _ => {}
                    }
                }
            }
            
            _ => {}
        }
    }
    
    // Save state functionality
    pub fn save_state(&self) -> crate::savestate::DmaState {
        use crate::savestate::{DmaState, DmaChannelState};
        
        let channel_states: Vec<DmaChannelState> = self.channels.iter().map(|ch| {
            DmaChannelState {
                enabled: false, // Will be set from enable registers
                hdma_enabled: false, // Will be set from enable registers
                direction: if ch.is_direction_b_to_a() { 1 } else { 0 },
                indirect: ch.is_indirect_hdma(),
                reverse_transfer: false, // TODO: Implement proper detection
                fixed_transfer: !ch.is_address_increment(),
                transfer_mode: ch.control & 0x07,
                b_address: ch.b_address,
                a_address: ch.a_address,
                a_bank: ch.a_bank,
                transfer_size: ch.transfer_size,
                indirect_bank: ch.indirect_bank,
                hdma_line_counter: ch.line_counter,
                hdma_address: ch.table_address,
                hdma_completed: ch.hdma_completed,
            }
        }).collect();
        
        // Set enable flags from enable registers
        let mut state = DmaState {
            channels: channel_states,
        };
        
        for (i, ch_state) in state.channels.iter_mut().enumerate() {
            ch_state.enabled = (self.dma_enable & (1 << i)) != 0;
            ch_state.hdma_enabled = (self.hdma_enable & (1 << i)) != 0;
        }
        
        state
    }
    
    pub fn load_state(&mut self, state: &crate::savestate::DmaState) {
        // Reset enable registers
        self.dma_enable = 0;
        self.hdma_enable = 0;
        
        for (i, ch_state) in state.channels.iter().enumerate() {
            if i < 8 {
                let channel = &mut self.channels[i];
                
                // Reconstruct control register
                let mut control = ch_state.transfer_mode & 0x07;
                if ch_state.indirect {
                    control |= 0x40;
                }
                if ch_state.direction != 0 {
                    control |= 0x80;
                }
                if ch_state.fixed_transfer {
                    control |= 0x08; // Fixed addressing
                }
                
                channel.control = control;
                channel.b_address = ch_state.b_address;
                channel.a_address = ch_state.a_address;
                channel.a_bank = ch_state.a_bank;
                channel.transfer_size = ch_state.transfer_size;
                channel.indirect_bank = ch_state.indirect_bank;
                channel.line_counter = ch_state.hdma_line_counter;
                channel.table_address = ch_state.hdma_address;
                channel.hdma_completed = ch_state.hdma_completed;
                channel.hdma_active = ch_state.hdma_enabled;
                
                // Set enable flags
                if ch_state.enabled {
                    self.dma_enable |= 1 << i;
                }
                if ch_state.hdma_enabled {
                    self.hdma_enable |= 1 << i;
                }
            }
        }
    }
}