use ccsnes::cpu::Cpu;
use ccsnes::memory::Bus;

#[test]
fn test_cpu_reset() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    
    // Set reset vector
    bus.write16(0xFFFC, 0x8000);
    
    cpu.reset(&mut bus).unwrap();
    
    assert_eq!(cpu.get_registers().pc, 0x8000);
    assert_eq!(cpu.get_registers().s, 0x01FF);
    assert_eq!(cpu.get_registers().p, 0x34);
    assert!(cpu.get_registers().emulation_mode);
}

#[test]
fn test_lda_immediate() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    
    // LDA #$42
    bus.write8(0x8000, 0xA9); // LDA immediate
    bus.write8(0x8001, 0x42);
    
    cpu.reset(&mut bus).unwrap();
    cpu.get_registers_mut().pc = 0x8000;
    
    let cycles = cpu.step(&mut bus).unwrap();
    
    assert_eq!(cpu.get_registers().get_a(), 0x42);
    assert_eq!(cpu.get_registers().pc, 0x8002);
    assert_eq!(cycles, 2);
    assert!(!cpu.get_registers().zero());
    assert!(!cpu.get_registers().negative());
}

#[test]
fn test_sta_absolute() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    
    // Set A register
    cpu.get_registers_mut().set_a(0x55);
    
    // STA $1234
    bus.write8(0x8000, 0x8D); // STA absolute
    bus.write8(0x8001, 0x34);
    bus.write8(0x8002, 0x12);
    
    cpu.get_registers_mut().pc = 0x8000;
    
    let cycles = cpu.step(&mut bus).unwrap();
    
    assert_eq!(bus.read8(0x1234), 0x55);
    assert_eq!(cpu.get_registers().pc, 0x8003);
    assert_eq!(cycles, 4);
}

#[test]
fn test_adc_with_carry() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    
    // Test ADC without carry
    cpu.get_registers_mut().set_a(0x50);
    cpu.get_registers_mut().set_carry(false);
    
    // ADC #$30
    bus.write8(0x8000, 0x69);
    bus.write8(0x8001, 0x30);
    
    cpu.get_registers_mut().pc = 0x8000;
    cpu.step(&mut bus).unwrap();
    
    assert_eq!(cpu.get_registers().get_a(), 0x80);
    assert!(!cpu.get_registers().carry());
    assert!(cpu.get_registers().negative());
    assert!(!cpu.get_registers().zero());
    
    // Test ADC with carry
    cpu.get_registers_mut().set_a(0xFF);
    cpu.get_registers_mut().set_carry(true);
    
    // ADC #$00
    bus.write8(0x8002, 0x69);
    bus.write8(0x8003, 0x00);
    
    cpu.get_registers_mut().pc = 0x8002;
    cpu.step(&mut bus).unwrap();
    
    assert_eq!(cpu.get_registers().get_a(), 0x00);
    assert!(cpu.get_registers().carry());
    assert!(cpu.get_registers().zero());
    assert!(!cpu.get_registers().negative());
}

#[test]
fn test_branch_instructions() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    
    // Test BNE (Branch if Not Equal)
    cpu.get_registers_mut().set_zero(false);
    
    // BNE +10
    bus.write8(0x8000, 0xD0);
    bus.write8(0x8001, 0x0A);
    
    cpu.get_registers_mut().pc = 0x8000;
    let cycles = cpu.step(&mut bus).unwrap();
    
    assert_eq!(cpu.get_registers().pc, 0x800C); // 0x8002 + 0x0A
    assert!(cycles >= 3); // Branch taken adds cycles
    
    // Test BEQ (Branch if Equal) - not taken
    cpu.get_registers_mut().set_zero(false);
    
    // BEQ +10
    bus.write8(0x800C, 0xF0);
    bus.write8(0x800D, 0x0A);
    
    let cycles = cpu.step(&mut bus).unwrap();
    
    assert_eq!(cpu.get_registers().pc, 0x800E); // Not taken
    assert_eq!(cycles, 2); // No extra cycles for not taken
}

#[test]
fn test_stack_operations() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    
    cpu.reset(&mut bus).unwrap();
    
    // Push A
    cpu.get_registers_mut().set_a(0xAB);
    
    // PHA
    bus.write8(0x8000, 0x48);
    cpu.get_registers_mut().pc = 0x8000;
    cpu.step(&mut bus).unwrap();
    
    assert_eq!(cpu.get_registers().s, 0x01FE);
    assert_eq!(bus.read8(0x01FF), 0xAB);
    
    // Pull A
    cpu.get_registers_mut().set_a(0x00);
    
    // PLA
    bus.write8(0x8001, 0x68);
    cpu.get_registers_mut().pc = 0x8001;
    cpu.step(&mut bus).unwrap();
    
    assert_eq!(cpu.get_registers().s, 0x01FF);
    assert_eq!(cpu.get_registers().get_a(), 0xAB);
}

