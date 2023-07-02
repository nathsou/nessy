import { Component } from "./component";

export const VList = (
    items: Component[],
    spacing = 1,
    align: 'start' | 'center' | 'end' = 'start'
): Component => {
    let width = 0;
    let height = 0;

    for (const item of items) {
        width = Math.max(width, item.width);
        height += item.height + spacing;
    }

    return {
        state: {},
        width,
        height,
        render: (x, y, screen) => {
            let y0 = y;

            for (const item of items) {
                let x0: number;
                switch (align) {
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
                y0 += item.height + spacing;
            }
        },
    };
};
