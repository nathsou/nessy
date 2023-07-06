import { Controls } from "./components/Controls";
import { HMenu } from "./components/HMenu";
import { Library } from "./components/Library";
import { VMenu } from "./components/VMenu";
import { Text } from "./components/text";
import { createScreen } from "./screen";
import { Store } from "./store";

export const createUI = (store: Store) => {
    let showUI = { ref: store.ref.rom == null };
    const screen = createScreen();
    const menuItems = [
        'library',
        'controls',
        // 'rendering',
        // 'save',
    ];

    const menu = HMenu(menuItems.map(item => Text(item)), 0);

    const renderingList = VMenu([
        Text('Scaling factor'),
        Text('Color palette'),
        Text('Scale mode'),
    ]);

    const saveList = VMenu([
        Text('Save state'),
        Text('Load state'),
    ]);

    const library = Library(store);
    const controls = Controls(store);

    const subMenuMapping: Record<string, typeof library> = {
        library: library,
        controls: controls,
        // rendering: renderingList,
        // save: saveList,
    };

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
            default: {
                if (showUI.ref) {
                    const menuItem = subMenuMapping[activeMenuItem];
                    menuItem.onKeyDown(event.key);
                }
                break;
            }
        }

        if (showUI.ref) {
            menu.state.active = subMenuMapping[activeMenuItem].state.activeIndex === -1;
        }
    });

    function render(imageData: ImageData): void {
        screen.clear();
        menu.render(0, 6, screen);
        const activeMenuItem = menuItems[menu.state.activeIndex];
        subMenuMapping[activeMenuItem].render(9, 9, screen);
        screen.render(imageData);
    }

    return { render, showUI };
};
