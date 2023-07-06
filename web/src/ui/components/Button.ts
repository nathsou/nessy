import { Component } from "./component";

export type Button = ReturnType<typeof Button>;
export const Button = (component: Component<{ active: boolean }>, onEnter: () => void) => {
    return {
        ...component,
        enter(): void {
            onEnter();
        },
    };
};
