
export function createWebglRenderer(canvas: HTMLCanvasElement) {
    const gl = canvas.getContext('webgl')!;

    if (gl == null) {
        throw new Error('Unable to get WebGL context. Your browser may not support it.');
    }

    // Create a texture.
    const texture = gl.createTexture();

    if (texture == null) {
        throw new Error('Unable to create texture.');
    }

    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

    const buffer = gl.createBuffer();

    if (buffer == null) {
        throw new Error('Unable to create buffer.');
    }

    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(
        gl.ARRAY_BUFFER,
        new Float32Array([
            -1.0, -1.0,
            1.0, -1.0,
            -1.0, 1.0,
            -1.0, 1.0,
            1.0, -1.0,
            1.0, 1.0
        ]),
        gl.STATIC_DRAW
    );

    const vertexShaderSource = `
        attribute vec2 a_position;
        varying vec2 v_texCoord;

        void main() {
            gl_Position = vec4(a_position, 0, 1);
            v_texCoord = a_position * vec2(0.5, -0.5) + 0.5;
        }
    `;

    const fragmentShaderSource = `
        precision mediump float;
        uniform sampler2D u_image;
        varying vec2 v_texCoord;

        void main() {
            gl_FragColor = texture2D(u_image, v_texCoord);
        }
    `;

    const vertexShader = createShader(gl.VERTEX_SHADER, vertexShaderSource);
    const fragmentShader = createShader(gl.FRAGMENT_SHADER, fragmentShaderSource);
    const program = createProgram(vertexShader, fragmentShader);
    const posAttribLoc = gl.getAttribLocation(program, "a_position");
    gl.viewport(0, 0, 256, 240);

    function createShader(type: number, source: string): WebGLShader {
        const shader = gl.createShader(type);

        if (!shader) {
            throw new Error('Unable to create shader.');
        }

        gl.shaderSource(shader, source);
        gl.compileShader(shader);
        if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
            const info = gl.getShaderInfoLog(shader);
            throw 'Could not compile WebGL shader. \n\n' + info;
        }

        return shader;
    }

    function createProgram(vertexShader: WebGLShader, fragmentShader: WebGLShader): WebGLProgram {
        const program = gl.createProgram();

        if (!program) {
            throw new Error('Unable to create program.');
        }

        gl.attachShader(program, vertexShader);
        gl.attachShader(program, fragmentShader);
        gl.linkProgram(program);

        if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
            const info = gl.getProgramInfoLog(program);
            throw 'Could not compile WebGL program. \n\n' + info;
        }

        return program;
    }

    function render(frame: Uint8Array): void {
        gl.useProgram(program);
        gl.enableVertexAttribArray(posAttribLoc);
        gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
        gl.vertexAttribPointer(posAttribLoc, 2, gl.FLOAT, false, 0, 0);
        gl.bindTexture(gl.TEXTURE_2D, texture);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, canvas.width, canvas.height, 0, gl.RGB, gl.UNSIGNED_BYTE, frame);
        gl.drawArrays(gl.TRIANGLES, 0, 6);
    }

    return { render };
}
