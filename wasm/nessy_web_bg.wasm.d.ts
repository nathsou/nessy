/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const __wbg_nes_free: (a: number, b: number) => void;
export const nes_new: (a: number, b: number, c: number) => [number, number, number];
export const nes_softReset: (a: number) => void;
export const nes_nextFrame: (a: number, b: number, c: number, d: any) => void;
export const nes_nextSamples: (a: number, b: number, c: number, d: any) => number;
export const nes_fillFrameBuffer: (a: number, b: number, c: number, d: any) => void;
export const nes_setJoypad1: (a: number, b: number) => void;
export const nes_setJoypad2: (a: number, b: number) => void;
export const nes_saveState: (a: number) => [number, number];
export const nes_loadState: (a: number, b: number, c: number) => [number, number];
export const nes_fillAudioBuffer: (a: number, b: number, c: number, d: any, e: number) => void;
export const nes_clearAudioBuffer: (a: number) => void;
export const nes_initPanicHook: () => void;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
export const __wbindgen_export_3: WebAssembly.Table;
export const __externref_table_dealloc: (a: number) => void;
export const __wbindgen_start: () => void;
