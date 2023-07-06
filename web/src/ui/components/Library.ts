import { fileOpen } from "browser-fs-access";
import { RomEntry, Store } from "../store";
import { VMenu } from "./VMenu";
import { Text } from "./text";

export const Library = (store: Store) => {
    const baseItems = [
        Text('Load ROMs...'),
    ];

    const list = VMenu(baseItems, 8);
    let roms: RomEntry[] = [];

    const updateList = async () => {
        roms = (await store.db.rom.list()).sort((a, b) => a.name.localeCompare(b.name));
        list.update(baseItems.concat(roms.map(rom => Text(rom.name, { maxLength: 22 }))));
    };

    updateList();

    const loadRomFile = async () => {
        try {
            const files = await fileOpen({
                description: 'NES ROM file',
                extensions: ['.nes'],
                mimeTypes: ['application/octet-stream'],
                multiple: true,
            });

            for (const file of files) {
                try {
                    const bytes = new Uint8Array(await file.arrayBuffer());
                    const hash = await store.db.rom.insert(file.name, bytes);

                    if (files.length === 1) {
                        store.set('rom', hash);
                    }
                } catch (error) {
                    console.error(`Failed to load file ${file.name}: ${error}`);
                }
            }

            await updateList();
        } catch (error) {
            console.error(error);
        }
    };

    const onEnterMappings = [
        loadRomFile,
    ];

    const onKeyDown = (key: string): void => {
        if (list.state.activeIndex !== -1 && key === 'Enter') {
            if (list.state.activeIndex < onEnterMappings.length) {
                onEnterMappings[list.state.activeIndex]();
            } else {
                const rom = roms[list.state.activeIndex - onEnterMappings.length];
                if (rom.hash !== store.ref.rom) {
                    store.set('rom', rom.hash);
                }
            }
        }
    };

    return {
        ...list,
        onKeyDown,
    };
};
