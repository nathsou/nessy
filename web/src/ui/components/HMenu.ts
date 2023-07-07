import { Component, WIDTH } from "./component";
import { drawText } from "./Text";

export type HMenuSettings = {
    initialIndex?: number,
    onSelect?: (index: number) => void,
};

export const HMenu = (
    items: Component<{ active: boolean }>[],
    { initialIndex = 0, onSelect }: HMenuSettings = {},
): Component<{ active: boolean, activeIndex: number }> & {
    next(): void,
    prev(): void,
} => {
    const state = { active: true, activeIndex: initialIndex };
    items[initialIndex].state.active = true;
    // compute the x offset for each item so that they are centered
    // on the screen
    const xOffsets: number[] = [];
    let maxLength = 0;
    let x0 = 0;

    for (let i = 0; i < items.length; i++) {
        const item = items[i];
        xOffsets.push(Math.round(x0 + item.width / 2));
        x0 += item.width + 1;
        maxLength = Math.max(maxLength, item.width);
    }

    const setActiveIndex = (index: number): void => {
        if (state.active && index !== state.activeIndex) {
            items[state.activeIndex].state.active = false;
            items[index].state.active = true;
            state.activeIndex = index;
            onSelect?.(index);
        }
    };

    setActiveIndex(initialIndex);

    return {
        state,
        width: WIDTH,
        height: 1,
        render(_x, y, screen): void {
            let x0 = WIDTH / 2 - xOffsets[state.activeIndex];

            for (let i = 0; i < items.length; i++) {
                const item = items[i];
                item.render(x0, y, screen);
                x0 += item.width + 1;
            }

            if (state.activeIndex > 0) {
                drawText(0, y, '< ', screen);
            }

            if (state.activeIndex < items.length - 1) {
                drawText(WIDTH - 2, y, ' >', screen);
            }
        },
        next(): void {
            setActiveIndex(Math.min(state.activeIndex + 1, items.length - 1));
        },
        prev(): void {
            setActiveIndex(Math.max(state.activeIndex - 1, 0));
        },
    };
};
