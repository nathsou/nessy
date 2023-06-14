import init, { createConsole, nextFrame } from '../public/pkg/nessy';

const WIDTH = 256; // px
const HEIGHT = 240; // px

document.addEventListener('DOMContentLoaded', () => {
    const gameURL = new URL('../roms/Balloon_Fight.nes', import.meta.url).href;
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

    function render(frame: Uint8Array): void {
        const data = imageData.data;
        let dataIdx = 0;
        let frameIdx = 0;

        for (let i = 0; i < WIDTH * HEIGHT; i++) {
            data[dataIdx] = frame[frameIdx];
            data[dataIdx + 1] = frame[frameIdx + 1];
            data[dataIdx + 2] = frame[frameIdx + 2];
            data[dataIdx + 3] = 255;

            dataIdx += 4;
            frameIdx += 3;
        }

        ctx.putImageData(imageData, 0, 0);
    }

    (async () => {
        await init();
        const rom = await fetch(gameURL);
        const bytes = await rom.arrayBuffer();
        const nes = createConsole(new Uint8Array(bytes));

        function run(): void {
            const frame = nextFrame(nes);
            render(frame);
            requestAnimationFrame(run);
        }

        run();
    })();
});
