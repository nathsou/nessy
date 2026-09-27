import { hooks } from "./ui/hooks";
import { Store } from "./ui/store";

export type ControlAction = 'up' | 'left' | 'down' | 'right' | 'b' | 'a' | 'start' | 'select';
export type MetaAction = 'toggleUI' | 'save' | 'loadLastSave';

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

type Controls = {
    keyboard: { inputs: Record<ControlAction, string>, meta: Record<MetaAction, string> },
    gamepad: { inputs: Record<ControlAction, number>, meta: Record<MetaAction, number> },
};

const JOYPAD_MAPPING: Record<ControlAction, Joypad> = {
    up: Joypad.UP,
    left: Joypad.LEFT,
    down: Joypad.DOWN,
    right: Joypad.RIGHT,
    b: Joypad.B,
    a: Joypad.A,
    start: Joypad.START,
    select: Joypad.SELECT,
};

const REVERSE_JOYPAD_MAPPING: Record<Joypad, ControlAction> = {
    [Joypad.UP]: 'up',
    [Joypad.LEFT]: 'left',
    [Joypad.DOWN]: 'down',
    [Joypad.RIGHT]: 'right',
    [Joypad.B]: 'b',
    [Joypad.A]: 'a',
    [Joypad.START]: 'start',
    [Joypad.SELECT]: 'select',
};

const DEFAULT_KEYBOARD_CONTROLS: Controls['keyboard'] = {
    inputs: {
        up: 'w',
        left: 'a',
        down: 's',
        right: 'd',
        b: 'k',
        a: 'l',
        start: 'Enter',
        select: ' ',
    },
    meta: {
        toggleUI: 'Escape',
        save: 'META+s',
        loadLastSave: 'META+l',
    },
};


// https://w3c.github.io/gamepad/#remapping
const GAMEPAD_MAPPING = {
    0: { type: 'button', index: 0 }, // Bottom button in right cluster
    1: { type: 'button', index: 1 }, // Right button in right cluster
    2: { type: 'button', index: 2 }, // Left button in right cluster
    3: { type: 'button', index: 3 }, // Top button in right cluster
    4: { type: 'button', index: 4 }, // Top left front button
    5: { type: 'button', index: 5 }, // Top right front button
    6: { type: 'button', index: 6 }, // Bottom left front button
    7: { type: 'button', index: 7 }, // Bottom right front button
    8: { type: 'button', index: 8 }, // Left button in center cluster
    9: { type: 'button', index: 9 }, // Right button in center cluster
    10: { type: 'button', index: 10 }, // Left button in left cluster
    11: { type: 'button', index: 11 }, // Right button in left cluster
    12: { type: 'button', index: 12 }, // Top button in left cluster
    13: { type: 'button', index: 13 }, // Bottom button in left cluster
    14: { type: 'button', index: 14 }, // Left button in right cluster
    15: { type: 'button', index: 15 }, // Right button in right cluster
    16: { type: 'button', index: 16 }, // Center button in right cluster

    17: { type: 'axis', index: 0, dir: -1 }, // Horizontal axis for left stick (negative left/positive right)
    18: { type: 'axis', index: 0, dir: 1 },
    19: { type: 'axis', index: 1, dir: -1 }, // Vertical axis for left stick (negative up/positive down)
    20: { type: 'axis', index: 1, dir: 1 },
    21: { type: 'axis', index: 2, dir: -1 }, // Horizontal axis for right stick (negative left/positive right)
    22: { type: 'axis', index: 2, dir: 1 },
    23: { type: 'axis', index: 3, dir: -1 }, // Vertical axis for right stick (negative up/positive down)
    24: { type: 'axis', index: 3, dir: 1 },
} satisfies Record<number, GamePadInput>;

type GamePadInput = { type: 'button', index: number } | { type: 'axis', index: number, dir: number };

const DEFAULT_GAMEPAD_CONTROLS: Controls['gamepad'] = {
    inputs: {
        up: 12,
        left: 14,
        down: 13,
        right: 15,
        b: 0,
        a: 1,
        start: 9,
        select: 8,
    },
    meta: {
        toggleUI: 6,
        save: 10,
        loadLastSave: 11,
    },
};

const DEFAULT_CONTROLS: Controls = {
    keyboard: DEFAULT_KEYBOARD_CONTROLS,
    gamepad: DEFAULT_GAMEPAD_CONTROLS,
};

export const createControls = (initial: Controls = DEFAULT_CONTROLS) => {
    const controls: Controls = JSON.parse(JSON.stringify(initial));
    const reversed: Record<'keyboard' | 'gamepad', Record<'inputs' | 'meta', Record<string, Joypad>>> = {
        keyboard: { inputs: {}, meta: {} },
        gamepad: { inputs: {}, meta: {} },
    };

    const serialize = (): string => JSON.stringify(controls);
    const update = (next: Controls): void => {
        const copy: Controls = JSON.parse(JSON.stringify(next));
        controls.keyboard = copy.keyboard;
        controls.gamepad = copy.gamepad;
        for (const device of ['keyboard', 'gamepad'] as const) {
            for (const key of Object.keys(reversed[device].inputs)) delete reversed[device].inputs[key];
            for (const [button, input] of Object.entries(controls[device].inputs)) {
                reversed[device].inputs[input] = JOYPAD_MAPPING[button as ControlAction];
            }
        }
    };

    update(controls);

    return {
        ref: controls,
        reversed,
        setKeyboard(key: string, btn: ControlAction): void {
            delete reversed.keyboard.inputs[controls.keyboard.inputs[btn]];
            reversed.keyboard.inputs[key] = JOYPAD_MAPPING[btn];
            controls.keyboard.inputs[btn] = key;
        },
        setGamepad(input: number, btn: ControlAction): void {
            delete reversed.gamepad.inputs[controls.gamepad.inputs[btn]];
            reversed.gamepad.inputs[input] = JOYPAD_MAPPING[btn];
            controls.gamepad.inputs[btn] = input;
        },
        getKeyboard(key: string): Joypad | null {
            return reversed.keyboard.inputs[key] ?? null;
        },
        getGamepad(input: number): Joypad | null {
            return reversed.gamepad.inputs[input] ?? null;
        },
        isKeyMapped(key: string): boolean {
            return key in reversed.keyboard.inputs;
        },
        isInputMapped(input: number): boolean {
            return input in reversed.gamepad.inputs;
        },
        serialize,
        update,
    };
};

