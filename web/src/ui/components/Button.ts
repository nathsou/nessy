import { Component } from "./component";

export const Button = (component: Component<{ active: boolean }>, onEnter: () => void) => {
    return {
        ...component,
        enter(): void {
            onEnter();
        },
    };
};
