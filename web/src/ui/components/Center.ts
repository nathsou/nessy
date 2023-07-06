import { Component, WIDTH } from "./component";

export const Center = (comp: Component) => {
    let dx = 0;

    function update(newComp: Component) {
        comp = newComp;
        dx = Math.floor((WIDTH - newComp.width) / 2);
        ret.height = newComp.height;
    }

    const ret: Component & {
        update(newComp: Component): void,
    } = {
        state: {},
        width: WIDTH,
        height: comp.height,
        render(x, y, screen) {
            comp.render(x + dx, y, screen);
        },
        update,
    };

    update(comp);

    return ret;
};
