import { Center } from "./components/Center";
import { Controls } from "./components/Controls";
import { HMenu } from "./components/HMenu";
import { Library } from "./components/Library";
import { Misc } from "./components/Misc";
import { Saves } from "./components/Saves";
import { Component } from "./components/component";
import { Text } from "./components/Text";
import { events } from "./events";
import { createScreen } from "./screen";
import { Store } from "./store";
import { hooks } from "./hooks";

export const createUI = (store: Store) => {
    const screen = createScreen();
    const subMenuMapping: Record<string, Component<{ activeIndex: number }> & {
        prev(): void,
        next(): void,
        onKeyDown(key: string): boolean,
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
    });

    const ret = { render, screen, onKeyDown, visible: true };

    function onKeyDown(key: string): boolean {
        const activeMenuItem = menuItems[menu.state.activeIndex];

        if (key === 'Escape' || key === 'Tab') {
            hooks.call('toggleUI');
        } else if (ret.visible) {
            const captured = subMenuMapping[activeMenuItem].onKeyDown(key);

            if (captured) {
                return true;
            }

            switch (key) {
                case 'ArrowLeft':
                    if (ret.visible) {
                        menu.prev();
                        subMenu.update(subMenuMapping[menuItems[menu.state.activeIndex]]);
                        return true;
                    }
                    break;
                case 'ArrowRight':
                    if (ret.visible) {
                        menu.next();
                        subMenu.update(subMenuMapping[menuItems[menu.state.activeIndex]]);
                        return true;
                    }
                    break;
                case 'ArrowDown':
                    if (ret.visible) {
                        subMenuMapping[activeMenuItem].next();
                        return true;
                    }
                    break;
                case 'ArrowUp':
                    if (ret.visible) {
                        subMenuMapping[activeMenuItem].prev();
                        return true;
                    }
                    break;
            }
        }

        return false;
    }

    function render(buffer: Uint8Array): void {
        screen.clear();
        menu.render(0, 6, screen);
        subMenu.render(0, 9, screen);
        screen.render(buffer);
    }

    return ret;
};
