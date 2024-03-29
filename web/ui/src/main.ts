import init, { Nes } from '../public/wasm/nessy_web';
import { createController } from './controls';
import { createWebglRenderer } from './webgl';
import { events } from './ui/events';
import { hooks } from './ui/hooks';
import { StoreData, createStore } from './ui/store';
import { createUI } from './ui/ui';

const WIDTH = 256; // px
const HEIGHT = 240; // px
type SyncMode = 0 | 1 | 2;
const SYNC_VIDEO: SyncMode = 0;
const SYNC_AUDIO: SyncMode = 1;
const SYNC_BOTH: SyncMode = 2;

const AUDIO_BUFFER_SIZE_MAPPING = {
    [SYNC_VIDEO]: 1024,
    [SYNC_AUDIO]: 512,
    [SYNC_BOTH]: 1024,
};

const SCALING_MODE_MAPPING: Record<StoreData['scalingMode'], HTMLCanvasElement['style']['imageRendering']> = {
    pixelated: 'pixelated',
    blurry: 'auto',
};

async function setup() {
    await init();
    Nes.initPanicHook();
    const store = await createStore();
    const ui = createUI(store);
    const syncMode = SYNC_BOTH;
    const audioBufferSize = AUDIO_BUFFER_SIZE_MAPPING[syncMode];
    const avoidUnderruns = syncMode === SYNC_BOTH;
    const canvas = document.querySelector<HTMLCanvasElement>('#screen')!;
    const renderer = createWebglRenderer(canvas);
    let nes: Nes;
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
        if (!ui.visible) {
            hooks.call('toggleUI');
        }
    });

    // TODO: use an AudioWorkletNode
    const scriptProcessor = audioCtx.createScriptProcessor(audioBufferSize, 0, 1);
    scriptProcessor.onaudioprocess = ((): ScriptProcessorNode['onaudioprocess'] => {
        if (syncMode === SYNC_AUDIO) {
            return (event: AudioProcessingEvent) => {
                if (!ui.visible) {
                    const channel = event.outputBuffer.getChannelData(0);

                    if (nes !== undefined) {
                        const newFrameReady = nes.nextSamples(channel);

                        if (newFrameReady) {
                            nes.fillFrameBuffer(frame);
                            renderer.render(frame);
                        }
                    }
                }
            };
        } else {
            return (event: AudioProcessingEvent) => {
                if (!ui.visible) {
                    const channel = event.outputBuffer.getChannelData(0);
                    nes.fillAudioBuffer(channel, avoidUnderruns);
                }
            };
        }
    })();

    scriptProcessor.connect(audioCtx.destination);

    hooks.register('input', (action, pressed) => {
        if (pressed) {
            ui.onAction(action);
        }
    });

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
        nes = Nes.new(rom, audioCtx.sampleRate);
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
        const prevState = nes.saveState();
        nes.loadState(state);
        nes.nextFrame(buffer);
        nes.loadState(prevState);
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
                nes.loadState(store.ref.lastState);
                nes.nextFrame(backgroundFrame);
                nes.loadState(store.ref.lastState);

                hooks.call('setBackground', { mode: 'current' });
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

                // Generate the screenshot after 2 seconds
                for (let i = 0; i < 120; i++) {
                    titleScreenNes.nextFrame(titleScreenFrame);
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
            if (syncMode !== SYNC_AUDIO) {
                nes.nextFrame(frame);
                renderer.render(frame);
            }
        }
    }

    await onInit();
    window.addEventListener('beforeunload', onExit);
}

window.addEventListener('DOMContentLoaded', setup);
