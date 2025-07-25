// Memory access cache for performance optimization
use std::cell::Cell;

const CACHE_LINE_SIZE: usize = 64;
const CACHE_LINES: usize = 256;
const CACHE_SIZE: usize = CACHE_LINE_SIZE * CACHE_LINES;

// Simple direct-mapped cache for memory reads
pub struct MemoryCache {
    // Cache data storage
    data: Box<[u8; CACHE_SIZE]>,
    // Tags for each cache line (upper address bits)
    tags: Box<[u32; CACHE_LINES]>,
    // Valid flags for each cache line
    valid: Box<[bool; CACHE_LINES]>,
    // Statistics
    hits: Cell<u64>,
    misses: Cell<u64>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            data: Box::new([0; CACHE_SIZE]),
            tags: Box::new([0; CACHE_LINES]),
            valid: Box::new([false; CACHE_LINES]),
            hits: Cell::new(0),
            misses: Cell::new(0),
        }
    }
    
    pub fn reset(&mut self) {
        self.valid.fill(false);
        self.hits.set(0);
        self.misses.set(0);
    }
    
    // Get cache line index from address
    #[inline(always)]
    fn get_line_index(address: u32) -> usize {
        ((address / CACHE_LINE_SIZE as u32) % CACHE_LINES as u32) as usize
    }
    
    // Get offset within cache line
    #[inline(always)]
    fn get_line_offset(address: u32) -> usize {
        (address % CACHE_LINE_SIZE as u32) as usize
    }
    
    // Get tag from address
    #[inline(always)]
    fn get_tag(address: u32) -> u32 {
        address / CACHE_SIZE as u32
    }
    
    // Check if address is in cache
    #[inline(always)]
    pub fn lookup(&self, address: u32) -> Option<u8> {
        let line_idx = Self::get_line_index(address);
        let tag = Self::get_tag(address);
        
        if self.valid[line_idx] && self.tags[line_idx] == tag {
            let offset = Self::get_line_offset(address);
            let base = line_idx * CACHE_LINE_SIZE;
            self.hits.set(self.hits.get() + 1);
            Some(self.data[base + offset])
        } else {
            self.misses.set(self.misses.get() + 1);
            None
        }
    }
    
    // Load a cache line from memory
    pub fn load_line(&mut self, address: u32, data: &[u8]) {
        let line_idx = Self::get_line_index(address);
        let tag = Self::get_tag(address);
        let base = line_idx * CACHE_LINE_SIZE;
        let line_base = (address / CACHE_LINE_SIZE as u32) * CACHE_LINE_SIZE as u32;
        
        // Copy data to cache line
        let copy_len = data.len().min(CACHE_LINE_SIZE);
        self.data[base..base + copy_len].copy_from_slice(&data[..copy_len]);
        
        // Update metadata
        self.tags[line_idx] = tag;
        self.valid[line_idx] = true;
    }
    
    // Invalidate cache line
    pub fn invalidate_line(&mut self, address: u32) {
        let line_idx = Self::get_line_index(address);
        self.valid[line_idx] = false;
    }
    
    // Invalidate entire cache
    pub fn invalidate_all(&mut self) {
        self.valid.fill(false);
    }
    
    // Get cache statistics
    pub fn get_stats(&self) -> (u64, u64) {
        (self.hits.get(), self.misses.get())
    }
    
    // Calculate hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits.get() + self.misses.get();
        if total == 0 {
            0.0
        } else {
            self.hits.get() as f64 / total as f64
        }
    }
}

// Fast memory region lookup using binary search
pub struct MemoryRegions {
    regions: Vec<(u32, u32, MemoryRegionType)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionType {
    Wram,
    Cartridge,
    Ppu,
    Apu,
    Dma,
    Unmapped,
}

impl MemoryRegions {
    pub fn new() -> Self {
        let mut regions = Vec::new();
        
        // Define memory regions for fast lookup
        // WRAM regions
        regions.push((0x7E0000, 0x7FFFFF, MemoryRegionType::Wram));
        regions.push((0x000000, 0x001FFF, MemoryRegionType::Wram)); // Mirror
        
        // PPU registers
        regions.push((0x002100, 0x00213F, MemoryRegionType::Ppu));
        
        // APU registers
        regions.push((0x002140, 0x00217F, MemoryRegionType::Apu));
        
        // DMA registers
        regions.push((0x004200, 0x0042FF, MemoryRegionType::Dma));
        regions.push((0x004300, 0x00437F, MemoryRegionType::Dma));
        
        // Sort regions by start address for binary search
        regions.sort_by_key(|&(start, _, _)| start);
        
        Self { regions }
    }
    
    #[inline(always)]
    pub fn lookup(&self, address: u32) -> MemoryRegionType {
        // Binary search for the region
        match self.regions.binary_search_by(|&(start, end, _)| {
            if address < start {
                std::cmp::Ordering::Greater
            } else if address > end {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        }) {
            Ok(idx) => self.regions[idx].2,
            Err(_) => MemoryRegionType::Cartridge, // Default to cartridge
        }
    }
}