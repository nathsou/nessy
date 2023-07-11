
const createHooks = <Hooks extends Record<string, (...args: any[]) => any>>() => {
    const hooks: Partial<Hooks> = {};

    return {
        register<H extends keyof Hooks>(hook: H, func: Hooks[H]): void {
            hooks[hook] = func;
        },
        call<H extends keyof Hooks>(hook: H, ...args: Parameters<Hooks[H]>): ReturnType<Hooks[H]> {
            const func = hooks[hook];
            if (func != null) {
                return func.apply(null, args);
            } else {
                throw new Error(`Hook ${String(hook)} is not registered`);
            }
        },
    };
};

export const hooks = createHooks<{
    loadSave(timestamp: number): Promise<void>,
    loadLastSave(): Promise<void>,
    saveState(): Promise<Uint8Array>,
    toggleFullscreen(): void,
    generateTitleScreen(hash: string): Promise<Uint8Array>,
    setBackground(
        mode:
            { mode: 'current' } |
            { mode: 'at', timestamp: number } |
            { mode: 'titleScreen', hash: string }
    ): void,
}>();