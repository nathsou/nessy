import { VList } from "./VList";
import { Component } from "./component";

type VMenuSettings = {
    visibleItems?: number,
    onSelect?(index: number): void,
};

export type VMenu = ReturnType<typeof VMenu>;
export const VMenu = <C extends Component<{ active: boolean }>>(
    items: C[],
    { visibleItems = items.length, onSelect }: VMenuSettings = {},
): Component<{ activeIndex: number, items: C[] }> & {
    next(): void,
    prev(): void,
    update(newItems: C[]): void,
} => {
    const state = { activeIndex: 0, items };
    const list = VList(items);
    items[0].state.active = true;

    const setActiveIndex = (index: number): void => {
        items[state.activeIndex].state.active = false;
        state.activeIndex = index;
        items[index].state.active = true;
        onSelect?.(index);
    };

    const updateVisibleWindow = (): void => {
        const visibleWindow: { firstIndex: number, lastIndex: number } = list.state;

        if (state.activeIndex < visibleWindow.firstIndex) {
            visibleWindow.firstIndex = Math.max(state.activeIndex, 0);
            visibleWindow.lastIndex = Math.min(visibleWindow.firstIndex + visibleItems - 1, items.length - 1);
        } if (state.activeIndex > visibleWindow.lastIndex) {
            visibleWindow.firstIndex = Math.max(state.activeIndex - visibleItems + 1, 0);
            visibleWindow.lastIndex = Math.min(state.activeIndex, items.length - 1);
        }
    };

    updateVisibleWindow();

    return {
        ...list,
        state,
        next(): void {
            setActiveIndex(Math.min(state.activeIndex + 1, items.length - 1));
            updateVisibleWindow();
        },
        prev(): void {
            setActiveIndex(Math.max(state.activeIndex - 1, 0));
            updateVisibleWindow();
        },
        update(newItems: C[]): void {
            list.update(newItems);
            items = newItems;
            state.items = newItems;
            state.activeIndex = 0;
            list.state.firstIndex = 0;
            list.state.lastIndex = Math.min(visibleItems, newItems.length) - 1;
        },
    };
};
