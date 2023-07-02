
const array = <T>(): T[] => [];
const union = <T>(value: T): T => value;
export type Store = typeof store;

export const store = {
    frameCount: 0,
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
