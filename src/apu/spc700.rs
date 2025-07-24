// SPC700 CPU (8-bit processor for audio)

pub struct Spc700 {
    // CPU registers
    pub(super) a: u8,      // Accumulator
    pub(super) x: u8,      // X index
    pub(super) y: u8,      // Y index
    pub(super) sp: u8,     // Stack pointer
    pub(super) pc: u16,    // Program counter
    pub(super) psw: u8,    // Processor status word
    
    // Audio RAM (64KB)
    pub(super) ram: Vec<u8>,
    
    // IPL ROM enable
    ipl_rom_enable: bool,
    
    // Communication ports with main CPU
    port_in: [u8; 4],
    port_out: [u8; 4],
    
    // Timers
    timer_enable: u8,
    timer_target: [u8; 3],
    timer_counter: [u8; 3],
    timer_output: [u8; 3],
    
    pub(super) cycles: u64,
}

impl Spc700 {
    pub fn new() -> Self {
        let mut spc = Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFF,
            pc: 0xFFC0, // Boot vector
            psw: 0x02,
            ram: vec![0; 0x10000], // 64KB
            ipl_rom_enable: true,
            port_in: [0; 4],
            port_out: [0; 4],
            timer_enable: 0,
            timer_target: [0; 3],
            timer_counter: [0; 3],
            timer_output: [0; 3],
            cycles: 0,
        };
        
        // Load IPL ROM
        spc.load_ipl();
        spc
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFF;
        self.pc = 0xFFC0;
        self.psw = 0x02;
        self.ipl_rom_enable = true;
        self.port_in = [0; 4];
        self.port_out = [0; 4];
        self.timer_enable = 0;
        self.timer_target = [0; 3];
        self.timer_counter = [0; 3];
        self.timer_output = [0; 3];
        
        // Load IPL (Initial Program Loader)
        self.load_ipl();
        
