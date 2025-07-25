// CCSNES WebAssembly Frontend
import init, { WasmEmulator } from '../pkg/ccsnes.js';

let emulator = null;
let animationId = null;
let isPaused = true;
let audioContext = null;
let audioBufferSize = 2048;
let lastFrameTime = performance.now();
let frameCount = 0;
let currentButtons = 0;

// Button mappings
const BUTTON_A     = 0x80;
const BUTTON_B     = 0x8000;
const BUTTON_X     = 0x40;
const BUTTON_Y     = 0x4000;
const BUTTON_L     = 0x20;
const BUTTON_R     = 0x10;
const BUTTON_START = 0x1000;
const BUTTON_SELECT = 0x2000;
const BUTTON_UP    = 0x800;
const BUTTON_DOWN  = 0x400;
const BUTTON_LEFT  = 0x200;
const BUTTON_RIGHT = 0x100;

// Key mappings
const keyMap = {
    'KeyZ': BUTTON_A,
    'KeyX': BUTTON_B,
    'KeyA': BUTTON_X,
    'KeyS': BUTTON_Y,
    'KeyQ': BUTTON_L,
    'KeyW': BUTTON_R,
    'Enter': BUTTON_START,
    'ShiftRight': BUTTON_SELECT,
    'ArrowUp': BUTTON_UP,
    'ArrowDown': BUTTON_DOWN,
    'ArrowLeft': BUTTON_LEFT,
    'ArrowRight': BUTTON_RIGHT,
};

// Initialize the emulator
async function initEmulator() {
    const loadingDiv = document.createElement('div');
    loadingDiv.id = 'loading';
    loadingDiv.textContent = 'Loading CCSNES...';
    loadingDiv.style.cssText = 'position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);';
    document.body.appendChild(loadingDiv);
    
    try {
        await init();
        
        // Initialize audio context
        audioContext = new (window.AudioContext || window.webkitAudioContext)({
            sampleRate: 32000
        });
        
        document.getElementById('loading').style.display = 'none';
        console.log('CCSNES WASM module loaded successfully');
    } catch (error) {
        console.error('Failed to initialize WASM module:', error);
        document.getElementById('loading').textContent = 'Failed to load emulator';
    }
}

// Load ROM file
function loadROM(arrayBuffer) {
    try {
        const romData = new Uint8Array(arrayBuffer);
        
        if (emulator) {
            emulator.free();
        }
        
        emulator = new WasmEmulator('screen');
        emulator.load_rom(romData);
        
        isPaused = false;
        updateControlStates();
        
        if (!animationId) {
            startEmulation();
        }
        
        return true;
    } catch (error) {
        console.error('Failed to load ROM:', error);
        alert('Failed to load ROM: ' + error.message);
        return false;
    }
}

// Start emulation loop
function startEmulation() {
    const canvas = document.getElementById('screen');
    const ctx = canvas.getContext('2d');
    const imageData = ctx.createImageData(256, 224);
    
    let lastTime = performance.now();
    let fpsTime = lastTime;
    let fpsFrames = 0;
    
    function frame(currentTime) {
        if (!emulator || isPaused) {
            animationId = null;
            return;
        }
        
        // Calculate delta time
        const deltaTime = currentTime - lastTime;
        lastTime = currentTime;
        
        // Run emulation
        try {
            // Run one frame (handles rendering internally)
            emulator.run_frame();
            
            // TODO: Process audio when implemented
            
            // Update FPS counter
            fpsFrames++;
            if (currentTime - fpsTime >= 1000) {
                const fpsElement = document.getElementById('fps');
                if (fpsElement) fpsElement.textContent = fpsFrames;
                fpsFrames = 0;
                fpsTime = currentTime;
            }
        } catch (error) {
            console.error('Emulation error:', error);
            pauseEmulation();
        }
        
        animationId = requestAnimationFrame(frame);
    }
    
    animationId = requestAnimationFrame(frame);
}

// Process audio samples
function processAudio() {
    if (!emulator || !audioContext) return;
    
    const samples = emulator.get_audio_buffer();
    if (samples.length === 0) return;
    
    // Create audio buffer
    const buffer = audioContext.createBuffer(2, samples.length / 2, 32000);
    const leftChannel = buffer.getChannelData(0);
    const rightChannel = buffer.getChannelData(1);
    
    // Deinterleave stereo samples
    for (let i = 0; i < samples.length; i += 2) {
        leftChannel[i / 2] = samples[i];
        rightChannel[i / 2] = samples[i + 1];
    }
    
    // Play buffer
    const source = audioContext.createBufferSource();
    source.buffer = buffer;
    source.connect(audioContext.destination);
    source.start();
}

// Pause emulation
function pauseEmulation() {
    isPaused = true;
    updateControlStates();
}

// Resume emulation
function resumeEmulation() {
    if (!emulator) return;
    
    isPaused = false;
    updateControlStates();
    
    if (!animationId) {
        startEmulation();
    }
}

// Update control button states
function updateControlStates() {
    const playPauseBtn = document.getElementById('play-pause-btn');
    const resetBtn = document.getElementById('reset-btn');
    
    playPauseBtn.textContent = isPaused ? 'Play' : 'Pause';
    playPauseBtn.disabled = !emulator;
    resetBtn.disabled = !emulator;
}

// Event handlers
document.addEventListener('DOMContentLoaded', async () => {
    await initEmulator();
    
    // ROM input
    const romInput = document.getElementById('rom-input');
    const loadRomBtn = document.getElementById('load-rom-btn');
    
    loadRomBtn.addEventListener('click', () => {
        romInput.click();
    });
    
    romInput.addEventListener('change', (event) => {
        const file = event.target.files[0];
        if (file) {
            const reader = new FileReader();
            reader.onload = (e) => {
                if (loadROM(e.target.result)) {
                    console.log('ROM loaded:', file.name);
                    document.getElementById('rom-status').textContent = file.name;
                }
            };
            reader.readAsArrayBuffer(file);
        }
    });
    
    // Control buttons
    document.getElementById('play-pause-btn').addEventListener('click', () => {
        if (isPaused) {
            resumeEmulation();
        } else {
            pauseEmulation();
        }
    });
    
    document.getElementById('reset-btn').addEventListener('click', () => {
        if (emulator) {
            emulator.reset();
        }
    });
    
    document.getElementById('fullscreen-btn').addEventListener('click', () => {
        const canvas = document.getElementById('screen');
        if (canvas.requestFullscreen) {
            canvas.requestFullscreen();
        } else if (canvas.webkitRequestFullscreen) {
            canvas.webkitRequestFullscreen();
        }
    });
    
    
    // Keyboard input
    document.addEventListener('keydown', (event) => {
        if (emulator && !isPaused) {
            emulator.handle_key_down(event);
            if (keyMap.hasOwnProperty(event.code)) {
                event.preventDefault();
            }
        }
    });
    
    document.addEventListener('keyup', (event) => {
        if (emulator) {
            emulator.handle_key_up(event);
            if (keyMap.hasOwnProperty(event.code)) {
                event.preventDefault();
            }
        }
    });
    
    // Prevent context menu on canvas
    document.getElementById('screen').addEventListener('contextmenu', (e) => {
        e.preventDefault();
    });
});

// Handle visibility change
document.addEventListener('visibilitychange', () => {
    if (document.hidden && !isPaused) {
        pauseEmulation();
    }
});