#[cfg(not(target_arch = "wasm32"))]
fn main() {
    env_logger::init();
    
    use clap::Parser;
    use ccsnes::frontend::native::NativeFrontend;
    use ccsnes::Emulator;
    use std::fs;
    use std::path::PathBuf;

    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    struct Args {
        /// ROM file to load
        #[arg(short, long)]
        rom: PathBuf,

        /// Enable debug mode
        #[arg(short, long)]
        debug: bool,

        /// Video scale factor
        #[arg(short, long, default_value = "2")]
        scale: u32,
    }

    fn run_emulator(args: Args) -> ccsnes::Result<()> {
        let rom_data = fs::read(&args.rom)?;
        
        let mut emulator = Emulator::new()?;
        emulator.load_rom(&rom_data)?;

        let mut frontend = NativeFrontend::new(args.scale, args.debug)?;
        frontend.run(emulator)?;

        Ok(())
    }

    let args = Args::parse();

    println!("CCSNES - Super Nintendo Entertainment System Emulator");
    println!("Loading ROM: {:?}", args.rom);

    match run_emulator(args) {
        Ok(_) => println!("Emulator exited successfully"),
        Err(e) => {
            eprintln!("Error running emulator: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // WebAssembly版では main() は使用されない
    // 代わりに wasm.rs の bindings を使用
}