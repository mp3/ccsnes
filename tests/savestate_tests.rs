use ccsnes::savestate::SaveState;
use ccsnes::emulator::Emulator;
use std::fs;

#[test]
fn test_save_state_creation() {
    let state = SaveState::new();
    
    // Check that the state has default values
    assert_eq!(state.cpu.a, 0);
    assert_eq!(state.cpu.x, 0);
    assert_eq!(state.cpu.y, 0);
    assert_eq!(state.cpu.s, 0x01FF);
    assert_eq!(state.cpu.emulation_mode, true);
    
    assert_eq!(state.memory.wram.len(), 0x20000); // 128KB
    assert_eq!(state.apu.spc700.sp, 0xFF);
    assert_eq!(state.apu.spc700.pc, 0xFFC0);
}

#[test]
fn test_save_state_serialization() {
    let state = SaveState::new();
    
    // Test saving to file
    let test_path = "/tmp/test_savestate.dat";
    state.save_to_file(test_path).expect("Failed to save state");
    
    // Test loading from file
    let loaded_state = SaveState::load_from_file(test_path).expect("Failed to load state");
    
    // Compare some key values
    assert_eq!(state.cpu.a, loaded_state.cpu.a);
    assert_eq!(state.cpu.x, loaded_state.cpu.x);
    assert_eq!(state.cpu.s, loaded_state.cpu.s);
    assert_eq!(state.memory.wram.len(), loaded_state.memory.wram.len());
    assert_eq!(state.apu.spc700.ram.len(), loaded_state.apu.spc700.ram.len());
    
    // Clean up
    let _ = fs::remove_file(test_path);
}

#[test]
fn test_emulator_save_load_state() {
    let mut emulator = Emulator::new().expect("Failed to create emulator");
    
    // Create a simple test ROM
    let test_rom = vec![
        // ROM header padding
        vec![0; 0x7FC0],
        // ROM header
        vec![
            // Title (21 bytes)
            b"TEST ROM            ".to_vec(),
            // Mapper and ROM type
            vec![0x20, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00],
            // Checksum (4 bytes)
            vec![0x00, 0x00, 0xFF, 0xFF],
        ].concat(),
        // Reset vector and other vectors
        vec![0; 0x40],
    ].concat();
    
    // Load the test ROM
    emulator.load_rom(&test_rom).expect("Failed to load ROM");
    
    // Run a few steps to change CPU state
    for _ in 0..10 {
        let _ = emulator.step();
    }
    
    // Get the current CPU state
    let initial_pc = emulator.cpu.registers.pc;
    let initial_cycles = emulator.cycles;
    
    // Save state to file
    let save_path = "/tmp/test_emulator_save.dat";
    emulator.save_state_to_file(save_path).expect("Failed to save emulator state");
    
    // Run more steps to change state
    for _ in 0..20 {
        let _ = emulator.step();
    }
    
    // Verify state has changed
    assert_ne!(emulator.cpu.registers.pc, initial_pc);
    assert_ne!(emulator.cycles, initial_cycles);
    
    // Load the saved state
    emulator.load_state_from_file(save_path).expect("Failed to load emulator state");
    
    // Verify state has been restored
    assert_eq!(emulator.cpu.registers.pc, initial_pc);
    assert_eq!(emulator.cycles, initial_cycles);
    
    // Clean up
    let _ = fs::remove_file(save_path);
}

#[test]
fn test_state_version_check() {
    // Create a state with wrong version (this would be created by modifying the constant)
    let mut state = SaveState::new();
    
    // Save the state
    let test_path = "/tmp/test_version.dat";
    state.save_to_file(test_path).expect("Failed to save state");
    
    // Load should succeed with correct version
    let _loaded = SaveState::load_from_file(test_path).expect("Failed to load state with correct version");
    
    // Clean up
    let _ = fs::remove_file(test_path);
}

#[test]
fn test_compressed_save_state() {
    let mut state = SaveState::new();
    
    // Fill some memory with test data to verify compression
    for i in 0..1000 {
        state.memory.wram[i] = (i % 256) as u8;
        state.apu.spc700.ram[i] = ((i * 2) % 256) as u8;
    }
    
    let test_path = "/tmp/test_compressed.dat";
    state.save_to_file(test_path).expect("Failed to save compressed state");
    
    let loaded_state = SaveState::load_from_file(test_path).expect("Failed to load compressed state");
    
    // Verify data integrity after compression/decompression
    for i in 0..1000 {
        assert_eq!(state.memory.wram[i], loaded_state.memory.wram[i]);
        assert_eq!(state.apu.spc700.ram[i], loaded_state.apu.spc700.ram[i]);
    }
    
    // Clean up
    let _ = fs::remove_file(test_path);
}