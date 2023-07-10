import init, { Nes } from '../public/pkg/nessy';
import { createController } from './controls';
import { events } from './ui/events';
import { hooks } from './ui/hooks';
import { Binary, StoreData, createStore } from './ui/store';
import { createUI } from './ui/ui';
const WIDTH = 256; // px
const HEIGHT = 240; // px
const AUDIO_BUFFER_SIZE = 4096; // samples

const SCALING_MODE_MAPPING: Record<StoreData['scalingMode'], HTMLCanvasElement['style']['imageRendering']> = {
    pixelated: 'pixelated',
    blurry: 'auto',
};

async function setup() {
    const store = await createStore();
    const canvas = document.querySelector<HTMLCanvasElement>('#screen')!;
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
    store.subscribe('scalingFactor', resize);
    store.subscribe('scalingMode', () => {
        canvas.style.imageRendering = SCALING_MODE_MAPPING[store.ref.scalingMode];
    });

    const ctx = canvas.getContext('2d')!;
    const imageData = ctx.createImageData(WIDTH, HEIGHT);

    await init();
    let nes: Nes;
    let controller: ReturnType<typeof createController>;
    const frame: Uint8Array = imageData.data as any;
    const audioCtx = new AudioContext();
    const ui = createUI(store, audioCtx);

    // TODO: use an AudioWorkletNode
    const scriptProcessor = audioCtx.createScriptProcessor(AUDIO_BUFFER_SIZE, 0, 1);
    scriptProcessor.onaudioprocess = e => {
        if (!ui.visible.ref) {
            const channel = e.outputBuffer.getChannelData(0);
            nes.fillAudioBuffer(channel);
        }
    };

    scriptProcessor.connect(audioCtx.destination);

    function updateROM(rom: Uint8Array): void {
        ui.visible.ref = false;
        nes = Nes.new(rom, audioCtx.sampleRate);

        const onKeyDown = (event: KeyboardEvent) => {
            if (!ui.visible.ref) {
                controller.onKeyDown(event);
            }
        };

        const onKeyUp = (event: KeyboardEvent) => {
            if (!ui.visible.ref) {
                controller.onKeyUp(event);
            }
        };

        if (controller) {
            window.removeEventListener('resize', resize);
            window.removeEventListener('keyup', onKeyUp);
            window.removeEventListener('keydown', onKeyDown);
            window.removeEventListener('beforeunload', controller.save);
        }

        controller = createController(nes, store);

        window.addEventListener('resize', resize);
        window.addEventListener('keyup', onKeyUp);
        window.addEventListener('keydown', onKeyDown);
        window.addEventListener('beforeunload', controller.save);

        frame.fill(0);
    }

    async function loadROM(hash: string | null): Promise<void> {
        if (hash != null) {
            try {
                const rom = await store.db.rom.get(hash);
                updateROM(rom.data);
            } catch (e) {
                console.error(e);
                store.set('rom', null);
            }
        }
    }

    store.subscribe('rom', async (rom) => {
        await loadROM(rom);
    });

    hooks.register('loadSave', async timestamp => {
        const save = await store.db.save.get(timestamp);
        nes.loadState(save.state);
        ui.visible.ref = false;
    });

    hooks.register('loadLastSave', async () => {
        const save = await store.db.save.getLast(store.ref.rom!);
        if (save != null) {
            nes.loadState(save.state);
            ui.visible.ref = false;
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

    let pausedState: Uint8Array | null = null;

    hooks.register('setBackground', async payload => {
        switch (payload.mode) {
            case 'current': {
                if (pausedState != null) {
                    renderState(pausedState, frame);
                } else {
                    return;
                }
                break;
            }
            case 'at': {
                const save = await store.db.save.get(payload.timestamp);
                renderState(save.state, frame);
                break;
            }
            case 'titleScreen': {
                const titleScreen = await store.db.titleScreen.get(payload.hash);
                if (titleScreen != null) {
                    frame.set(titleScreen.data);
                } else {
                    frame.fill(0);
                }

                break;
            }
        }

        ui.screen.setBackground(frame);
        ctx.putImageData(imageData, 0, 0);
        ui.screen.setBackgroundOpacity(0.2);
    });

    async function onInit() {
        if (store.ref.rom != null) {
            await loadROM(store.ref.rom);

            if (store.ref.lastState != null) {
                nes.loadState(store.ref.lastState);
                nes.nextFrame(frame);
                nes.loadState(store.ref.lastState);

                ui.visible.ref = true;
                pausedState = store.ref.lastState;
                hooks.call('setBackground', { mode: 'current' });
            }
        }

        run();
    }

    events.on('uiToggled', ({ visible }) => {
        if (visible) {
            pausedState = nes.saveState();
            hooks.call('setBackground', { mode: 'current' });
        } else {
            if (pausedState !== null) {
                nes.loadState(pausedState);
            }
        }
    });

    const titleScreenFrame = new Uint8Array(256 * 240 * 4);

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

    function onExit() {
        if (nes != null) {
            store.ref.lastState = nes.saveState();
            Binary.hash(store.ref.lastState).then(hash => {
                console.log(`onExit: ${hash}`);
            });
        }

        store.save();
    }

    function run(): void {
        requestAnimationFrame(run);

        if (!ui.visible.ref) {
            if (nes !== undefined) {
                nes.nextFrame(frame);
                controller.tick();
            }
        } else {
            ui.render(imageData);
        }

        ctx.putImageData(imageData, 0, 0);
    }

    await onInit();

    window.addEventListener('beforeunload', onExit);
}

window.addEventListener('DOMContentLoaded', setup);