        self.cycles = 0;
    }

    pub fn step(&mut self) {
        // Execute one instruction
        self.execute_instruction();
        
        // Update timers
        self.update_timers();
    }

    fn load_ipl(&mut self) {
        // IPL ROM (64 bytes at $FFC0-$FFFF)
        // This is a minimal IPL that waits for the main CPU to upload code
        const IPL_ROM: &[u8] = &[
            // $FFC0: Wait for CPU communication
            0xCD, 0xEF,  // MOV X, #$EF
            0xBD,        // MOV SP, X
            0xE8, 0x00,  // MOV A, #$00
            0xC6,        // MOV (X), A
            0x1D,        // DEC X
            0xD0, 0xFC,  // BNE -4
            0x8F, 0xAA, 0xF4,  // MOV $F4, #$AA
            0x8F, 0xBB, 0xF5,  // MOV $F5, #$BB
            // Wait for response
            0x78, 0xCC, 0xF4,  // CMP $F4, #$CC
            0xD0, 0xFB,        // BNE -5
            0xE8, 0x00,        // MOV A, #$00
            0xC4, 0xF4,        // MOV $F4, A
            0xD4, 0xF5,        // MOV $F5, X
            // Main transfer loop
            0x3E, 0xF4,        // CMP X, $F4
            0xD0, 0xFC,        // BNE -4
            0xE4, 0xF5,        // MOV A, $F5
            0xCB, 0xF4,        // MOV $F4, Y
            0xEE,              // POP Y
            0xEC, 0xF6,        // MOV Y, $F6
            0xEC, 0xF7,        // MOV Y, $F7
            0xD7, 0x00,        // MOV ($00)+Y, A
            0xFC,              // INC Y
            0xD0, 0xF1,        // BNE -15
            0xAB, 0x01,        // INC $01
            0x3E, 0xF4,        // CMP X, $F4
            0xD0, 0xE9,        // BNE -23
            // Padding
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // Reset vector at $FFFE
            0xC0, 0xFF,  // Points to $FFC0
        ];
        
        // Copy IPL ROM to high memory
        for (i, &byte) in IPL_ROM.iter().enumerate() {
            if 0xFFC0 + i < 0x10000 {
                self.ram[0xFFC0 + i] = byte;
            }
        }
    }

    pub fn read8(&self, address: u16) -> u8 {
        match address {
            // I/O Ports (check these first)
            0x00F0 => 0,  // Test register (unused)
            0x00F1 => {
                // Control register
                let mut value = 0x00;
                if self.ipl_rom_enable { value |= 0x80; }
                if self.timer_enable & 0x01 != 0 { value |= 0x01; }
                if self.timer_enable & 0x02 != 0 { value |= 0x02; }
                if self.timer_enable & 0x04 != 0 { value |= 0x04; }
                value
            }
            0x00F2 => 0,  // DSP address (handled by DSP)
            0x00F3 => 0,  // DSP data (handled by DSP)
            0x00F4 => self.port_out[0],  // Port 0
            0x00F5 => self.port_out[1],  // Port 1
            0x00F6 => self.port_out[2],  // Port 2
            0x00F7 => self.port_out[3],  // Port 3
            0x00F8 => self.ram[address as usize],  // RAM mirror
            0x00F9 => self.ram[address as usize],  // RAM mirror
            0x00FA => self.timer_target[0],
            0x00FB => self.timer_target[1],
            0x00FC => self.timer_target[2],
            0x00FD => self.timer_output[0],
            0x00FE => self.timer_output[1],
            0x00FF => self.timer_output[2],
            
            // IPL ROM area
            0xFFC0..=0xFFFF => {
                if self.ipl_rom_enable {
                    self.ram[address as usize]  // IPL ROM
                } else {
                    self.ram[address as usize]  // RAM
                }
            }
            
            // RAM (everything else)
            _ => self.ram[address as usize],
        }
    }

    pub fn write8(&mut self, address: u16, value: u8) {
        match address {
            // I/O Ports (check these first)
            0x00F0 => {} // Test register (unused)
            0x00F1 => {
                // Control register
                self.ipl_rom_enable = (value & 0x80) != 0;
                self.timer_enable = value & 0x07;
                
                // Clear timers on write
                if value & 0x01 != 0 { self.timer_output[0] = 0; self.timer_counter[0] = 0; }
                if value & 0x02 != 0 { self.timer_output[1] = 0; self.timer_counter[1] = 0; }
                if value & 0x04 != 0 { self.timer_output[2] = 0; self.timer_counter[2] = 0; }
            }
            0x00F2 => {} // DSP address (handled by DSP)
            0x00F3 => {} // DSP data (handled by DSP)
            0x00F4 => self.port_in[0] = value,  // Port 0
            0x00F5 => self.port_in[1] = value,  // Port 1
            0x00F6 => self.port_in[2] = value,  // Port 2
            0x00F7 => self.port_in[3] = value,  // Port 3
            0x00F8 => self.ram[address as usize] = value,  // RAM
            0x00F9 => self.ram[address as usize] = value,  // RAM
            0x00FA => self.timer_target[0] = value,
            0x00FB => self.timer_target[1] = value,
            0x00FC => self.timer_target[2] = value,
            0x00FD..=0x00FF => {} // Timer outputs are read-only
            
            // High memory (IPL ROM area when enabled)
            0xFFC0..=0xFFFF => {
                if !self.ipl_rom_enable {
                    self.ram[address as usize] = value;
                }
            }
            
            // RAM (everything else)
            _ => self.ram[address as usize] = value,
        }
    }
    
    fn update_timers(&mut self) {
        // Timer 0 and 1: 8 kHz (every 128 cycles)
        // Timer 2: 64 kHz (every 16 cycles)
        
        if self.timer_enable & 0x01 != 0 && self.cycles % 128 == 0 {
            self.timer_counter[0] = self.timer_counter[0].wrapping_add(1);
            if self.timer_counter[0] == self.timer_target[0] {
                self.timer_counter[0] = 0;
                self.timer_output[0] = self.timer_output[0].wrapping_add(1) & 0x0F;
            }
        }
        
        if self.timer_enable & 0x02 != 0 && self.cycles % 128 == 0 {
            self.timer_counter[1] = self.timer_counter[1].wrapping_add(1);
            if self.timer_counter[1] == self.timer_target[1] {
                self.timer_counter[1] = 0;
                self.timer_output[1] = self.timer_output[1].wrapping_add(1) & 0x0F;
            }
        }
        
        if self.timer_enable & 0x04 != 0 && self.cycles % 16 == 0 {
            self.timer_counter[2] = self.timer_counter[2].wrapping_add(1);
            if self.timer_counter[2] == self.timer_target[2] {
                self.timer_counter[2] = 0;
                self.timer_output[2] = self.timer_output[2].wrapping_add(1) & 0x0F;
            }
        }
    }
    
    // Communication with main CPU
    pub fn read_port(&self, port: usize) -> u8 {
        if port < 4 {
            self.port_in[port]
        } else {
            0
        }
    }
    
    pub fn write_port(&mut self, port: usize, value: u8) {
        if port < 4 {
            self.port_out[port] = value;
        }
    }
}