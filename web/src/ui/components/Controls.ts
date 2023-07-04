import { Joypad } from "../../main";
import { ControllerMapping } from "./ControllerMapping";
import { VMenu } from "./VMenu";

export const Controls = () => {
    const ctrls = [
        Joypad.UP,
        Joypad.LEFT,
        Joypad.DOWN,
        Joypad.RIGHT,
        Joypad.A,
        Joypad.B,
        Joypad.START,
        Joypad.SELECT,
    ].map(ControllerMapping);

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
