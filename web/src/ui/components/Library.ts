import { directoryOpen, fileOpen } from "browser-fs-access";
import { VMenu } from "./VMenu";
import { Text } from "./text";
import { store } from "../store";

export const Library = () => {
    const list = VMenu([
        Text('Load ROM file'),
        Text('Load ROM dir.'),
    ]);

    const loadRomFile = async () => {
        const file = await fileOpen({
            description: 'NES ROM file',
            extensions: ['.nes'],
            mimeTypes: ['application/octet-stream'],
            multiple: false,
            startIn: 'downloads',
        });

        const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
        store.set('rom', bytes);
    };

    const loadRomDir = async () => {
        const dir = await directoryOpen({
            recursive: false,
            mode: 'read',
            startIn: 'downloads',
        });

        console.log(dir);
    };

    const onClickMappings = [
        loadRomFile,
        loadRomDir,
    ];

    const onKeyDown = (key: string): void => {
        if (list.state.activeIndex !== -1 && key === 'Enter') {
            onClickMappings[list.state.activeIndex]();
        }
    };

    return {
        ...list,
        onKeyDown,
    };
};
