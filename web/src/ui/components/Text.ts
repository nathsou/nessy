import { getCharacterTile } from "../font";
import { Screen } from "../screen";
import { Component } from "./component";

export function drawText(
    x: number,
    y: number,
    text: string,
    screen: Screen,
    textColor = 0x3F,
    bgColor = 0x30,
) {
    for (let i = 0; i < text.length; i++) {
        screen.setTile(x + i, y, getCharacterTile(text[i], textColor, bgColor));
    }
}

export const Text = (text: string, textColor = 0x3F, bgColor = 0x30): Component<{ active: boolean }> => {
    const state = { active: false };

    return {
        state,
        width: text.length,
        height: 1,
        render: (x, y, screen) => {
            if (state.active) {
                drawText(x, y, text, screen, bgColor, textColor);
            } else {
                drawText(x, y, text, screen, textColor, bgColor);
            }
        },
    };
};
