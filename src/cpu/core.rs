use crate::memory::Bus;
use crate::Result;

pub struct Cpu {
    // TODO: Implement 65C816 CPU
    cycles: u64,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            cycles: 0,
        }
    }

    pub fn reset(&mut self, _bus: &mut Bus) -> Result<()> {
        self.cycles = 0;
        Ok(())
    }

    pub fn step(&mut self, _bus: &mut Bus) -> Result<u32> {
        // TODO: Implement CPU instruction execution
        // For now, just return 1 cycle
        self.cycles += 1;
        Ok(1)
    }

    pub fn trigger_nmi(&mut self, _bus: &mut Bus) -> Result<()> {
        // TODO: Implement NMI handling
        Ok(())
    }

    pub fn trigger_irq(&mut self, _bus: &mut Bus) -> Result<()> {
        // TODO: Implement IRQ handling
        Ok(())
    }
}