import init, { Nes, createConsole, nextFrame, setJoypad1 } from '../public/pkg/nessy';

const WIDTH = 256; // px
const HEIGHT = 240; // px

const roms = {
    Metroid: 'Metroid',
    BalloonFight: 'Balloon Fight',
    SuperMarioBros: 'Super Mario Bros',
    PacMan: 'Pac-Man',
    DonkeyKong: 'Donkey Kong',
    DonkeyKongJr: 'Donkey Kong Jr',
    Tetris: 'Tetris',
    DrMario: 'Dr. Mario',
    IceClimber: 'Ice Climber',
    Pinball: 'Pinball',
    Bomberman: 'Bomberman',
    Tennis: 'Tennis',
    Spelunker: 'Spelunker',
    UrbanChampion: 'Urban Champion',
    Excitebike: 'Excitebike',
    Zelda: 'Zelda',
    Zelda2: 'Zelda II',
    KidIcarus: 'Kid Icarus',
    MegaMan: 'Mega Man',
    MegaMan2: 'Mega Man 2',
    Castlevania: 'Castlevania',
    Castlevania2: 'Castlevania II',
    Contra: 'Contra',
    Chessmaster: 'Chessmaster',
    NinjaTurtles: 'Teenage Mutant Ninja Turtles',
    PrinceOfPersia: 'Prince of Persia',
    DuckTales: 'Duck Tales',
    MetalGear: 'Metal Gear',
    GhostsNGoblins: "Ghosts 'N Goblins",
    BackToTheFuture: 'Back to the Future',
    // BackToTheFuture2And3: 'Back to the Future II & III',
};

const game = roms.KidIcarus;

enum Joypad {
    A = 0b0000_0001,
    B = 0b0000_0010,
    Select = 0b0000_0100,
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
    ' ': Joypad.Select,
};

type Mode = {
    type: 'play',
} | {
    type: 'replay',
    inputs: number[],
} | {
    type: 'loadSave',
    inputs: number[],
};

const createController = (nes: Nes) => {
    let currentFrame = 0;
    let state = 0;
    let changed = false;
    const history: number[] = [];
    const localStorageKey = `nessy.inputs.${game}.${Date.now()}`;

    function handleInput(nes: Nes, event: KeyboardEvent, pressed: boolean): void {
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
            localStorage.setItem(localStorageKey, JSON.stringify(history));
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
    const inputs = await (await fetch(`inputs/zelda2.json`)).json();
    const mode: Mode = {
        type: 'play',
        // inputs,
    };

    await init();
    const rom = await (await fetch(`roms/${game}.nes`)).arrayBuffer();
    const nes = createConsole(new Uint8Array(rom));
    const frame: Uint8Array = imageData.data as any;
    const controller = createController(nes);
    const replay = createReplay(nes, inputs);

    window.addEventListener('resize', resize);
    window.addEventListener('keyup', controller.onKeyUp);
    window.addEventListener('keydown', controller.onKeyDown);
    window.addEventListener('beforeunload', controller.save);

    function run(): void {
        requestAnimationFrame(run);

        nextFrame(nes, frame);
        ctx.putImageData(imageData, 0, 0);

        if (mode.type === 'play' || mode.type === 'loadSave') {
            controller.tick();
        }

        if (mode.type === 'replay') {
            replay.tick();
        }
    }

    run();
}

document.addEventListener('DOMContentLoaded', setup);
