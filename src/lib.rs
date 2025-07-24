pub mod apu;
pub mod cartridge;
pub mod cpu;
pub mod dma;
pub mod emulator;
pub mod input;
pub mod memory;
pub mod ppu;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub mod frontend;

pub use emulator::Emulator;

pub type Result<T> = std::result::Result<T, anyhow::Error>;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn init_wasm() {
    #[cfg(feature = "wee_alloc")]
    {
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
        #[global_allocator]
        static mut GLOBAL: wee_alloc::WeeAlloc = ALLOC;
    }

    web_sys::console::log_1(&"CCSNES WebAssembly module initialized".into());
}