import init, {
    Nes, createConsole,
    loadState,
    nextFrame,
    saveState,
    setJoypad1
} from '../public/pkg/nessy';
import { createController } from './controls';
import { events } from './ui/events';
import { StoreData, createStore } from './ui/store';
import { createUI } from './ui/ui';
const WIDTH = 256; // px
const HEIGHT = 240; // px

const createReplay = (nes: Nes, inputs: number[]) => {
    let currentFrame = 0;
    let index = 0;
    const ret = { tick, reachEnd, isOver: false };

    function tick(): void {
        if (!ret.isOver) {
            currentFrame += 1;
            const frame = inputs[index];

            if (frame === currentFrame + 1) {
                const buttons = inputs[index + 1];
                setJoypad1(nes, buttons);
                index += 2;
            }

            if (index >= inputs.length) {
                ret.isOver = true;
            }
        }
    }

    function reachEnd(buffer: Uint8Array): void {
        while (!ret.isOver) {
            nextFrame(nes, buffer);
            tick();
        }
    }

    return ret;
};

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
    const ui = createUI(store);

    const updateROM = (rom: Uint8Array): void => {
        ui.showUI.ref = false;
        nes = createConsole(rom);

        const onKeyDown = (event: KeyboardEvent) => {
            if (!ui.showUI.ref) {
                controller.onKeyDown(event);
            }
        };

        const onKeyUp = (event: KeyboardEvent) => {
            if (!ui.showUI.ref) {
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
    };

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

    loadROM(store.ref.rom);

    store.subscribe('rom', async (rom) => {
        await loadROM(rom);
    });

    events.on('loadRequest', async ({ timestamp }) => {
        const save = await store.db.save.get(timestamp);
        loadState(nes, save.state);
        ui.showUI.ref = false;
        events.emit('loaded', { timestamp });
    });

    events.on('loadLastRequest', async () => {
        const save = await store.db.save.getLast(store.ref.rom!);
        if (save != null) {
            loadState(nes, save.state);
            ui.showUI.ref = false;
        }
    });

    events.on('saveRequest', async () => {
        const state = saveState(nes);
        const timestamp = await store.db.save.insert(store.ref.rom!, state);
        events.emit('saved', { timestamp });
    });

    const renderState = (state: Uint8Array, buffer: Uint8Array): void => {
        loadState(nes, state);
        nextFrame(nes, buffer);
    };

    let pausedState: Uint8Array | null = null;

    events.on('setBackgroundRequest', async event => {
        switch (event.mode) {
            case 'current': {
                if (pausedState != null) {
                    renderState(pausedState, frame);
                }
                break;
            }
            case 'at': {
                const save = await store.db.save.get(event.timestamp);
                renderState(save.state, frame);
                break;
            }
            case 'titleScreen': {
                const titleScreen = await store.db.titleScreen.get(event.hash);
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

    events.on('uiToggled', ({ visible }) => {
        if (visible) {
            pausedState = saveState(nes);
            events.emit('setBackgroundRequest', { mode: 'current' });
        } else {
            if (pausedState !== null) {
                loadState(nes, pausedState);
            }
        }
    });

    const titleScreenFrame = new Uint8Array(256 * 240 * 4);

    events.on('generateTitleScreenRequest', async ({ hash }) => {
        try {
            const rom = await store.db.rom.get(hash);
            const titleScreen = await store.db.titleScreen.get(hash);

            if (titleScreen == null) {
                const titleScreenNes = createConsole(rom.data);

                // Generate the screenshot after 2 seconds
                for (let i = 0; i < 120; i++) {
                    nextFrame(titleScreenNes, titleScreenFrame);
                }

                await store.db.titleScreen.insert(hash, titleScreenFrame);
                events.emit('titleScreenGenerated', { hash, data: titleScreenFrame });
            } else {
                events.emit('titleScreenGenerated', { hash, data: titleScreen.data });
            }
        } catch (error) {
            console.error(`Failed to generate title screen for ${hash}: ${error}`);
            titleScreenFrame.fill(0);
            events.emit('titleScreenGenerated', { hash, data: titleScreenFrame });
        }
    });

    function run(): void {
        requestAnimationFrame(run);

        if (!ui.showUI.ref) {
            if (nes !== undefined) {
                nextFrame(nes, frame);
                controller.tick();
            }
        } else {
            ui.render(imageData);
        }

        ctx.putImageData(imageData, 0, 0);
    }

    run();

    window.addEventListener('beforeunload', store.save);
}

window.addEventListener('DOMContentLoaded', setup);
