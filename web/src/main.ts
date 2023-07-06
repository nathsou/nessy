import init, {
    Nes, createConsole,
    loadState,
    nextFrame,
    resetConsole,
    saveState,
    setJoypad1,
} from '../public/pkg/nessy';
import { createStore } from './ui/store';
import { createUI } from './ui/ui';
const WIDTH = 256; // px
const HEIGHT = 240; // px

export enum Joypad {
    A = 0b0000_0001,
    B = 0b0000_0010,
    SELECT = 0b0000_0100,
    START = 0b0000_1000,
    UP = 0b0001_0000,
    DOWN = 0b0010_0000,
    LEFT = 0b0100_0000,
    RIGHT = 0b1000_0000,
};

const joypad1Mapping: Record<KeyboardEvent['key'], Joypad> = {
    'w': Joypad.UP,
    'a': Joypad.LEFT,
    's': Joypad.DOWN,
    'd': Joypad.RIGHT,
    'k': Joypad.B,
    'l': Joypad.A,
    'Enter': Joypad.START,
    ' ': Joypad.SELECT,
};

const createController = (nes: Nes) => {
    let currentFrame = 0;
    let state = 0;
    let changed = false;
    const history: number[] = [];
    let isMetaDown = false;

    function handleInput(nes: Nes, event: KeyboardEvent, pressed: boolean): void {
        if (event.key === 'Meta') {
            isMetaDown = pressed;
            event.preventDefault();
        }

        if (isMetaDown && pressed) {
            switch (event.key) {
                case 's': {
                    const save = saveState(nes);
                    localStorage.setItem('nessy.save', `[${save.join(',')}]`);
                    event.preventDefault();
                    return;
                }
                case 'r': {
                    resetConsole(nes);
                    event.preventDefault();
                    return;
                }
                case 'l': {
                    const save = localStorage.getItem('nessy.save');
                    if (save != null) {
                        loadState(nes, new Uint8Array(JSON.parse(save)));
                    }
                    event.preventDefault();
                    return;
                }
            }
        }

        if (event.key in joypad1Mapping) {
            if (event.key === 'Enter' || event.key === ' ') {
                event.preventDefault();
            }

            const prevState = state;

            if (pressed) {
                state |= joypad1Mapping[event.key];
            } else {
                state &= ~joypad1Mapping[event.key];
            }

            if (prevState !== state) {
                changed = true;
            }

            setJoypad1(nes, state);
        }
    }

    const onKeyDown = (event: KeyboardEvent) => handleInput(nes, event, true);
    const onKeyUp = (event: KeyboardEvent) => handleInput(nes, event, false);

    function tick(): void {
        currentFrame += 1;

        if (changed) {
            history.push(currentFrame, state);
            changed = false;
        }
    }

    function save(): void {
        if (history.length > 0) {
            // localStorage.setItem(localStorageKey, JSON.stringify(history));
        }
    }

    return {
        onKeyDown,
        onKeyUp,
        history,
        tick,
        save,
    };
};

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

async function setup() {
    const canvas = document.querySelector<HTMLCanvasElement>('#screen')!;
    canvas.width = WIDTH;
    canvas.height = HEIGHT;
    canvas.style.imageRendering = 'pixelated';

    function resize(): void {
        const w = window.innerWidth;
        const h = window.innerHeight;
        const scale = Math.min(w / WIDTH, h / HEIGHT, 3);
        canvas.style.width = `${WIDTH * scale}px`;
        canvas.style.height = `${HEIGHT * scale}px`;
    }

    resize();

    const ctx = canvas.getContext('2d')!;
    const imageData = ctx.createImageData(WIDTH, HEIGHT);

    await init();
    const store = await createStore();
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

        controller = createController(nes);

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
