import { Component } from "./component";

type VListSettings = {
    spacing: number,
    align: 'start' | 'center' | 'end',
    firstIndex: number,
    lastIndex: number,
};

export const VList = (
    items: Component[],
    settings: VListSettings = {
        spacing: 1,
        align: 'start',
        firstIndex: 0,
        lastIndex: items.length - 1,
    },
): Component<VListSettings> & { update: (newItems: Component[]) => void } => {
    const state = { ...settings };
    let width = 0;
    let height = 0;

    for (const item of items) {
        width = Math.max(width, item.width);
        height += item.height + state.spacing;
    }

    return {
        state,
        width,
        height,
        render(x, y, screen): void {
            let y0 = y;

            for (let i = state.firstIndex; i <= state.lastIndex; i++) {
                const item = items[i];
                let x0: number;
                switch (state.align) {
                    case 'start':
                        x0 = 0;
                        break;
                    case 'center':
                        x0 = (width - item.width) / 2;
                        break;
                    case 'end':
                        x0 = width - item.width;
                        break;
                }

                item.render(x0 + x, y0, screen);
                y0 += item.height + state.spacing;
            }
        },
        update(newItems: Component[]): void {
            items = newItems;
            state.firstIndex = 0;
            state.lastIndex = items.length - 1;
        },
    };
};
