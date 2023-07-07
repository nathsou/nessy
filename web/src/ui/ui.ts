import { Center } from "./components/Center";
import { Controls } from "./components/Controls";
import { HMenu } from "./components/HMenu";
import { Library } from "./components/Library";
import { Misc } from "./components/Misc";
import { Saves } from "./components/Saves";
import { Component } from "./components/component";
import { Text } from "./components/text";
import { events } from "./events";
import { createScreen } from "./screen";
import { Store } from "./store";

export const createUI = (store: Store) => {
    let showUI = { ref: true };
    const screen = createScreen();
    const subMenuMapping: Record<string, Component<{ activeIndex: number }> & {
        prev(): void,
        next(): void,
        onKeyDown(key: string): void,
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

    window.addEventListener('keydown', event => {
        const activeMenuItem = menuItems[menu.state.activeIndex];

        switch (event.key) {
            case 'Escape':
                if (store.ref.rom != null) {
                    showUI.ref = !showUI.ref;
                    events.emit('uiToggled', { visible: showUI.ref });
                    event.preventDefault();
                }
                break;
            case 'ArrowLeft':
                if (showUI.ref) {
                    menu.prev();
                    subMenu.update(subMenuMapping[menuItems[menu.state.activeIndex]]);
                }
                break;
            case 'ArrowRight':
                if (showUI.ref) {
                    menu.next();
                    subMenu.update(subMenuMapping[menuItems[menu.state.activeIndex]]);
                }
                break;
            case 'ArrowDown':
                if (showUI.ref) {
                    subMenuMapping[activeMenuItem].next();
                }
                break;
            case 'ArrowUp':
                if (showUI.ref) {
                    subMenuMapping[activeMenuItem].prev();
                }
                break;
            default: {
                if (showUI.ref) {
                    const menuItem = subMenuMapping[activeMenuItem];
                    menuItem.onKeyDown(event.key);
                }
                break;
            }
        }
    });

    function render(imageData: ImageData): void {
        screen.clear();
        menu.render(0, 6, screen);
        subMenu.render(0, 9, screen);
        screen.render(imageData);
    }

    return { render, screen, showUI };
};
