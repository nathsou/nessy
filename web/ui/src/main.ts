import init, { Nes } from '../public/wasm/nessy_web';
import { createController } from './controls';
import { createWebglRenderer } from './webgl';
import { events } from './ui/events';
import { hooks } from './ui/hooks';
import { StoreData, createStore } from './ui/store';
import { createUI } from './ui/ui';

const WIDTH = 256; // px
const HEIGHT = 240; // px
const SCALING_MODE_MAPPING: Record<StoreData['scalingMode'], HTMLCanvasElement['style']['imageRendering']> = {
    pixelated: 'pixelated',
    blurry: 'auto',
};

async function setup() {
    await init();
    Nes.initPanicHook();
    const store = await createStore();
    const ui = createUI(store);
    const audioBufferSize = 512;
    const canvas = document.querySelector<HTMLCanvasElement>('#screen')!;
    const renderer = createWebglRenderer(canvas);
    let nes: Nes;
    let activeRom: Uint8Array;
    const controller = createController(store);
    const frame = new Uint8Array(WIDTH * HEIGHT * 3);
    const audioCtx = new AudioContext();
    const backgroundFrame = new Uint8Array(WIDTH * HEIGHT * 3);

    function attempt<T>(fn: () => T): T {
        try {
            return fn();
        } catch (error) {
            ui.alert({
                text: `${error}`,
                type: 'error',
                frames: 2.5 * 60,
            });

            throw error;
        }
    }

    canvas.width = WIDTH;
    canvas.height = HEIGHT;
    canvas.style.imageRendering = SCALING_MODE_MAPPING[store.ref.scalingMode];

    function resize(): void {
        const w = window.innerWidth;
        const h = window.innerHeight;
        const scale = Math.min(w / WIDTH, h / HEIGHT, store.ref.scalingFactor);
        canvas.style.width = `${WIDTH * scale}px`;
        canvas.style.height = `${HEIGHT * scale}px`;
    }

    resize();
    window.addEventListener('resize', resize);
    store.subscribe('scalingFactor', resize);
    store.subscribe('scalingMode', () => {
        canvas.style.imageRendering = SCALING_MODE_MAPPING[store.ref.scalingMode];
    });

    hooks.register('toggleUI', async visible => {
        if (visible !== undefined) {
            ui.visible = visible;
        } else {
            ui.visible = !ui.visible;
        }

        if (!ui.visible) {
            if (nes !== undefined) {
                nes.clearAudioBuffer();
                await audioCtx.resume();
            }
        } else {
            backgroundFrame.set(frame);
            hooks.call('setBackground', { mode: 'current' });
            await audioCtx.suspend();
        }

        events.emit('uiToggled', { visible: ui.visible });
    });

    window.addEventListener('blur', () => {
        controller.releaseAll();
        if (!ui.visible) {
            hooks.call('toggleUI');
        }
    });

    // TODO: use an AudioWorkletNode
    const scriptProcessor = audioCtx.createScriptProcessor(audioBufferSize, 0, 1);
    // Audio consumption is the only emulation clock; display refresh never advances the CPU.
    scriptProcessor.onaudioprocess = event => {
        const channel = event.outputBuffer.getChannelData(0);
        channel.fill(0);
        if (!ui.visible && nes !== undefined && nes.nextSamples(channel)) {
            nes.fillFrameBuffer(frame);
        }
    };

    scriptProcessor.connect(audioCtx.destination);

    hooks.register('input', action => { ui.onAction(action); });

    const onKeyDown = (event: KeyboardEvent) => {
        const capturedByUI = ui.onKeyDown(event.key);

        if (!capturedByUI) {
            controller?.onKeyDown(event);
        } else {
            event.preventDefault();
        }
    };

    const onKeyUp = (event: KeyboardEvent) => {
        controller?.onKeyUp(event);
    };

    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
    window.addEventListener('gamepadconnected', controller.onGamepadConnected);
    window.addEventListener('gamepaddisconnected', controller.onGamepadDisconnected);

    function updateROM(rom: Uint8Array): void {
        const next = Nes.new(rom, audioCtx.sampleRate);
        nes?.free();
        nes = next;
        activeRom = rom.slice();
        frame.fill(0);
    }

    async function loadROM(hash: string | null): Promise<boolean> {
        if (hash != null) {
            try {
                const rom = await store.db.rom.get(hash);
                updateROM(rom.data);
                return true;
            } catch (error) {
                console.error('Could not load ROM:', error);

                ui.alert({
                    text: `${error}`,
                    type: 'error',
                    frames: 2.5 * 60,
                });

                store.set('rom', null);
            }
        }

        return false;
    }

    store.subscribe('rom', async (rom, prev) => {
        if (rom != null) {
            if (rom !== prev) {
                if (await loadROM(rom)) {
                    hooks.call('toggleUI');
                }
            } else {
                hooks.call('toggleUI');
            }
        }
    });

    hooks.register('loadSave', async timestamp => {
        const save = await store.db.save.get(timestamp);
        attempt(() => {
            nes.loadState(save.state);
            hooks.call('toggleUI', false);
        });
    });

    hooks.register('loadLastSave', async () => {
        const save = await store.db.save.getLast(store.ref.rom!);
        if (save != null) {
            attempt(() => {
                nes.loadState(save.state);
                hooks.call('toggleUI', false);
            });
        }
    });

    hooks.register('saveState', async () => {
        const state = nes.saveState();
        const timestamp = await store.db.save.insert(store.ref.rom!, state);
        events.emit('saved', { timestamp });
        return state;
    });

    function renderState(state: Uint8Array, buffer: Uint8Array): void {
        const preview = Nes.new(activeRom, audioCtx.sampleRate);
        try {
            preview.loadState(state);
            preview.nextFrame(buffer);
        } catch (error) {
            // Older save formats remain in the library, but cannot provide a preview.
            console.warn('Could not render saved preview:', error);
            buffer.fill(0);
        } finally {
            preview.free();
        }
    }

    hooks.register('setBackground', async payload => {
        switch (payload.mode) {
            case 'current': {
                frame.set(backgroundFrame);
                break;
            }
            case 'at': {
                const save = await store.db.save.get(payload.timestamp);
                renderState(save.state, frame);
                break;
            }
            case 'titleScreen': {
                if (store.ref.rom === payload.hash) {
                    frame.set(backgroundFrame);
                } else {
                    const titleScreen = await store.db.titleScreen.get(payload.hash);
                    if (titleScreen != null) {
                        frame.set(titleScreen.data);
                    } else {
                        frame.fill(0);
                    }
                }

                break;
            }
        }

        ui.screen.setBackground(frame, 0.2);
        renderer.render(frame);
    });

    async function onInit() {
        if (store.ref.rom != null) {
            await loadROM(store.ref.rom);

            if (nes && store.ref.lastState != null) {
                try {
                    nes.loadState(store.ref.lastState);
                    renderState(store.ref.lastState, backgroundFrame);
                    hooks.call('setBackground', { mode: 'current' });
                } catch (error) {
                    ui.alert({ text: `Could not resume the saved game: ${error}. Starting a new game.`, type: 'error', frames: 300 });
                }
            }
        }

        run();
    }

    const titleScreenFrame = new Uint8Array(WIDTH * HEIGHT * 3);

    hooks.register('generateTitleScreen', async hash => {
        try {
            const rom = await store.db.rom.get(hash);
            const titleScreen = await store.db.titleScreen.get(hash);

            if (titleScreen == null) {
                const titleScreenNes = Nes.new(rom.data, audioCtx.sampleRate);

                try {
                    // Generate the screenshot after 2 seconds.
                    for (let i = 0; i < 120; i++) titleScreenNes.nextFrame(titleScreenFrame);
                } finally {
                    titleScreenNes.free();
                }

                await store.db.titleScreen.insert(hash, titleScreenFrame);
                return titleScreenFrame;
            } else {
                return titleScreen.data;
            }
        } catch (error) {
            console.error(`Failed to generate title screen for ${hash}: ${error}`);
            titleScreenFrame.fill(0);
            return titleScreenFrame;
        }
    });

    hooks.register('toggleFullscreen', () => {
        if (document.fullscreenElement) {
            document.exitFullscreen();
        } else {
            canvas.requestFullscreen();
        }
    });

    hooks.register('softReset', () => {
        nes?.softReset();
    });

    hooks.register('setJoypad1', state => {
        if (!ui.visible) {
            nes?.setJoypad1(state);
        }
    });

    function onExit() {
        if (nes != null) {
            store.ref.lastState = nes.saveState();
        }

        store.save();
    }

    function run(): void {
        requestAnimationFrame(run);
        controller.tick();

        if (ui.visible) {
            ui.render(frame);
            renderer.render(frame);
        } else if (nes !== undefined) {
            renderer.render(frame);
        }
    }

    await onInit();
    window.addEventListener('beforeunload', onExit);
}

window.addEventListener('DOMContentLoaded', setup);
