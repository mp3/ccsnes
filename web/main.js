import init, { 
    SnesEmulator, 
    InputMapper, 
    log
} from './pkg/ccsnes.js';

class EmulatorApp {
    constructor() {
        this.emulator = null;
        this.isRunning = false;
        this.animationId = null;
        this.audioContext = null;
        this.audioBuffer = [];
        this.pressedKeys = new Set();
        this.currentButtons = 0;
        
        this.elements = {
            screen: document.getElementById('screen'),
            romInput: document.getElementById('rom-input'),
            loadRomBtn: document.getElementById('load-rom-btn'),
            playPauseBtn: document.getElementById('play-pause-btn'),
            resetBtn: document.getElementById('reset-btn'),
            fullscreenBtn: document.getElementById('fullscreen-btn'),
            romStatus: document.getElementById('rom-status'),
            volumeSlider: document.getElementById('volume-slider'),
            volumeDisplay: document.getElementById('volume-display'),
            romInfo: document.getElementById('rom-info'),
            romDetails: document.getElementById('rom-details')
        };
        
        this.initEventListeners();
    }

    async init() {
        try {
            // Initialize WASM module
            await init();
            
            log('WASM module initialized');
            
            // Create emulator instance
            this.emulator = new SnesEmulator('screen');
            log('Emulator created');
            
            // Initialize audio context
            this.initAudio();
            
            this.updateUI();
        } catch (error) {
            console.error('Failed to initialize emulator:', error);
            this.showError('Failed to initialize emulator: ' + error.message);
        }
    }

    initEventListeners() {
        // File loading
        this.elements.loadRomBtn.addEventListener('click', () => {
            this.elements.romInput.click();
        });
        
        this.elements.romInput.addEventListener('change', (e) => {
            this.loadROM(e.target.files[0]);
        });
        
        // Emulator controls
        this.elements.playPauseBtn.addEventListener('click', () => {
            this.togglePlayPause();
        });
        
        this.elements.resetBtn.addEventListener('click', () => {
            this.reset();
        });
        
        this.elements.fullscreenBtn.addEventListener('click', () => {
            this.toggleFullscreen();
        });
        
        // Volume control
        this.elements.volumeSlider.addEventListener('input', (e) => {
            const volume = e.target.value;
            this.elements.volumeDisplay.textContent = volume + '%';
            if (this.audioContext) {
                this.audioContext.volume = volume / 100;
            }
        });
        
        // Keyboard input
        document.addEventListener('keydown', (e) => this.handleKeyDown(e));
        document.addEventListener('keyup', (e) => this.handleKeyUp(e));
        
        // Prevent default behavior for game keys
        document.addEventListener('keydown', (e) => {
            const gameKeys = [
                'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight',
                'KeyZ', 'KeyX', 'KeyA', 'KeyS', 'KeyJ', 'KeyK', 'KeyU', 'KeyI',
                'KeyQ', 'KeyW', 'KeyO', 'KeyP', 'Enter', 'Space'
            ];
            if (gameKeys.includes(e.code)) {
                e.preventDefault();
            }
        });
        
        // Handle file drops
        document.addEventListener('dragover', (e) => {
            e.preventDefault();
        });
        
        document.addEventListener('drop', (e) => {
            e.preventDefault();
            const files = e.dataTransfer.files;
            if (files.length > 0) {
                this.loadROM(files[0]);
            }
        });
    }

    async loadROM(file) {
        if (!file) return;
        
        try {
            this.elements.romStatus.innerHTML = '<span class="loading"></span> Loading...';
            
            const arrayBuffer = await file.arrayBuffer();
            const uint8Array = new Uint8Array(arrayBuffer);
            
            await this.emulator.load_rom(uint8Array);
            
            this.elements.romStatus.innerHTML = '<span class="success">ROM loaded: ' + file.name + '</span>';
            this.elements.playPauseBtn.disabled = false;
            this.elements.resetBtn.disabled = false;
            
            // Show ROM info (placeholder - would need to expose ROM header info from Rust)
            this.showRomInfo(file.name, uint8Array.length);
            
            log('ROM loaded successfully');
        } catch (error) {
            console.error('Failed to load ROM:', error);
            this.elements.romStatus.innerHTML = '<span class="error">Failed to load ROM</span>';
            this.showError('Failed to load ROM: ' + error.message);
        }
    }