#[test]
fn test_jsr_rts() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    
    cpu.reset(&mut bus).unwrap();
    
    // JSR $9000
    bus.write8(0x8000, 0x20); // JSR
    bus.write8(0x8001, 0x00);
    bus.write8(0x8002, 0x90);
    
    // RTS at $9000
    bus.write8(0x9000, 0x60); // RTS
    
    cpu.get_registers_mut().pc = 0x8000;
    
    // Execute JSR
    cpu.step(&mut bus).unwrap();
    
    assert_eq!(cpu.get_registers().pc, 0x9000);
    assert_eq!(cpu.get_registers().s, 0x01FD); // Pushed 2 bytes
    
    // Check return address on stack (should be 0x8002)
    // Stack grows downward: high byte at higher address, low byte at lower address
    let high_byte = bus.read8(0x01FF);
    let low_byte = bus.read8(0x01FE);
    assert_eq!(high_byte, 0x80); // High byte at 0x01FF
    assert_eq!(low_byte, 0x02); // Low byte at 0x01FE
    
    // Execute RTS
    cpu.step(&mut bus).unwrap();
    
    assert_eq!(cpu.get_registers().pc, 0x8003); // Return address + 1
    assert_eq!(cpu.get_registers().s, 0x01FF); // Stack restored
}

#[test]
fn test_native_mode_switch() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    
    cpu.reset(&mut bus).unwrap();
    assert!(cpu.get_registers().emulation_mode);
    
    // Clear carry
    cpu.get_registers_mut().set_carry(false);
    
    // XCE - Exchange Carry and Emulation
    bus.write8(0x8000, 0xFB);
    cpu.get_registers_mut().pc = 0x8000;
    cpu.step(&mut bus).unwrap();
    
    assert!(!cpu.get_registers().emulation_mode);
    assert!(cpu.get_registers().carry()); // Old emulation mode value
}

#[test]
fn test_16bit_mode() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    
    // Switch to native mode first
    cpu.get_registers_mut().emulation_mode = false;
    
    // REP #$30 - Clear M and X flags (16-bit A/Memory and X/Y)
    bus.write8(0x8000, 0xC2);
    bus.write8(0x8001, 0x30);
    
    cpu.get_registers_mut().pc = 0x8000;
    let p_before = cpu.get_registers().p;
    cpu.step(&mut bus).unwrap();
    let p_after = cpu.get_registers().p;
    
    eprintln!("P before REP: ${:02X}, after: ${:02X}", p_before, p_after);
    eprintln!("Memory width: {}, Index width: {}", cpu.get_registers().memory_width(), cpu.get_registers().index_width());
    
    assert!(!cpu.get_registers().memory_width());
    assert!(!cpu.get_registers().index_width());
    
    // LDA #$1234 (16-bit immediate)
    bus.write8(0x8002, 0xA9);
    bus.write8(0x8003, 0x34);
    bus.write8(0x8004, 0x12);
    
    cpu.get_registers_mut().pc = 0x8002;
    cpu.step(&mut bus).unwrap();
    
    assert_eq!(cpu.get_registers().a, 0x1234);
}

#[test]
fn test_block_move() {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();
    
    // Switch to native mode with 16-bit index and accumulator
    cpu.get_registers_mut().emulation_mode = false;
    cpu.get_registers_mut().set_index_width(false); // 16-bit X/Y
    cpu.get_registers_mut().set_memory_width(false); // 16-bit A
    
    // Setup source data
    bus.write8(0x1000, 0x11);
    bus.write8(0x1001, 0x22);
    bus.write8(0x1002, 0x33);
    
    // Setup registers
    cpu.get_registers_mut().set_a(2); // Transfer 3 bytes (count = 2)
    cpu.get_registers_mut().set_x(0x1000); // Source address in WRAM
    cpu.get_registers_mut().set_y(0x1800); // Destination address in WRAM (0x0000-0x1FFF is WRAM mirror)
    
    // MVP $00,$00 - Move block positive
    bus.write8(0x8000, 0x44);
    bus.write8(0x8001, 0x00); // Source bank
    bus.write8(0x8002, 0x00); // Destination bank
    
    cpu.get_registers_mut().pc = 0x8000;
    
    // Execute first byte transfer
    cpu.step(&mut bus).unwrap();
    assert_eq!(bus.read8(0x1800), 0x11);
    assert_eq!(cpu.get_registers().get_x(), 0x1001);
    assert_eq!(cpu.get_registers().get_y(), 0x1801);
    assert_eq!(cpu.get_registers().get_a(), 1);
    
    // Execute second byte transfer
    cpu.step(&mut bus).unwrap();
    assert_eq!(bus.read8(0x1801), 0x22);
    assert_eq!(cpu.get_registers().get_a(), 0);
    
    // Execute third byte transfer
    cpu.step(&mut bus).unwrap();
    assert_eq!(bus.read8(0x1802), 0x33);
    assert_eq!(cpu.get_registers().get_a(), 0xFFFF);
    
    // Should continue to next instruction now
    assert_eq!(cpu.get_registers().pc, 0x8003);
}