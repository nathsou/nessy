import { Joypad } from "../../main";
import { Store } from "../store";
import { ControllerMapping } from "./ControllerMapping";
import { VMenu } from "./VMenu";

export const Controls = (store: Store) => {
    const ctrls = [
        Joypad.UP,
        Joypad.LEFT,
        Joypad.DOWN,
        Joypad.RIGHT,
        Joypad.A,
        Joypad.B,
        Joypad.START,
        Joypad.SELECT,
    ].map(btn => ControllerMapping(btn, store));

    const controlsList = VMenu(ctrls);

    const onKeyDown = (key: string) => {
        if (controlsList.state.activeIndex !== -1) {
            ctrls[controlsList.state.activeIndex].onKeyDown(key);
        }
    };

    return {
        ...controlsList,
        onKeyDown,
    };
};