    togglePlayPause() {
        if (!this.emulator) return;
        
        if (this.isRunning) {
            this.pause();
        } else {
            this.play();
        }
    }

    play() {
        if (!this.emulator) return;
        
        this.isRunning = true;
        this.emulator.resume();
        this.elements.playPauseBtn.textContent = 'Pause';
        this.startEmulationLoop();
        log('Emulation started');
    }

    pause() {
        if (!this.emulator) return;
        
        this.isRunning = false;
        this.emulator.pause();
        this.elements.playPauseBtn.textContent = 'Play';
        if (this.animationId) {
            cancelAnimationFrame(this.animationId);
            this.animationId = null;
        }
        log('Emulation paused');
    }

    reset() {
        if (!this.emulator) return;
        
        try {
            this.emulator.reset();
            log('Emulator reset');
        } catch (error) {
            console.error('Failed to reset emulator:', error);
            this.showError('Failed to reset: ' + error.message);
        }
    }

    startEmulationLoop() {
        if (!this.isRunning) return;
        
        try {
            // Update controller state
            this.emulator.set_controller_state(0, this.currentButtons);
            
            // Step one frame
            this.emulator.step_frame();
            
            // Queue next frame
            this.animationId = requestAnimationFrame(() => this.startEmulationLoop());
        } catch (error) {
            console.error('Emulation error:', error);
            this.pause();
            this.showError('Emulation error: ' + error.message);
        }
    }

    handleKeyDown(e) {
        this.pressedKeys.add(e.code);
        this.updateControllerState();
    }

    handleKeyUp(e) {
        this.pressedKeys.delete(e.code);
        this.updateControllerState();
    }

    updateControllerState() {
        this.currentButtons = 0;
        
        for (const key of this.pressedKeys) {
            this.currentButtons |= InputMapper.keyboard_to_snes_buttons(key);
        }
    }

    initAudio() {
        try {
            this.audioContext = new (window.AudioContext || window.webkitAudioContext)();
            log('Audio context initialized');
        } catch (error) {
            console.error('Failed to initialize audio:', error);
        }
    }

    toggleFullscreen() {
        const canvas = this.elements.screen;
        
        if (document.fullscreenElement) {
            document.exitFullscreen();
        } else {
            canvas.requestFullscreen().catch(err => {
                console.error('Failed to enter fullscreen:', err);
            });
        }
    }

    showRomInfo(filename, size) {
        const sizeKB = Math.round(size / 1024);
        this.elements.romDetails.innerHTML = `
            <p><strong>File:</strong> ${filename}</p>
            <p><strong>Size:</strong> ${sizeKB} KB</p>
            <p><strong>Status:</strong> Loaded</p>
        `;
        this.elements.romInfo.style.display = 'block';
    }

    showError(message) {
        // Create a simple error display
        const errorDiv = document.createElement('div');
        errorDiv.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            background: #ff6b6b;
            color: white;
            padding: 15px;
            border-radius: 10px;
            z-index: 1000;
            max-width: 300px;
        `;
        errorDiv.textContent = message;
        
        document.body.appendChild(errorDiv);
        
        setTimeout(() => {
            document.body.removeChild(errorDiv);
        }, 5000);
    }

    updateUI() {
        // Update UI state based on emulator status
        const hasROM = this.emulator && this.elements.romStatus.textContent.includes('loaded');
        this.elements.playPauseBtn.disabled = !hasROM;
        this.elements.resetBtn.disabled = !hasROM;
    }
}

// Initialize the application
const app = new EmulatorApp();
app.init().catch(error => {
    console.error('Failed to initialize application:', error);
});