const array = <T>(): T[] => [];
const union = <T>(value: T): T => value;

export type Store = ReturnType<typeof createStore>['ref'];

export const createStore = () => {
    type Store = Readonly<typeof store>;
    const store = {
        frameCount: 0,
        rom: union<number[] | null>(null),
        interface: {
            primary: union<'library' | 'controls'>('controls'),
            secondary: union<string | number>(0),
        },
        controls: {
            up: 'w',
            left: 'a',
            down: 's',
            right: 'd',
            a: 'l',
            b: 'k',
            start: 'Enter',
            select: 'Space',
        },
    };

    const listeners = array<{ key: keyof Store, handler: (value: any) => void }>();

    const get = <K extends keyof Store>(key: K): Store[K] => {
        return store[key];
    };

    const set = <K extends keyof Store>(key: K, value: Store[K]): void => {
        store[key] = value;
        listeners.forEach(({ key: listenerKey, handler }) => {
            if (listenerKey === key) {
                handler(value);
            }
        });
    };

    const subscribe = <K extends keyof Store>(key: K, handler: (value: Store[K]) => void): void => {
        listeners.push({ key, handler });
    };

    return {
        ref: store,
        subscribe,
        get,
        set,
    };
};

export const store = createStore();
