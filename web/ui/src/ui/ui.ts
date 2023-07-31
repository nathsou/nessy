import { Center } from "./components/Center";
import { Controls } from "./components/Controls";
import { HMenu } from "./components/HMenu";
import { Library } from "./components/Library";
import { Misc } from "./components/Misc";
import { Saves } from "./components/Saves";
import { Component, WIDTH } from "./components/component";
import { Text, drawText } from "./components/Text";
import { events } from "./events";
import { createScreen } from "./screen";
import { Store } from "./store";
import { hooks } from "./hooks";

type AlertType = 'info' | 'error';

type Alert = {
    text: string,
    type: AlertType,
    frames: number,
};

const ALERT_BACKGROUND = {
    info: 0x00,
    error: 0x06,
};

const ALERT_TEXT = {
    info: 0x30,
    error: 0x30,
};

export type Action = 'toggle_ui' | 'start' | 'select' | 'left' | 'right' | 'up' | 'down' | 'a' | 'b';

const UI_ACTION_MAPPING: Record<string, Action> = {
    // Escape: 'toggle_ui',
    // Tab: 'toggle_ui',
    ArrowLeft: 'left',
    ArrowRight: 'right',
    ArrowUp: 'up',
    ArrowDown: 'down',
    Enter: 'start',
    ' ': 'select',
};

export const createUI = (store: Store, width: number, height: number) => {
    const screen = createScreen(width, height);
    const alerts: Alert[] = [];
    const subMenuMapping: Record<string, Component<{ activeIndex: number }> & {
        prev(): void,
        next(): void,
        onKeyDown?(key: string): void,
        onGamePad?(id: number): void,
        onAction?(action: Action): boolean,
        setActive(isActive: boolean): void,
    }> = {
        library: Library(store),
        saves: Saves(store),
        controls: Controls(store),
        'misc.': Misc(store),
    };

    const menuItems = Object.keys(subMenuMapping);
    const menu = HMenu(menuItems.map(item => Text(item)), { initialIndex: 0, onSelect });
    const subMenu = Center(subMenuMapping[menuItems[menu.state.activeIndex]]);
    let previousActiveIndex = menu.state.activeIndex;

    function onSelect(index: number): void {
        subMenuMapping[menuItems[previousActiveIndex]].setActive(false);
        subMenuMapping[menuItems[index]].setActive(true);
        previousActiveIndex = index;
    }

    events.on('uiToggled', ({ visible }) => {
        subMenuMapping[menuItems[menu.state.activeIndex]].setActive(visible);
        alerts.length = 0;
    });

    const ret = { render, screen, onAction, onKeyDown, onGamePad, alert, visible: true };

    function onAction(action: Action): boolean {
        const activeMenuItem = menuItems[menu.state.activeIndex];

        if (action === 'toggle_ui') {
            hooks.call('toggleUI');
            return true;
        } else if (ret.visible) {
            const captured = subMenuMapping[activeMenuItem].onAction?.(action) ?? false;

            if (captured) {
                return true;
            }

            switch (action) {
                case 'left':
                    if (ret.visible) {
                        menu.prev();
                        subMenu.update(subMenuMapping[menuItems[menu.state.activeIndex]]);
                        return true;
                    }
                    break;
                case 'right':
                    if (ret.visible) {
                        menu.next();
                        subMenu.update(subMenuMapping[menuItems[menu.state.activeIndex]]);
                        return true;
                    }
                    break;
                case 'down':
                    if (ret.visible) {
                        subMenuMapping[activeMenuItem].next();
                        return true;
                    }
                    break;
                case 'up':
                    if (ret.visible) {
                        subMenuMapping[activeMenuItem].prev();
                        return true;
                    }
                    break;
            }
        }

        return false;
    }

    function onGamePad(id: number): void {
        if (ret.visible) {
            const activeMenuItem = menuItems[menu.state.activeIndex];
            subMenuMapping[activeMenuItem].onGamePad?.(id);
        }
    }

    function onKeyDown(key: KeyboardEvent['key']): boolean {
        if (ret.visible) {
            const captured = subMenuMapping[menuItems[menu.state.activeIndex]].onKeyDown?.(key);

            if (captured) {
                return true;
            }

            const action = UI_ACTION_MAPPING[key];

            if (action) {
                return onAction(action);
            }
        }

        return false;
    }

    function renderAlerts(): void {
        if (alerts.length > 0) {
            const oldestAlert = alerts[0];

            if (oldestAlert.frames > 0) {
                const textColor = ALERT_TEXT[oldestAlert.type];
                const bgColor = ALERT_BACKGROUND[oldestAlert.type];
                const clippedText = oldestAlert.text.slice(0, WIDTH);
                drawText(
                    WIDTH - clippedText.length,
                    2,
                    clippedText,
                    screen,
                    textColor,
                    bgColor
                );
                oldestAlert.frames -= 1;
            } else {
                alerts.shift();
            }
        }
    }

    function render(buffer: Uint8Array): void {
        screen.clear();
        menu.render(0, 6, screen);
        subMenu.render(0, 9, screen);
        renderAlerts();
        screen.render(buffer);
    }

    function alert(info: Alert): void {
        alerts.push(info);
    }

    return ret;
};
