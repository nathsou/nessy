import { hooks } from "../hooks";
import { Store } from "../store";
import { Action } from "../ui";
import { Button } from "./Button";
import { Select } from "./Select";
import { Text } from "./Text";
import { VMenu } from "./VMenu";

const ScalingFactor = (store: Store) => {
    const OPTION_MAPPING = {
        '1x': 1,
        '2x': 2,
        '3x': 3,
        '4x': 4,
        'max': 50,
    } as const;

    const REVERSE_MAPPING = {
        1: '1x',
        2: '2x',
        3: '3x',
        4: '4x',
        50: 'max',
    } as const;

    return Select({
        name: 'Zoom',
        options: ['1x', '2x', '3x', '4x', 'max'],
        initialOption: REVERSE_MAPPING[store.ref.scalingFactor],
        onChange: option => store.set('scalingFactor', OPTION_MAPPING[option]),
    });
};

const ScalingMode = (store: Store) => {
    return Select({
        name: 'Rendering',
        options: ['pixelated', 'blurry'],
        initialOption: store.ref.scalingMode,
        onChange: option => store.set('scalingMode', option),
    });
};

const FullScreen = () => ({
    ...Button(
        Text('Toggle Fullscreen'),
        () => hooks.call('toggleFullscreen'),
    ),
    onAction(action: Action) {
        if (action === 'start' || action === 'a') {
            this.enter();
            return true;
        }

        return false;
    },
});

const SoftReset = () => ({
    ...Button(
        Text('Soft Reset (CTRL+R)'),
        () => {
            hooks.call('softReset');
            hooks.call('toggleUI', false);
        },
    ),
    onAction(action: Action) {
        if (action === 'start' || action === 'a') {
            this.enter();
            return true;
        }

        return false;
    },
});

export const Misc = (store: Store) => {
    const list = VMenu([
        ScalingFactor(store),
        ScalingMode(store),
        FullScreen(),
        SoftReset(),
    ]);

    const onAction = (action: Action): boolean => {
        return list.state.items[list.state.activeIndex].onAction(action);
    };

    const setActive = (_isActive: boolean) => { };

    return {
        ...list,
        onAction,
        setActive,
    };
};
