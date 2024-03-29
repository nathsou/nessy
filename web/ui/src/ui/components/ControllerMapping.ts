
import { ControlAction, Joypad } from "../../controls";
import { Screen } from "../screen";
import { Store } from "../store";
import { Text } from "./Text";

export const ControllerMapping = (button: Joypad, store: Store) => {
    let buttonName: ControlAction;
    let isListening = false;

    switch (button) {
        case Joypad.UP:
            buttonName = 'up';
            break;
        case Joypad.LEFT:
            buttonName = 'left';
            break;
        case Joypad.DOWN:
            buttonName = 'down';
            break;
        case Joypad.RIGHT:
            buttonName = 'right';
            break;
        case Joypad.A:
            buttonName = 'a';
            break;
        case Joypad.B:
            buttonName = 'b';
            break;
        case Joypad.START:
            buttonName = 'start';
            break;
        case Joypad.SELECT:
            buttonName = 'select';
            break;
    }

    const getText = () => {
        const btnName = `${Joypad[button].padEnd(6, ' ')}`;

        if (isListening) {
            return `${btnName} > ...`;
        }

        let keyName = store.ref.controls.ref.keyboard.inputs[buttonName];
        if (keyName === ' ') {
            keyName = 'space';
        }

        return `${btnName} > ${keyName.toUpperCase()}`;
    };

    const text = Text(getText());

    return {
        ...text,
        render(x: number, y: number, screen: Screen): void {
            text.update(getText());
            text.render(x, y, screen);
        },
        onKeyDown(key: string): boolean {
            if (isListening) {
                store.ref.controls.setKeyboard(key, buttonName);
                isListening = false;
                text.update(getText());
                return true;
            } else if (key === 'Enter') {
                isListening = true;
                return true;
            }

            return false;
        },
    };
};
