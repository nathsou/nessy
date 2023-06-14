import init, { createConsole, nextFrame } from '../public/pkg/nessy';

const WIDTH = 256; // px
const HEIGHT = 240; // px

document.addEventListener('DOMContentLoaded', () => {
    const gameURL = new URL('../roms/Balloon Fight.nes', import.meta.url).href;
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

    (async () => {
        await init();
        const rom = await fetch(gameURL);
        const bytes = await rom.arrayBuffer();
        const nes = createConsole(new Uint8Array(bytes));
        const frame = new Uint8Array(imageData.data);

        function run(): void {
            nextFrame(nes, frame);
            imageData.data.set(frame);
            ctx.putImageData(imageData, 0, 0);
            requestAnimationFrame(run);
        }

        run();
    })();
});
