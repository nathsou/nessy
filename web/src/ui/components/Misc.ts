import { Store } from "../store";
import { Select } from "./Select";
import { VMenu } from "./VMenu";

const ScalingFactor = (store: Store) => {
    const OPTION_MAPPING = {
        '1x': 1,
        '2x': 2,
        '3x': 3,
        '4x': 4,
    } as const;

    const REVERSE_MAPPING = {
        1: '1x',
        2: '2x',
        3: '3x',
        4: '4x',
    } as const;

    return Select({
        name: 'Zoom',
        options: ['1x', '2x', '3x', '4x'],
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

export const Misc = (store: Store) => {
    const list = VMenu([
        ScalingFactor(store),
        ScalingMode(store),
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
