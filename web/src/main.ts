import init, { Nes, createConsole, nextFrame, updateJoypad1 } from '../public/pkg/nessy';

const WIDTH = 256; // px
const HEIGHT = 240; // px

const roms = {
    Metroid: 'Metroid',
    BalloonFight: 'Balloon Fight',
    SuperMarioBros: 'Super Mario Bros',
    PacMan: 'Pac-Man',
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
    KidIcarus: 'Kid Icarus',
};

const game = roms.DrMario;

document.addEventListener('DOMContentLoaded', () => {
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
    window.addEventListener('resize', resize);

    const ctx = canvas.getContext('2d')!;
    const imageData = ctx.createImageData(WIDTH, HEIGHT);

    enum JoypadStatus {
        A = 0b0000_0001,
        B = 0b0000_0010,
        SELECT = 0b0000_0100,
        START = 0b0000_1000,
        UP = 0b0001_0000,
        DOWN = 0b0010_0000,
        LEFT = 0b0100_0000,
        RIGHT = 0b1000_0000,
    };

    function handleInput(nes: Nes, event: KeyboardEvent, pressed: boolean): void {
        switch (event.key) {
            case 'w':
                updateJoypad1(nes, JoypadStatus.UP, pressed);
                break;
            case 'a':
                updateJoypad1(nes, JoypadStatus.LEFT, pressed);
                break;
            case 's':
                updateJoypad1(nes, JoypadStatus.DOWN, pressed);
                break;
            case 'd':
                updateJoypad1(nes, JoypadStatus.RIGHT, pressed);
                break;
            case 'k':
                updateJoypad1(nes, JoypadStatus.B, pressed);
                break;
            case 'l':
                updateJoypad1(nes, JoypadStatus.A, pressed);
                break;
            case 'Enter':
                event.preventDefault();
                updateJoypad1(nes, JoypadStatus.START, pressed);
                break;
            case ' ':
                event.preventDefault();
                updateJoypad1(nes, JoypadStatus.SELECT, pressed);
                break;
        }
    }

    (async () => {
        await init();
        const rom = await fetch(`roms/${game}.nes`);
        const bytes = await rom.arrayBuffer();
        const nes = createConsole(new Uint8Array(bytes));
        const frame = new Uint8Array(imageData.data);

        window.addEventListener('keydown', (e) => handleInput(nes, e, true));
        window.addEventListener('keyup', (e) => handleInput(nes, e, false));

        function run(): void {
            requestAnimationFrame(run);
            nextFrame(nes, frame);
            imageData.data.set(frame);
            ctx.putImageData(imageData, 0, 0);
        }

        run();
    })();
});
