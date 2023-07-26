/* tslint:disable */
/* eslint-disable */
/**
*/
export class Nes {
  free(): void;
/**
*/
  static initPanicHook(): void;
/**
* @param {Uint8Array} rom
* @param {number} sample_rate
* @returns {Nes}
*/
  static new(rom: Uint8Array, sample_rate: number): Nes;
/**
*/
  softReset(): void;
/**
*/
  nextFrame(): void;
/**
* @param {Float32Array} audio_buffer
* @returns {boolean}
*/
  nextSamples(audio_buffer: Float32Array): boolean;
/**
* @param {Uint8Array} buffer
*/
  fillFrameBuffer(buffer: Uint8Array): void;
/**
* @returns {number}
*/
  getUpdatedTilesCount(): number;
/**
* @param {number} buttons
*/
  setJoypad1(buttons: number): void;
/**
* @param {number} buttons
*/
  setJoypad2(buttons: number): void;
/**
* @returns {Uint8Array}
*/
  saveState(): Uint8Array;
/**
* @param {Uint8Array} data
*/
  loadState(data: Uint8Array): void;
/**
* @param {Float32Array} buffer
* @param {boolean} avoid_underruns
*/
  fillAudioBuffer(buffer: Float32Array, avoid_underruns: boolean): void;
/**
*/
  clearAudioBuffer(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_nes_free: (a: number) => void;
  readonly nes_new: (a: number, b: number, c: number, d: number) => void;
  readonly nes_softReset: (a: number) => void;
  readonly nes_nextFrame: (a: number) => void;
  readonly nes_nextSamples: (a: number, b: number, c: number, d: number) => number;
  readonly nes_fillFrameBuffer: (a: number, b: number, c: number, d: number) => void;
  readonly nes_getUpdatedTilesCount: (a: number) => number;
  readonly nes_setJoypad1: (a: number, b: number) => void;
  readonly nes_setJoypad2: (a: number, b: number) => void;
  readonly nes_saveState: (a: number, b: number) => void;
  readonly nes_loadState: (a: number, b: number, c: number, d: number) => void;
  readonly nes_fillAudioBuffer: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly nes_clearAudioBuffer: (a: number) => void;
  readonly nes_initPanicHook: () => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
