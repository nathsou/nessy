import { Menu } from "./components/Menu";
import { VMenu } from "./components/VMenu";
import { Text } from "./components/text";
import { createScreen } from "./screen";
import { store } from "./store";

export const createUI = () => {
    let showUI = { ref: true };
    const screen = createScreen();
    const menuItems = [
        'library',
        'controls',
        'rendering',
        'save',
    ];

    let activeMenuItem = menuItems[1];

    const menu = Menu(menuItems, 1, item => { activeMenuItem = item; });

    const libraryList = VMenu([
        Text('Library'),
    ]);

    const controlsList = VMenu([
        Text(`UP     -> ${store.controls.up.toUpperCase()}`),
        Text(`LEFT   -> ${store.controls.left.toUpperCase()}`),
        Text(`DOWN   -> ${store.controls.down.toUpperCase()}`),
        Text(`RIGHT  -> ${store.controls.right.toUpperCase()}`),
        Text(`A      -> ${store.controls.a.toUpperCase()}`),
        Text(`B      -> ${store.controls.b.toUpperCase()}`),
        Text(`START  -> ${store.controls.start.toUpperCase()}`),
        Text(`SELECT -> ${store.controls.select.toUpperCase()}`),
    ]);

    const renderingList = VMenu([
        Text('Scaling factor'),
        Text('Color palette'),
        Text('Scale mode'),
    ]);

    const saveList = VMenu([
        Text('Save state'),
        Text('Load state'),
    ]);

    const subMenuMapping: Record<string, VMenu> = {
        library: libraryList,
        controls: controlsList,
        rendering: renderingList,
        save: saveList,
    };

    window.addEventListener('keydown', event => {
        switch (event.key) {
            case 'Escape':
                showUI.ref = !showUI.ref;
                event.preventDefault();
                break;
            case 'ArrowLeft':
                if (showUI.ref) {
                    menu.prev();
                }
                break;
            case 'ArrowRight':
                if (showUI.ref) {
                    menu.next();
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
        }

        if (showUI.ref) {
            menu.state.active = subMenuMapping[activeMenuItem].state.activeIndex === -1;
        }
    });

    function render(imageData: ImageData): void {
        screen.clear();
        menu.render(0, 6, screen);
        subMenuMapping[activeMenuItem].render(9, 9, screen);
        screen.render(imageData);
    }

    return { render, showUI };
};
