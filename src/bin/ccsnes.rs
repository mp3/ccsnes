// CCSNES CLI - Command line interface for the SNES emulator
use clap::{Parser, Subcommand};
use ccsnes::{Emulator, config::Config};
use std::path::PathBuf;
use std::time::Instant;
use log::{info, error};

#[derive(Parser)]
#[command(name = "ccsnes")]
#[command(author, version, about = "SNES Emulator", long_about = None)]
struct Cli {
    /// ROM file to load
    rom: Option<PathBuf>,
    
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
    
    /// Video scale factor (1-4)
    #[arg(short, long, default_value = "2")]
    scale: u32,
    
    /// Enable fullscreen
    #[arg(short, long)]
    fullscreen: bool,
    
    /// Disable audio
    #[arg(long)]
    no_audio: bool,
    
    /// Show FPS counter
    #[arg(long)]
    show_fps: bool,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the emulator with a ROM
    Run {
        /// ROM file to load
        rom: PathBuf,
    },
    /// Run test suite
    Test {
        /// Test ROM path
        #[arg(short, long)]
        rom: Option<PathBuf>,
    },
    /// Show ROM information
    Info {
        /// ROM file to analyze
        rom: PathBuf,
    },
    /// Benchmark emulation performance
    Bench {
        /// ROM file to benchmark
        rom: PathBuf,
        /// Number of frames to run
        #[arg(short, long, default_value = "1000")]
        frames: u64,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Load or create configuration
    let mut config = if let Some(config_path) = cli.config {
        Config::load_from_file(config_path)?
    } else {
        Config::load_or_default()
    };
    
    // Apply CLI overrides
    if cli.scale > 0 && cli.scale <= 4 {
        config.video.scale = cli.scale;
    }
    config.video.fullscreen = cli.fullscreen;
    config.audio.enabled = !cli.no_audio;
    config.debug.show_fps = cli.show_fps;
    
    // Create directories if needed
    config.create_directories()?;
    
    // Handle commands
    match cli.command {
        Some(Commands::Run { rom }) => {
            run_emulator(&rom, &config)?;
        }
        Some(Commands::Test { rom }) => {
            run_tests(rom.as_ref())?;
        }
        Some(Commands::Info { rom }) => {
            show_rom_info(&rom)?;
        }
        Some(Commands::Bench { rom, frames }) => {
            benchmark_emulator(&rom, frames)?;
        }
        None => {
            // No subcommand, check if ROM was provided as positional argument
            if let Some(rom) = cli.rom {
                run_emulator(&rom, &config)?;
            } else {
                eprintln!("No ROM file specified. Use --help for usage information.");
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}

fn run_emulator(rom_path: &PathBuf, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting CCSNES emulator...");
    info!("Loading ROM: {:?}", rom_path);
    
    // Load ROM file
    let rom_data = std::fs::read(rom_path)?;
    
    // Create emulator
    let mut emulator = Emulator::new()?;
    emulator.load_rom(&rom_data)?;
    
    // Get ROM info
    if let Some(rom_info) = emulator.get_rom_info() {
        info!("ROM Title: {}", rom_info.title);
        info!("Mapper: {:?}", rom_info.mapper_type);
        info!("Region: {:?}", rom_info.region);
        info!("ROM Size: {} KB", rom_info.rom_size / 1024);
    }
    
    // Check for SRAM file
    let sram_path = config.paths.sram_dir.join(
        rom_path.file_stem().unwrap().to_string_lossy().to_string() + ".srm"
    );
    
    if sram_path.exists() {
        info!("Loading SRAM from: {:?}", sram_path);
        let sram_data = std::fs::read(&sram_path)?;
        emulator.load_sram(&sram_data)?;
    }
    
    #[cfg(not(target_arch = "wasm32"))] {
        // Create frontend
        let mut frontend = ccsnes::frontend::native::NativeFrontend::new(config.video.scale, false)?;
        
        // Run emulation loop
        frontend.run(emulator)?;
        
        // TODO: Handle SRAM saving after emulation ends
        // This requires either modifying the frontend to return the emulator
        // or handling SRAM saving within the frontend itself
    }
    
    #[cfg(target_arch = "wasm32")] {
        error!("Native frontend not available in WebAssembly build");
    }
    
    info!("Emulator shut down cleanly");
    Ok(())
}

fn run_tests(test_rom: Option<&PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running emulator tests...");
    
    if let Some(rom_path) = test_rom {
        info!("Using test ROM: {:?}", rom_path);
        
        let rom_data = std::fs::read(rom_path)?;
        let mut emulator = Emulator::new()?;
        emulator.load_rom(&rom_data)?;
        
        // Run for a fixed number of frames
        let start = Instant::now();
        for frame in 0..60 {
            emulator.step_frame()?;
            
            // Check for test completion patterns
            // This would be customized based on the test ROM being used
            if frame % 10 == 0 {
                info!("Frame {}/60", frame);
            }
        }
        let elapsed = start.elapsed();
        
        info!("Test completed in {:?}", elapsed);
    } else {
        // Run built-in unit tests
        info!("Running unit tests...");
        
        // This would typically use cargo test, but we can run some basic tests here
        let mut emulator = Emulator::new()?;
        
        // Test ROM loading with invalid data
        assert!(emulator.load_rom(&[]).is_err());
        
        // Test save state functionality
        let state = emulator.save_state()?;
        emulator.load_state(&state)?;
        
        info!("All tests passed!");
    }
    
    Ok(())
}

fn show_rom_info(rom_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let rom_data = std::fs::read(rom_path)?;
    
    // Create temporary emulator just to load ROM
    let mut emulator = Emulator::new()?;
    emulator.load_rom(&rom_data)?;
    
    if let Some(info) = emulator.get_rom_info() {
        println!("ROM Information:");
        println!("================");
        println!("File: {:?}", rom_path);
        println!("Title: {}", info.title);
        println!("Mapper Type: {:?}", info.mapper_type);
        println!("ROM Size: {} KB ({} Mbit)", info.rom_size / 1024, info.rom_size * 8 / 1024 / 1024);
        println!("SRAM Size: {} KB", info.sram_size / 1024);
        println!("Region: {:?}", info.region);
        println!("Version: {}", info.version);
        println!("Coprocessor: {:?}", info.coprocessor);
    } else {
        error!("Failed to read ROM information");
    }
    
    Ok(())
}

fn benchmark_emulator(rom_path: &PathBuf, frames: u64) -> Result<(), Box<dyn std::error::Error>> {
    info!("Benchmarking emulator performance...");
    info!("ROM: {:?}", rom_path);
    info!("Frames to run: {}", frames);
    
    let rom_data = std::fs::read(rom_path)?;
    let mut emulator = Emulator::new()?;
    emulator.load_rom(&rom_data)?;
    
    // Warm up
    for _ in 0..60 {
        emulator.step_frame()?;
    }
    
    // Benchmark
    let start = Instant::now();
    let mut frame_times = Vec::with_capacity(frames as usize);
    
    for i in 0..frames {
        let frame_start = Instant::now();
        emulator.step_frame()?;
        let frame_time = frame_start.elapsed();
        frame_times.push(frame_time);
        
        if i % 100 == 0 && i > 0 {
            let elapsed = start.elapsed();
            let fps = i as f64 / elapsed.as_secs_f64();
            info!("Progress: {}/{} frames, {:.1} FPS", i, frames, fps);
        }
    }
    
    let total_time = start.elapsed();
    
    // Calculate statistics
    let avg_frame_time = total_time / frames as u32;
    let min_frame_time = *frame_times.iter().min().unwrap();
    let max_frame_time = *frame_times.iter().max().unwrap();
    
    // Sort for percentiles
    frame_times.sort();
    let p50 = frame_times[frames as usize / 2];
    let p95 = frame_times[(frames as f64 * 0.95) as usize];
    let p99 = frame_times[(frames as f64 * 0.99) as usize];
    
    let avg_fps = frames as f64 / total_time.as_secs_f64();
    let cpu_cycles = emulator.get_cycle_count();
    let cycles_per_frame = cpu_cycles / frames;
    
    println!("\nBenchmark Results:");
    println!("==================");
    println!("Total frames: {}", frames);
    println!("Total time: {:?}", total_time);
    println!("Average FPS: {:.2}", avg_fps);
    println!("\nFrame Times:");
    println!("  Average: {:?}", avg_frame_time);
    println!("  Min: {:?}", min_frame_time);
    println!("  Max: {:?}", max_frame_time);
    println!("  P50: {:?}", p50);
    println!("  P95: {:?}", p95);
    println!("  P99: {:?}", p99);
    println!("\nEmulation Stats:");
    println!("  Total CPU cycles: {}", cpu_cycles);
    println!("  Cycles per frame: {}", cycles_per_frame);
    println!("  Speed: {:.1}%", avg_fps / 60.0 * 100.0);
    
    Ok(())
}