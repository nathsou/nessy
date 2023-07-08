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

export const createUI = (store: Store) => {
    let visible = { ref: true };
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

    window.addEventListener('keydown', event => {
        const activeMenuItem = menuItems[menu.state.activeIndex];

        if (event.key === 'Escape' || event.key === 'Tab') {
            visible.ref = !visible.ref;
            events.emit('uiToggled', { visible: visible.ref });
            event.preventDefault();
            event.stopPropagation();
        } else if (visible.ref) {
            const captured = subMenuMapping[activeMenuItem].onKeyDown(event.key);

            if (!captured) {
                switch (event.key) {
                    case 'ArrowLeft':
                        if (visible.ref) {
                            menu.prev();
                            subMenu.update(subMenuMapping[menuItems[menu.state.activeIndex]]);
                        }
                        break;
                    case 'ArrowRight':
                        if (visible.ref) {
                            menu.next();
                            subMenu.update(subMenuMapping[menuItems[menu.state.activeIndex]]);
                        }
                        break;
                    case 'ArrowDown':
                        if (visible.ref) {
                            subMenuMapping[activeMenuItem].next();
                        }
                        break;
                    case 'ArrowUp':
                        if (visible.ref) {
                            subMenuMapping[activeMenuItem].prev();
                        }
                        break;
                }
            }
        }
    });

    function render(imageData: ImageData): void {
        screen.clear();
        menu.render(0, 6, screen);
        subMenu.render(0, 9, screen);
        screen.render(imageData);
    }

    return { render, screen, visible };
};
