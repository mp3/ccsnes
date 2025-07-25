// Performance profiler for optimization
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Profiler {
    // Function timing
    function_times: HashMap<String, FunctionProfile>,
    
    // Hot spot tracking
    hot_spots: HashMap<u32, HotSpot>,
    
    // Frame timing
    frame_times: Vec<Duration>,
    frame_start: Option<Instant>,
    
    // Component timing
    component_times: HashMap<Component, ComponentProfile>,
    
    // Enable/disable
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionProfile {
    pub name: String,
    pub call_count: u64,
    pub total_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
}

#[derive(Debug, Clone)]
pub struct HotSpot {
    pub address: u32,
    pub hit_count: u64,
    pub total_cycles: u64,
}

#[derive(Debug, Clone)]
pub struct ComponentProfile {
    pub total_time: Duration,
    pub call_count: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Component {
    Cpu,
    Ppu,
    Apu,
    Dma,
    Memory,
    Input,
    Other,
}

pub struct ProfileScope {
    profiler: *mut Profiler,
    name: String,
    start: Instant,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            function_times: HashMap::new(),
            hot_spots: HashMap::new(),
            frame_times: Vec::with_capacity(1000),
            frame_start: None,
            component_times: HashMap::new(),
            enabled: false,
        }
    }
    
    // Enable/disable profiling
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            log::info!("Profiling enabled");
        } else {
            log::info!("Profiling disabled");
        }
    }
    
    // Start profiling a function
    pub fn start_function(&mut self, name: &str) -> ProfileScope {
        ProfileScope {
            profiler: self as *mut Profiler,
            name: name.to_string(),
            start: Instant::now(),
        }
    }
    
    // End function profiling (called by ProfileScope drop)
    fn end_function(&mut self, name: &str, duration: Duration) {
        if !self.enabled {
            return;
        }
        
        let profile = self.function_times.entry(name.to_string())
            .or_insert_with(|| FunctionProfile {
                name: name.to_string(),
                call_count: 0,
                total_time: Duration::ZERO,
                min_time: Duration::MAX,
                max_time: Duration::ZERO,
            });
        
        profile.call_count += 1;
        profile.total_time += duration;
        profile.min_time = profile.min_time.min(duration);
        profile.max_time = profile.max_time.max(duration);
    }
    
    // Track hot spot
    pub fn track_hot_spot(&mut self, address: u32, cycles: u64) {
        if !self.enabled {
            return;
        }
        
        let hot_spot = self.hot_spots.entry(address)
            .or_insert_with(|| HotSpot {
                address,
                hit_count: 0,
                total_cycles: 0,
            });
        
        hot_spot.hit_count += 1;
        hot_spot.total_cycles += cycles;
    }
    
    // Start frame timing
    pub fn start_frame(&mut self) {
        if self.enabled {
            self.frame_start = Some(Instant::now());
        }
    }
    
    // End frame timing
    pub fn end_frame(&mut self) {
        if let Some(start) = self.frame_start.take() {
            let duration = start.elapsed();
            
            if self.frame_times.len() >= 1000 {
                self.frame_times.remove(0);
            }
            self.frame_times.push(duration);
        }
    }
    
    // Profile component
    pub fn profile_component<F, R>(&mut self, component: Component, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if !self.enabled {
            return f();
        }
        
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        
        let profile = self.component_times.entry(component)
            .or_insert_with(|| ComponentProfile {
                total_time: Duration::ZERO,
                call_count: 0,
                percentage: 0.0,
            });
        
        profile.total_time += duration;
        profile.call_count += 1;
        
        result
    }
    
    // Get frame statistics
    pub fn get_frame_stats(&self) -> FrameStats {
        if self.frame_times.is_empty() {
            return FrameStats::default();
        }
        
        let total: Duration = self.frame_times.iter().sum();
        let avg = total / self.frame_times.len() as u32;
        let min = *self.frame_times.iter().min().unwrap();
        let max = *self.frame_times.iter().max().unwrap();
        
        // Calculate percentiles
        let mut sorted = self.frame_times.clone();
        sorted.sort();
        let p50 = sorted[sorted.len() / 2];
        let p95 = sorted[sorted.len() * 95 / 100];
        let p99 = sorted[sorted.len() * 99 / 100];
        
        FrameStats {
            count: self.frame_times.len(),
            avg_time: avg,
            min_time: min,
            max_time: max,
            p50_time: p50,
            p95_time: p95,
            p99_time: p99,
            fps: 1.0 / avg.as_secs_f64(),
        }
    }
    
    // Get top hot spots
    pub fn get_hot_spots(&self, count: usize) -> Vec<&HotSpot> {
        let mut spots: Vec<_> = self.hot_spots.values().collect();
        spots.sort_by_key(|s| std::cmp::Reverse(s.total_cycles));
        spots.into_iter().take(count).collect()
    }
    
    // Get function profiles sorted by total time
    pub fn get_function_profiles(&self) -> Vec<&FunctionProfile> {
        let mut profiles: Vec<_> = self.function_times.values().collect();
        profiles.sort_by_key(|p| std::cmp::Reverse(p.total_time));
        profiles
    }
    
    // Update component percentages
    pub fn update_percentages(&mut self) {
        let total: Duration = self.component_times.values()
            .map(|p| p.total_time)
            .sum();
        
        if total.as_nanos() == 0 {
            return;
        }
        
        for profile in self.component_times.values_mut() {
            profile.percentage = profile.total_time.as_nanos() as f64 / total.as_nanos() as f64 * 100.0;
        }
    }
    
    // Reset profiling data
    pub fn reset(&mut self) {
        self.function_times.clear();
        self.hot_spots.clear();
        self.frame_times.clear();
        self.component_times.clear();
        log::info!("Profiling data reset");
    }
    
    // Generate profiling report
    pub fn generate_report(&mut self) -> String {
        self.update_percentages();
        
        let mut report = String::new();
        report.push_str("=== Performance Profile Report ===\n\n");
        
        // Frame statistics
        let frame_stats = self.get_frame_stats();
        report.push_str(&format!("Frame Statistics:\n"));
        report.push_str(&format!("  Average: {:.2}ms ({:.1} FPS)\n", 
            frame_stats.avg_time.as_secs_f64() * 1000.0, frame_stats.fps));
        report.push_str(&format!("  Min: {:.2}ms, Max: {:.2}ms\n",
            frame_stats.min_time.as_secs_f64() * 1000.0,
            frame_stats.max_time.as_secs_f64() * 1000.0));
        report.push_str(&format!("  P50: {:.2}ms, P95: {:.2}ms, P99: {:.2}ms\n\n",
            frame_stats.p50_time.as_secs_f64() * 1000.0,
            frame_stats.p95_time.as_secs_f64() * 1000.0,
            frame_stats.p99_time.as_secs_f64() * 1000.0));
        
        // Component breakdown
        report.push_str("Component Breakdown:\n");
        let mut components: Vec<_> = self.component_times.iter().collect();
        components.sort_by_key(|(_, p)| std::cmp::Reverse(p.total_time));
        
        for (component, profile) in components {
            report.push_str(&format!("  {:?}: {:.1}% ({:.2}ms total)\n",
                component,
                profile.percentage,
                profile.total_time.as_secs_f64() * 1000.0));
        }
        report.push_str("\n");
        
        // Top functions
        report.push_str("Top Functions by Time:\n");
        for profile in self.get_function_profiles().iter().take(10) {
            let avg_time = profile.total_time / profile.call_count as u32;
            report.push_str(&format!("  {}: {:.2}ms total, {} calls, {:.3}µs avg\n",
                profile.name,
                profile.total_time.as_secs_f64() * 1000.0,
                profile.call_count,
                avg_time.as_secs_f64() * 1_000_000.0));
        }
        report.push_str("\n");
        
        // Hot spots
        report.push_str("CPU Hot Spots:\n");
        for hot_spot in self.get_hot_spots(10) {
            report.push_str(&format!("  ${:06X}: {} hits, {} cycles\n",
                hot_spot.address,
                hot_spot.hit_count,
                hot_spot.total_cycles));
        }
        
        report
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        unsafe {
            (*self.profiler).end_function(&self.name, duration);
        }
    }
}

#[derive(Debug, Default)]
pub struct FrameStats {
    pub count: usize,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub p50_time: Duration,
    pub p95_time: Duration,
    pub p99_time: Duration,
    pub fps: f64,
}