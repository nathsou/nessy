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

export type TextSettings = {
    textColor: number,
    bgColor: number,
    maxLength: number,
};

const DEFAULT_SETTINGS: TextSettings = {
    textColor: 0x30,
    bgColor: 0x00,
    maxLength: Infinity,
};

const textWithEllipsis = (text: string, maxLength: number) => {
    if (text.length > maxLength) {
        return text.slice(0, maxLength - 3) + '...';
    } else {
        return text;
    }
};

export const Text = (text: string, options: Partial<TextSettings> = DEFAULT_SETTINGS) => {
    const state = { active: false };
    const settings = { ...DEFAULT_SETTINGS, ...options };
    text = textWithEllipsis(text, settings.maxLength);

    const ret: Component<{ active: boolean }> & { update(newText: string): void } = {
        state,
        width: text.length,
        height: 1,
        update(newText: string): void {
            if (newText !== text) {
                ret.width = newText.length;
                text = textWithEllipsis(newText, settings.maxLength ?? Infinity);
            }
        },
        render: (x, y, screen) => {
            if (state.active) {
                drawText(x, y, text, screen, settings.bgColor, settings.textColor);
            } else {
                drawText(x, y, text, screen, settings.textColor, settings.bgColor);
            }
        },
    };

    return ret;
};
