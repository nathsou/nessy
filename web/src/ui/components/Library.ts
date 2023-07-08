import { fileOpen } from "browser-fs-access";
import { RomEntry, Store } from "../store";
import { VMenu } from "./VMenu";
import { Text } from "./Text";
import { Button } from "./Button";
import { hooks } from "../hooks";

const MAX_LENGTH = 22;

export const Library = (store: Store) => {
    const baseItems = [
        Button(Text('Load ROMs...'), loadRoms),
    ];

    const list = VMenu(baseItems, { visibleItems: 8, onSelect });
    list.width = MAX_LENGTH;
    let roms: RomEntry[] = [];

    async function loadRoms(): Promise<void> {
        try {
            const files = await fileOpen({
                description: 'NES ROM file',
                extensions: ['.nes'],
                mimeTypes: ['application/octet-stream'],
                multiple: true,
            });

            let index = 0;
            const getLoaderText = () => `Title Screens [${index}/${files.length}]`;
            const loader = Text(getLoaderText());
            list.update(baseItems.concat(Button(loader, () => { })));

            for (const file of files) {
                try {
                    const bytes = new Uint8Array(await file.arrayBuffer());
                    const romHash = await store.db.rom.insert(file.name, bytes);

                    if (files.length === 1) {
                        store.set('rom', romHash);
                    }

                    await hooks.call('generateTitleScreen', romHash);
                } catch (error) {
                    console.error(`Failed to load file ${file.name}: ${error}`);
                }

                index += 1;
                loader.update(getLoaderText());
            }

            await updateList();

        } catch (error) {
            console.error(error);
        }
    };

    const playROM = (rom: RomEntry): void => {
        store.set('rom', rom.hash);
    };

    const updateList = async () => {
        roms = (await store.db.rom.list()).sort((a, b) => a.name.localeCompare(b.name));
        list.update(baseItems.concat(roms.map(rom => Button(Text(rom.name, { maxLength: MAX_LENGTH }), () => playROM(rom)))));
    };

    function updateBackground() {
        const index = list.state.activeIndex;
        if (index >= baseItems.length) {
            const hash = roms[index - baseItems.length].hash;
            hooks.call('setBackground', { mode: 'titleScreen', hash });
        } else {
            hooks.call('setBackground', { mode: 'current' });
        }
    }

    function onSelect() {
        updateBackground();
    }

    updateList();

    const onKeyDown = (key: string): boolean => {
        if (key === 'Enter') {
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
        onKeyDown,
        setActive,
    };
};
