import { Component } from "./component";

export const HList = (
    items: Component[],
    spacing = 1,
    align: 'start' | 'center' | 'end' = 'start'
): Component => {
    let width = 0;
    let height = 0;

    for (const item of items) {
        width += item.width + spacing;
        height = Math.max(height, item.height);
    }

    return {
        state: {},
        width,
        height,
        render: (x, y, screen) => {
            let x0 = x;

            for (const item of items) {
                let y0: number;
                switch (align) {
                    case 'start':
                        y0 = 0;
                        break;
                    case 'center':
                        y0 = (height - item.height) / 2;
                        break;
                    case 'end':
                        y0 = height - item.height;
                        break;
                }

                item.render(x0, y0 + y, screen);
                x0 += item.width + spacing;
            }
        },
    };
};
