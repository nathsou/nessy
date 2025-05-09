/* tslint:disable */
/* eslint-disable */
export class Nes {
  private constructor();
  free(): void;
  static initPanicHook(): void;
  static new(rom: Uint8Array, sample_rate: number): Nes;
  softReset(): void;
  nextFrame(buffer: Uint8Array): void;
  nextSamples(audio_buffer: Float32Array): boolean;
  fillFrameBuffer(buffer: Uint8Array): void;
  setJoypad1(buttons: number): void;
  setJoypad2(buttons: number): void;
  saveState(): Uint8Array;
  loadState(data: Uint8Array): void;
  fillAudioBuffer(buffer: Float32Array, avoid_underruns: boolean): void;
  clearAudioBuffer(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_nes_free: (a: number, b: number) => void;
  readonly nes_new: (a: number, b: number, c: number) => [number, number, number];
  readonly nes_softReset: (a: number) => void;
  readonly nes_nextFrame: (a: number, b: number, c: number, d: any) => void;
  readonly nes_nextSamples: (a: number, b: number, c: number, d: any) => number;
  readonly nes_fillFrameBuffer: (a: number, b: number, c: number, d: any) => void;
  readonly nes_setJoypad1: (a: number, b: number) => void;
  readonly nes_setJoypad2: (a: number, b: number) => void;
  readonly nes_saveState: (a: number) => [number, number];
  readonly nes_loadState: (a: number, b: number, c: number) => [number, number];
  readonly nes_fillAudioBuffer: (a: number, b: number, c: number, d: any, e: number) => void;
  readonly nes_clearAudioBuffer: (a: number) => void;
  readonly nes_initPanicHook: () => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_3: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
