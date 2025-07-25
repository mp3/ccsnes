#!/usr/bin/env python3
"""
Create a simple test ROM that displays a color pattern
This creates a minimal SNES ROM for testing basic functionality
"""

import struct
import sys

def create_test_rom():
    # Create a 32KB ROM
    rom = bytearray(32 * 1024)
    
    # Fill with 0xFF (empty ROM)  
    for i in range(len(rom)):
        rom[i] = 0xFF
    
    # SNES Header at $FFC0 (LoROM) - correct offset
    header_offset = 0x7FC0
    
    # Title: "CCSNES TEST ROM     " (21 bytes)
    title = b"CCSNES TEST ROM     "
    rom[header_offset:header_offset+21] = title[:21]
    
    # Map mode: $20 = LoROM
    rom[header_offset + 0x15] = 0x20
    
    # Cartridge type: $00 = ROM only
    rom[header_offset + 0x16] = 0x00
    
    # ROM size: $08 = 256KB (even though we're only using 32KB, this is the standard minimum)
    rom[header_offset + 0x17] = 0x08
    
    # RAM size: $00 = No RAM
    rom[header_offset + 0x18] = 0x00
    
    # Country: $00 = Japan
    rom[header_offset + 0x19] = 0x00
    
    # Developer: $00
    rom[header_offset + 0x1A] = 0x00
    
    # Version: $00
    rom[header_offset + 0x1B] = 0x00
    
    # Checksum complement (will calculate later)
    rom[header_offset + 0x1C:header_offset + 0x1E] = b'\x00\x00'
    
    # Checksum (will calculate later)
    rom[header_offset + 0x1E:header_offset + 0x20] = b'\x00\x00'
    
    # Reset vector at $FFFC
    reset_vector = 0x8000
    rom[0x7FFC:0x7FFE] = struct.pack('<H', reset_vector)
    
    # Simple reset code at $8000
    code_offset = 0x0000
    
    # Reset routine
    code = [
        0x78,           # SEI - Disable interrupts
        0x18,           # CLC
        0xFB,           # XCE - Switch to native mode
        0xC2, 0x30,     # REP #$30 - 16-bit A/X/Y
        0xA9, 0xFF, 0x1F,  # LDA #$1FFF
        0x1B,           # TCS - Set stack pointer
        0x64, 0x00,     # STZ $00 - Clear direct page
        0xE2, 0x20,     # SEP #$20 - 8-bit A
        
        # Turn off screen
        0xA9, 0x8F,     # LDA #$8F
        0x8D, 0x00, 0x21,  # STA $2100 - Screen off
        
        # Set background color
        0x9C, 0x21, 0x21,  # STZ $2121 - CGRAM address = 0
        0xA9, 0x1F,     # LDA #$1F - Red
        0x8D, 0x22, 0x21,  # STA $2122
        0xA9, 0x00,     # LDA #$00
        0x8D, 0x22, 0x21,  # STA $2122
        
        # Turn on screen
        0xA9, 0x0F,     # LDA #$0F
        0x8D, 0x00, 0x21,  # STA $2100 - Screen on
        
        # Enable NMI
        0xA9, 0x80,     # LDA #$80
        0x8D, 0x00, 0x42,  # STA $4200
        
        # Infinite loop
        0x80, 0xFE,     # BRA $ (infinite loop)
    ]
    
    # Write code to ROM
    for i, byte in enumerate(code):
        rom[code_offset + i] = byte
    
    # Calculate checksum
    checksum = sum(rom) & 0xFFFF
    rom[header_offset + 0x1E:header_offset + 0x20] = struct.pack('<H', checksum)
    rom[header_offset + 0x1C:header_offset + 0x1E] = struct.pack('<H', checksum ^ 0xFFFF)
    
    # Ensure exactly 32KB
    if len(rom) < 32768:
        rom.extend([0xFF] * (32768 - len(rom)))
    return bytes(rom[:32768])

def main():
    rom_data = create_test_rom()
    
    import os
    output_file = os.path.join(os.path.dirname(__file__), "test_roms", "simple_test.sfc")
    
    # Create directory if it doesn't exist
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    
    with open(output_file, 'wb') as f:
        f.write(rom_data)
    
    print(f"Created test ROM: {output_file}")
    print(f"Size: {len(rom_data)} bytes")
    print(f"Header at 0x7FC0: Title = {rom_data[0x7FC0:0x7FD5].decode('ascii', 'ignore').strip()}")

if __name__ == "__main__":
    main()