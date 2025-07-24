use crate::memory::Bus;

pub struct DmaController {
    channels: [DmaChannel; 8],
    hdma_enabled: u8,
}

#[derive(Clone, Copy)]
pub struct DmaChannel {
    // $43x0: DMA/HDMA parameters
    pub control: u8,
    // $43x1: Destination register
    pub destination: u8,
    // $43x2-$43x3: Source address (low 16 bits)
    pub source_address: u16,
    // $43x4: Source bank
    pub source_bank: u8,
    // $43x5-$43x6: Transfer size / HDMA indirect address
    pub transfer_size: u16,
    // $43x7: HDMA indirect bank
    pub indirect_bank: u8,
    // $43x8-$43x9: HDMA table address
    pub table_address: u16,
    // $43xA: HDMA line counter
    pub line_counter: u8,
}

impl Default for DmaChannel {
    fn default() -> Self {
        Self {
            control: 0,
            destination: 0,
            source_address: 0,
            source_bank: 0,
            transfer_size: 0,
            indirect_bank: 0,
            table_address: 0,
            line_counter: 0,
        }
    }
}

impl DmaController {
    pub fn new() -> Self {
        Self {
            channels: [DmaChannel::default(); 8],
            hdma_enabled: 0,
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        let channel = ((address - 0x4300) / 0x10) as usize;
        let register = (address - 0x4300) % 0x10;

        if channel >= 8 {
            return;
        }

        match register {
            0x0 => self.channels[channel].control = value,
            0x1 => self.channels[channel].destination = value,
            0x2 => self.channels[channel].source_address = 
                (self.channels[channel].source_address & 0xFF00) | value as u16,
            0x3 => self.channels[channel].source_address = 
                (self.channels[channel].source_address & 0x00FF) | ((value as u16) << 8),
            0x4 => self.channels[channel].source_bank = value,
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

    pub fn read_register(&self, address: u16) -> u8 {
        let channel = ((address - 0x4300) / 0x10) as usize;
        let register = (address - 0x4300) % 0x10;

        if channel >= 8 {
            return 0;
        }

        match register {
            0x0 => self.channels[channel].control,
            0x1 => self.channels[channel].destination,
            0x2 => (self.channels[channel].source_address & 0xFF) as u8,
            0x3 => ((self.channels[channel].source_address >> 8) & 0xFF) as u8,
            0x4 => self.channels[channel].source_bank,
            0x5 => (self.channels[channel].transfer_size & 0xFF) as u8,
            0x6 => ((self.channels[channel].transfer_size >> 8) & 0xFF) as u8,
            0x7 => self.channels[channel].indirect_bank,
            0x8 => (self.channels[channel].table_address & 0xFF) as u8,
            0x9 => ((self.channels[channel].table_address >> 8) & 0xFF) as u8,
            0xA => self.channels[channel].line_counter,
            _ => 0,
        }
    }

    pub fn start_dma(&mut self, channels: u8, bus: &mut Bus) -> u32 {
        let mut total_cycles = 0;

        for channel in 0..8 {
            if (channels & (1 << channel)) != 0 {
                total_cycles += self.execute_dma_transfer(channel, bus);
            }
        }

        total_cycles
    }

    pub fn set_hdma_enabled(&mut self, enabled: u8) {
        self.hdma_enabled = enabled;
    }

    pub fn hdma_enabled(&self) -> u8 {
        self.hdma_enabled
    }

    fn execute_dma_transfer(&mut self, channel: usize, bus: &mut Bus) -> u32 {
        let ch = &mut self.channels[channel];
        let transfer_mode = ch.control & 0x07;
        let direction = (ch.control & 0x80) != 0; // 0 = CPU->PPU, 1 = PPU->CPU
        let step = if (ch.control & 0x10) != 0 { -1i32 } else { 1i32 };

        let mut cycles = 0;
        let mut size = if ch.transfer_size == 0 { 0x10000 } else { ch.transfer_size as u32 };
        
        let mut source_addr = ((ch.source_bank as u32) << 16) | (ch.source_address as u32);
        let destination_reg = 0x2100 + (ch.destination as u16);

        while size > 0 {
            match transfer_mode {
                0 => {
                    // 1 byte transfer
                    if direction {
                        // PPU -> CPU
                        let value = bus.read8(destination_reg as u32);
                        bus.write8(source_addr, value);
                    } else {
                        // CPU -> PPU
                        let value = bus.read8(source_addr);
                        bus.write8(destination_reg as u32, value);
                    }
                    source_addr = (source_addr as i32 + step) as u32;
                    size -= 1;
                    cycles += 8;
                }
                1 => {
                    // 2 bytes transfer (A, B)
                    for i in 0..2 {
                        if size == 0 { break; }
                        let dest = destination_reg + (i % 2);
                        if direction {
                            let value = bus.read8(dest as u32);
                            bus.write8(source_addr, value);
                        } else {
                            let value = bus.read8(source_addr);
                            bus.write8(dest as u32, value);
                        }
                        source_addr = (source_addr as i32 + step) as u32;
                        size -= 1;
                        cycles += 8;
                    }
                }
                2 => {
                    // 2 bytes transfer (A, A)
                    if direction {
                        let value = bus.read8(destination_reg as u32);
                        bus.write8(source_addr, value);
                    } else {
                        let value = bus.read8(source_addr);
                        bus.write8(destination_reg as u32, value);
                    }
                    source_addr = (source_addr as i32 + step) as u32;
                    size -= 1;
                    cycles += 8;
                }
                3 => {
                    // 4 bytes transfer (A, B, A, B)
                    for i in 0..4 {
                        if size == 0 { break; }
                        let dest = destination_reg + (i % 2);
                        if direction {
                            let value = bus.read8(dest as u32);
                            bus.write8(source_addr, value);
                        } else {
                            let value = bus.read8(source_addr);
                            bus.write8(dest as u32, value);
                        }
                        source_addr = (source_addr as i32 + step) as u32;
                        size -= 1;
                        cycles += 8;
                    }
                }
                4 => {
                    // 4 bytes transfer (A, B, C, D)
                    for i in 0..4 {
                        if size == 0 { break; }
                        let dest = destination_reg + i;
                        if direction {
                            let value = bus.read8(dest as u32);
                            bus.write8(source_addr, value);
                        } else {
                            let value = bus.read8(source_addr);
                            bus.write8(dest as u32, value);
                        }
                        source_addr = (source_addr as i32 + step) as u32;
                        size -= 1;
                        cycles += 8;
                    }
                }
                _ => {
                    // Invalid transfer mode
                    break;
                }
            }
        }

        // Update source address
        ch.source_address = (source_addr & 0xFFFF) as u16;
        ch.source_bank = ((source_addr >> 16) & 0xFF) as u8;
        ch.transfer_size = size as u16;

        cycles
    }

    pub fn do_hdma(&mut self, _bus: &mut Bus) {
        // TODO: Implement HDMA (Horizontal DMA) for scanline effects
        // This is more complex and involves per-scanline transfers
    }
}