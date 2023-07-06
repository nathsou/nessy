import { events } from "../events";
import { SaveEntry, Store } from "../store";
import { Button } from "./Button";
import { VMenu } from "./VMenu";
import { Text } from "./text";

export const Saves = (store: Store) => {
    const baseItems: Button[] = [
        Button(Text('Save (CTRL+S)'), () => events.emit('saveRequest', {})),
        Button(Text('Load last (CTRL+L)'), () => events.emit('loadLastRequest', {})),
    ];

    const list = VMenu<Button>(baseItems, 8);

    list.width = 19;
    let saves: SaveEntry[] = [];

    const updateList = async () => {
        saves = (await store.db.save.list(store.ref.rom!)).sort((a, b) => b.timestamp - a.timestamp);
        list.update(baseItems.concat(saves.map(save => {
            const date = new Date(save.timestamp);
            return Button(
                Text(`${date.toLocaleDateString()} ${date.toLocaleTimeString()}`),
                () => events.emit('loadRequest', { timestamp: save.timestamp }),
            );
        })));
    };

    updateList();
    store.subscribe('rom', updateList);
    events.on('saved', updateList);

    const onKeyDown = (key: string): void => {
        if (list.state.activeIndex !== -1 && key === 'Enter') {
            list.state.items[list.state.activeIndex].enter();
        }
    };

    return {
        ...list,
        onKeyDown,
    };
};
