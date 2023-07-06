import { VList } from "./VList";
import { Component } from "./component";

export type VMenu = ReturnType<typeof VMenu>;
export const VMenu = (items: Component<{ active: boolean }>[], visibleItems = items.length): Component<{ activeIndex: number }> & {
    next(): void,
    prev(): void,
    update(newItems: Component<{ active: boolean }>[]): void,
} => {
    const state = { activeIndex: -1 };
    const list = VList(items, {
        spacing: 1,
        align: 'start',
        firstIndex: 0,
        lastIndex: visibleItems - 1,
    });

    const setActiveIndex = (index: number): void => {
        if (state.activeIndex !== -1) {
            items[state.activeIndex].state.active = false;
        }

        if (index !== -1) {
            items[index].state.active = true;
        }

        state.activeIndex = index;
    };

    const visibleWindow = {
        firstIndex: 0,
        lastIndex: visibleItems - 1,
    };

    const updateVisibleWindow = (): void => {
        if (state.activeIndex < visibleWindow.firstIndex) {
            visibleWindow.firstIndex = Math.max(state.activeIndex, 0);
            visibleWindow.lastIndex = Math.min(visibleWindow.firstIndex + visibleItems - 1, items.length - 1);
        } else if (state.activeIndex > visibleWindow.lastIndex) {
            visibleWindow.firstIndex = Math.max(state.activeIndex - visibleItems + 1, 0);
            visibleWindow.lastIndex = Math.min(state.activeIndex, items.length - 1);
        }

        list.state.firstIndex = visibleWindow.firstIndex;
        list.state.lastIndex = visibleWindow.lastIndex;
    };

    setActiveIndex(-1);
    updateVisibleWindow();

    return {
        ...list,
        state,
        next(): void {
            setActiveIndex(Math.min(state.activeIndex + 1, items.length - 1));
            updateVisibleWindow();
        },
        prev(): void {
            setActiveIndex(Math.max(state.activeIndex - 1, -1));
            updateVisibleWindow();
        },
        update(newItems: Component<{ active: boolean }>[]): void {
            setActiveIndex(-1);
            list.update(newItems);
            items = newItems;
            updateVisibleWindow();
        },
    };
};
