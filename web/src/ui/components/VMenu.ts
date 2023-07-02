import { VList } from "./VList";
import { Component } from "./component";

export type VMenu = ReturnType<typeof VMenu>;
export const VMenu = (items: Component<{ active: boolean }>[]): Component<{ activeIndex: number }> & {
    next(): void,
    prev(): void,
} => {
    const state = { activeIndex: -1 };

    const setActiveIndex = (index: number) => {
        if (state.activeIndex != -1) {
            items[state.activeIndex].state.active = false;
        }

        if (index != -1) {
            items[index].state.active = true;
        }

        state.activeIndex = index;
    };

    setActiveIndex(-1);

    return {
        ...VList(items, 1, 'start'),
        state,
        next(): void {
            setActiveIndex(Math.min(state.activeIndex + 1, items.length - 1));
        },
        prev(): void {
            setActiveIndex(Math.max(state.activeIndex - 1, -1));
        },
    };
};
