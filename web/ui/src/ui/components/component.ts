import { Screen } from "../screen";

export const WIDTH = 32;
export const HEIGHT = 30;

export type Component<T extends {} = {}> = {
    state: T,
    width: number,
    height: number,
    render: (
        x: number,
        y: number,
        screen: Screen,
    ) => void,
};
