import { Nes } from "../public/pkg/nessy";
import { events } from "./ui/events";
import { hooks } from "./ui/hooks";
import { Store } from "./ui/store";

type ControlButton = keyof typeof JOYPAD_MAPPING;

export enum Joypad {
    A = 0b0000_0001,
    B = 0b0000_0010,
    SELECT = 0b0000_0100,
    START = 0b0000_1000,
    UP = 0b0001_0000,
    DOWN = 0b0010_0000,
    LEFT = 0b0100_0000,
    RIGHT = 0b1000_0000,
};

const JOYPAD_MAPPING = {
    up: Joypad.UP,
    left: Joypad.LEFT,
    down: Joypad.DOWN,
    right: Joypad.RIGHT,
    b: Joypad.B,
    a: Joypad.A,
    start: Joypad.START,
    select: Joypad.SELECT,
};

const DEFAULT_CONTROLS = {
    up: 'w',
    left: 'a',
    down: 's',
    right: 'd',
    b: 'k',
    a: 'l',
    start: 'Enter',
    select: ' ',
};

export const createControls = (controls: Record<ControlButton, string> = DEFAULT_CONTROLS) => {
    const reversed: Record<string, Joypad> = {};
    const serialize = (): string => JSON.stringify(controls);
    const update = (controls: Record<ControlButton, string>): void => {
        for (const [btn, key] of Object.entries(controls)) {
            controls[btn as ControlButton] = key;
            reversed[key] = JOYPAD_MAPPING[btn as ControlButton];
        }
    };

    update(controls);

    return {
        ref: controls,
        set(key: string, btn: ControlButton): void {
            delete reversed[controls[btn]];
            reversed[key] = JOYPAD_MAPPING[btn];
            controls[btn] = key;
        },
        get(key: string): Joypad {
            return reversed[key] ?? null;
        },
        isKeyMapped(key: string): boolean {
            return key in reversed;
        },
        serialize,
        update,
    };
};

export const createController = (nes: Nes, store: Store) => {
    let currentFrame = 0;
    let state = 0;
    let changed = false;
    const history: number[] = [];
    let isMetaDown = false;

    async function handleInput(event: KeyboardEvent, pressed: boolean): Promise<void> {
        if (event.key === 'Meta') {
            isMetaDown = pressed;
            event.preventDefault();
        }

        if (isMetaDown && pressed) {
            switch (event.key) {
                case 's': {
                    event.preventDefault();
                    hooks.call('saveState');
                    return;
                }
                case 'r': {
                    nes.softReset();
                    event.preventDefault();
                    return;
                }
                case 'l': {
                    event.preventDefault();
                    await hooks.call('loadLastSave');
                    return;
                }
                default:
                    event.preventDefault();
            }
        }

        if (store.ref.controls.isKeyMapped(event.key)) {
            if (event.key === 'Enter' || event.key === ' ') {
                event.preventDefault();
            }

            const prevState = state;
            const btn = store.ref.controls.get(event.key);

            if (pressed) {
                state |= btn;
            } else {
                state &= ~btn;
            }

            if (prevState !== state) {
                changed = true;
            }

            nes.setJoypad1(state);
        }
    }

    const onKeyDown = (event: KeyboardEvent) => handleInput(event, true);
    const onKeyUp = (event: KeyboardEvent) => handleInput(event, false);

    function tick(): void {
        currentFrame += 1;

        if (changed) {
            history.push(currentFrame, state);
            changed = false;
        }
    }

    function save(): void {
        if (history.length > 0) {
            // localStorage.setItem(localStorageKey, JSON.stringify(history));
        }
    }

    function updateNes(newNes: Nes): void {
        nes = newNes;
    }

    return {
        onKeyDown,
        onKeyUp,
        history,
        tick,
        save,
        updateNes,
    };
};
