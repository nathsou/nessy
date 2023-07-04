import { getCharacterTile } from "../font";
import { Screen } from "../screen";
import { Component } from "./component";

export function drawText(
    x: number,
    y: number,
    text: string,
    screen: Screen,
    textColor = 0x30,
    bgColor = 0x00,
) {
    for (let i = 0; i < text.length; i++) {
        screen.setTile(x + i, y, getCharacterTile(text[i], textColor, bgColor));
    }
};

export const Text = (text: string, textColor = 0x30, bgColor = 0x00) => {
    const state = { active: false };
    const ret: Component<{ active: boolean }> & { update(newText: string): void } = {
        state,
        width: text.length,
        height: 1,
        update(newText: string): void {
            if (newText !== text) {
                ret.width = newText.length;
                text = newText;
            }
        },
        render: (x, y, screen) => {
            if (state.active) {
                drawText(x, y, text, screen, bgColor, textColor);
            } else {
                drawText(x, y, text, screen, textColor, bgColor);
            }
        },
    };

    return ret;
};
