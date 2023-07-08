import { hooks } from "../hooks";
import { Store } from "../store";
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
    onKeyDown(key: string) {
        if (key === 'Enter') {
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
    ]);

    const onKeyDown = (key: string): boolean => {
        return list.state.items[list.state.activeIndex].onKeyDown(key);
    };

    const setActive = (_isActive: boolean) => { };

    return {
        ...list,
        onKeyDown,
        setActive,
    };
};
