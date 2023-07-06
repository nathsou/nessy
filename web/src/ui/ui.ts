import { Center } from "./components/Center";
import { Controls } from "./components/Controls";
import { HMenu } from "./components/HMenu";
import { Library } from "./components/Library";
import { Misc } from "./components/Misc";
import { Saves } from "./components/Saves";
import { Component } from "./components/component";
import { Text } from "./components/text";
import { createScreen } from "./screen";
import { Store } from "./store";

export const createUI = (store: Store) => {
    let showUI = { ref: store.ref.rom == null };
    const screen = createScreen();
    const menuItems = [
        'library',
        'controls',
        'saves',
        'misc.',
    ];

    const menu = HMenu(menuItems.map(item => Text(item)), 0);
    const subMenuMapping: Record<string, Component<{ activeIndex: number }> & {
        prev(): void,
        next(): void,
        onKeyDown(key: string): void
    }> = {
        library: Library(store),
        controls: Controls(store),
        saves: Saves(store),
        'misc.': Misc(store),
    };

    const subMenu = Center(subMenuMapping[menuItems[menu.state.activeIndex]]);

    window.addEventListener('keydown', event => {
        const activeMenuItem = menuItems[menu.state.activeIndex];

        switch (event.key) {
            case 'Escape':
                if (store.ref.rom != null) {
                    showUI.ref = !showUI.ref;
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

        if (showUI.ref) {
            // menu.state.active = subMenuMapping[activeMenuItem].state.activeIndex === -1;
        }
    });

    function render(imageData: ImageData): void {
        screen.clear();
        menu.render(0, 6, screen);
        subMenu.render(0, 9, screen);
        screen.render(imageData);
    }

    return { render, showUI };
};
