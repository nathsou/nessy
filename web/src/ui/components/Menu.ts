import { Component, WIDTH } from "./component";
import { drawText } from "./text";

export const Menu = (
    items: string[],
    initialIndex = 0,
    onItemChange?: (item: string, itemIndex: number) => void
): Component<{ active: boolean }> & {
    next(): void,
    prev(): void,
} => {
    const state = { active: true };
    let activeIndex = initialIndex;
    // compute the x offset for each item so that they are centered
    // on the screen
    const xOffsets: number[] = [];
    let maxLength = 0;
    let x0 = 0;

    for (let i = 0; i < items.length; i++) {
        const item = items[i];
        xOffsets.push(Math.round(x0 + item.length / 2));
        x0 += item.length + 1;
        maxLength = Math.max(maxLength, item.length);
    }

    const setActiveIndex = (index: number): void => {
        if (state.active && index !== activeIndex) {
            activeIndex = index;
            onItemChange?.(items[activeIndex], activeIndex);
        }
    };

    return {
        state,
        width: WIDTH,
        height: 1,
        render(_x, y, screen): void {
            let x0 = WIDTH / 2 - xOffsets[activeIndex];

            for (let i = 0; i < items.length; i++) {
                const item = items[i];
                let textColor = 0x3F;
                let bgColor = 0x30;

                if (state.active && i === activeIndex) {
                    textColor = 0x30;
                    bgColor = 0x3F;
                }

                drawText(x0, y, item, screen, textColor, bgColor);
                x0 += item.length + 1;
            }

            if (activeIndex > 0) {
                drawText(0, y, '< ', screen);
            }

            if (activeIndex < items.length - 1) {
                drawText(WIDTH - 2, y, ' >', screen);
            }
        },
        next(): void {
            setActiveIndex(Math.min(activeIndex + 1, items.length - 1));
        },
        prev(): void {
            setActiveIndex(Math.max(activeIndex - 1, 0));
        },
    };
};
