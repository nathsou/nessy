import { events } from "../events";
import { SaveEntry, Store } from "../store";
import { Button } from "./Button";
import { VMenu } from "./VMenu";
import { Text } from "./Text";
import { hooks } from "../hooks";
import { Action } from "../ui";

const SAVE_MENU_ITEM_INDEX = 0;
const LOAD_LAST_MENU_ITEM_INDEX = 1;

export const Saves = (store: Store) => {
    const baseItems: Button[] = [
        Button(Text('Save (CTRL+S)'), () => hooks.call('saveState')),
        Button(Text('Load last (CTRL+L)'), () => hooks.call('loadLastSave')),
    ];

    const list = VMenu<Button>(baseItems, { visibleItems: 8, onSelect });

    list.width = 19;
    let saves: SaveEntry[] = [];

    const updateList = async () => {
        if (store.ref.rom == null) {
            list.update(baseItems);
        } else {
            saves = (await store.db.save.list(store.ref.rom)).sort((a, b) => b.timestamp - a.timestamp);
            list.update(baseItems.concat(saves.map(save => {
                const date = new Date(save.timestamp);
                return Button(
                    Text(`${date.toLocaleDateString()} ${date.toLocaleTimeString()}`),
                    () => hooks.call('loadSave', save.timestamp),
                );
            })));
        }
    };

    async function updateBackground() {
        const index = list.state.activeIndex;
        switch (index) {
            case SAVE_MENU_ITEM_INDEX:
                hooks.call('setBackground', { mode: 'current' });
                break;
            case LOAD_LAST_MENU_ITEM_INDEX:
                if (store.ref.rom != null) {
                    const lastSave = await store.db.save.getLast(store.ref.rom);
                    if (lastSave != null) {
                        hooks.call('setBackground', { mode: 'at', timestamp: lastSave.timestamp });
                    }
                }
                break;
            default:
                const { timestamp } = saves[index - baseItems.length];
                hooks.call('setBackground', { mode: 'at', timestamp });
                break;
        }
    }

    async function onSelect() {
        await updateBackground();
    }

    updateList();
    store.subscribe('rom', updateList);
    events.on('saved', updateList);

    const onAction = (action: Action): boolean => {
        if (action === 'start' || action === 'a') {
            list.state.items[list.state.activeIndex].enter();
            return true;
        }

        return false;
    };

    const setActive = (isActive: boolean) => {
        if (!isActive) {
            hooks.call('setBackground', { mode: 'current' });
        } else {
            updateBackground();
        }
    };

    return {
        ...list,
        onAction,
        setActive,
    };
};