export const createController = (store: Store) => {
    let gamepadIndex = -1;
    let keyboardState = 0;
    let gamepadState = 0;

    const previousState: { inputs: Record<ControlAction, boolean>, meta: Record<MetaAction, boolean> } = {
        inputs: {
            up: false,
            left: false,
            down: false,
            right: false,
            b: false,
            a: false,
            start: false,
            select: false,
        },
        meta: {
            toggleUI: false,
            save: false,
            loadLastSave: false,
        },
    };

    async function handleInput(event: KeyboardEvent, pressed: boolean): Promise<void> {
        if (!pressed) {
            previousState.meta.save=false;
            previousState.meta.loadLastSave=false;
        }
        if (pressed && (event.key === 'Escape' || event.key === 'Tab')) {
            event.preventDefault();
            if (!previousState.meta.toggleUI) {
                hooks.call('toggleUI');
                previousState.meta.toggleUI = true;
            }
        } else {
            previousState.meta.toggleUI = false;
        }

        if ((event.metaKey || event.ctrlKey) && pressed) {
            switch (event.key) {
                case 's': {
                    event.preventDefault();
                    if (!previousState.meta.save) {
                        hooks.call('saveState');
                        previousState.meta.save = true;
                        return;
                    }
                    break;
                }
                case 'r': {
                    event.preventDefault();
                    hooks.call('softReset');
                    return;
                }
                case 'l': {
                    event.preventDefault();
                    if (!previousState.meta.loadLastSave) {
                        await hooks.call('loadLastSave');
                        previousState.meta.loadLastSave = true;
                        return;
                    }
                    break;
                }
                default:
                    previousState.meta.save = false;
                    previousState.meta.loadLastSave = false;
                    event.preventDefault();
            }
        }

        if (store.ref.controls.isKeyMapped(event.key)) {
            if (event.key === 'Enter' || event.key === ' ') {
                event.preventDefault();
            }

            const btn = store.ref.controls.getKeyboard(event.key)!;
            const action = REVERSE_JOYPAD_MAPPING[btn];

            if (pressed) {
                keyboardState |= btn;
            } else {
                keyboardState &= ~btn;
            }

            if (previousState.inputs[action] !== pressed) {
                if (pressed) hooks.call('input', action);
                previousState.inputs[action] = pressed;
            }
        }
    }

    const onKeyDown = (event: KeyboardEvent) => handleInput(event, true);
    const onKeyUp = (event: KeyboardEvent) => handleInput(event, false);

    function tickGamepad(): void {
        const gamepad = gamepadIndex >= 0 ? navigator.getGamepads()[gamepadIndex] : null;
        if (!gamepad) { gamepadState = 0; return; }
        let nextState = 0;
        for (const [action, index] of Object.entries(store.ref.controls.ref.gamepad.inputs)) {
            const input = GAMEPAD_MAPPING[index as keyof typeof GAMEPAD_MAPPING] as GamePadInput | undefined;
            if (!input) continue;
            const pressed = input.type === 'button'
                ? (gamepad.buttons[input.index]?.pressed ?? false)
                : (gamepad.axes[input.index] ?? 0) * input.dir > 0.5;
            const button = JOYPAD_MAPPING[action as ControlAction];
            if (pressed) {
                nextState |= button;
                if (!(gamepadState & button)) hooks.call('input', action as ControlAction, index);
            }
        }
        gamepadState = nextState;
        for (const action of ['toggleUI', 'save', 'loadLastSave'] as const) {
            const index = store.ref.controls.ref.gamepad.meta[action];
            const pressed = gamepad.buttons[index]?.pressed ?? false;
            if (pressed && !previousState.meta[action]) {
                if (action === 'save') hooks.call('saveState');
                else if (action === 'toggleUI') hooks.call('toggleUI');
                else hooks.call('loadLastSave');
            }
            previousState.meta[action] = pressed;
        }
    }

    function releaseAll(): void {
        keyboardState = 0; gamepadState = 0;
        for (const action of Object.keys(previousState.inputs) as ControlAction[]) previousState.inputs[action] = false;
        for (const action of Object.keys(previousState.meta) as MetaAction[]) previousState.meta[action] = false;
        hooks.call('setJoypad1', 0);
    }

    function tick(): void {
        tickGamepad();
        const state = keyboardState | gamepadState;
        hooks.call('setJoypad1', state);
    }

    function onGamepadConnected(event: GamepadEvent): void {
        gamepadIndex = event.gamepad.index;
    }

    function onGamepadDisconnected(event: GamepadEvent): void {
        if (event.gamepad.index === gamepadIndex) {
            gamepadIndex = -1;
            gamepadState = 0;
        }
    }

    return {
        releaseAll,
        onKeyDown,
        onKeyUp,
        onGamepadConnected,
        onGamepadDisconnected,
        tick,
    };
};
