import { createControls } from "../controls";

const array = <T>(): T[] => [];
const union = <T>(value: T): T => value;

export type StoreData = ReturnType<typeof getDefaultStore>;
export type Store = Awaited<ReturnType<typeof createStore>>;
export const LOCAL_STORAGE_STORE_KEY = 'nessy.store';

const STORE_VERSION = 2;

const getDefaultStore = () => ({
    version: STORE_VERSION,
    rom: union<string | null>(null),
    controls: createControls(),
    scalingFactor: union<1 | 2 | 3 | 4 | 50>(4),
    scalingMode: union<'pixelated' | 'blurry'>('pixelated'),
    lastState: union<Uint8Array | null>(null),
});

export const Binary = {
    serialize(data: Uint8Array): string {
        return Array.from(data).map(byte => String.fromCharCode(byte)).join('');
    },
    deserialize(data: string): Uint8Array {
        return Uint8Array.from(data.split('').map(char => char.charCodeAt(0)));
    },
    async hash(data: BufferSource): Promise<string> {
        const digest = await crypto.subtle.digest('SHA-256', data);
        return Array.from(new Uint8Array(digest))
            .map((b) => b.toString(16).padStart(2, '0'))
            .join('');
    },
};

export type RomEntry = {
    hash: string,
    name: string,
    data: Uint8Array,
};

export type SaveEntry = {
    timestamp: number,
    romHash: string,
    state: Uint8Array,
};

export type TitleScreenEntry = {
    romHash: string,
    data: Uint8Array,
};

const createDatabase = async () => {
    const db = await new Promise<IDBDatabase>((resolve, reject) => {
        const request = indexedDB.open('nessy', 1);
        request.onerror = reject;

        request.onupgradeneeded = () => {
            const db = request.result;

            const roms = db.createObjectStore('roms', { keyPath: 'hash' });
            roms.createIndex('hash', 'hash', { unique: true });
            roms.createIndex('name', 'name', { unique: false });
            roms.createIndex('data', 'data', { unique: false });

            const saves = db.createObjectStore('saves', { keyPath: 'timestamp' });
            saves.createIndex('timestamp', 'timestamp', { unique: true });
            saves.createIndex('romHash', 'romHash', { unique: false });
            saves.createIndex('state', 'state', { unique: false });

            const titleScreens = db.createObjectStore('titleScreens', { keyPath: 'romHash' });
            titleScreens.createIndex('romHash', 'romHash', { unique: true });
            titleScreens.createIndex('data', 'data', { unique: false });
        };

        request.onsuccess = () => {
            resolve(request.result);
        };
    });

    async function insertROM(name: string, data: Uint8Array): Promise<string> {
        const entry: RomEntry = {
            hash: await Binary.hash(data),
            name: name.endsWith('.nes') ? name.slice(0, -4) : name,
            data,
        };

        const transaction = db.transaction(['roms'], 'readwrite');
        const roms = transaction.objectStore('roms');
        const request = roms.put(entry);

        return new Promise<string>((resolve, reject) => {
            request.onerror = reject;
            request.onsuccess = () => {
                resolve(entry.hash);
            };
        });
    };

    async function getROM(hash: string): Promise<RomEntry> {
        return new Promise<RomEntry>((resolve, reject) => {
            const transaction = db.transaction(['roms'], 'readonly');
            const roms = transaction.objectStore('roms');
            const request = roms.get(hash);

            request.onerror = reject;
            request.onsuccess = () => {
                if (request.result == null) {
                    reject(new Error(`ROM with hash ${hash} not found`));
                } else {
                    resolve(request.result);
                }
            };
        });
    }

    async function listROMs(): Promise<RomEntry[]> {
        return new Promise<RomEntry[]>((resolve, _reject) => {
            const transaction = db.transaction(['roms'], 'readonly');
            const roms = transaction.objectStore('roms');
            const request = roms.getAll();

            request.onerror = () => {
                resolve([]);
            };
            request.onsuccess = () => {
                resolve(request.result);
            };
        });
    }

    async function insertSave(romHash: string, state: Uint8Array): Promise<number> {
        return new Promise<number>((resolve, reject) => {
            const entry: SaveEntry = {
                timestamp: Date.now(),
                romHash,
                state,
            };

            const transaction = db.transaction(['saves'], 'readwrite');
            const saves = transaction.objectStore('saves');
            const request = saves.put(entry);

            request.onerror = reject;
            request.onsuccess = () => {
                resolve(entry.timestamp);
            };
        });
    }

    async function getSave(timestamp: number): Promise<SaveEntry> {
        return new Promise<SaveEntry>((resolve, reject) => {
            const transaction = db.transaction(['saves'], 'readonly');
            const saves = transaction.objectStore('saves');
            const request = saves.get(timestamp);

            request.onerror = reject;
            request.onsuccess = () => {
                if (request.result == null) {
                    reject(new Error(`Save with timestamp ${timestamp} not found`));
                } else {
                    resolve(request.result);
                }
            };
        });
    }

    async function getLastSave(romHash: string): Promise<SaveEntry | null> {
        return new Promise<SaveEntry | null>((resolve, reject) => {
            const transaction = db.transaction(['saves'], 'readonly');
            const saves = transaction.objectStore('saves');
            const index = saves.index('romHash');
            const request = index.openCursor(IDBKeyRange.only(romHash), 'prev');

            request.onerror = reject;
            request.onsuccess = () => {
                if (request.result == null) {
                    resolve(null);
                } else {
                    resolve(request.result.value);
                }
            };
        });
    }

    async function listSaves(romHash: string): Promise<SaveEntry[]> {
        return new Promise<SaveEntry[]>((resolve, _reject) => {
            const transaction = db.transaction(['saves'], 'readonly');
            const saves = transaction.objectStore('saves');
            const index = saves.index('romHash');
            const request = index.getAll(romHash);

            request.onerror = () => {
                resolve([]);
            };

            request.onsuccess = () => {
                resolve(request.result);
            };
        });
    }

    async function insertTitleScreen(romHash: string, data: Uint8Array): Promise<void> {
        return new Promise<void>((resolve, reject) => {
            const entry: TitleScreenEntry = {
                romHash,
                data,
            };

            const transaction = db.transaction(['titleScreens'], 'readwrite');
            const titleScreens = transaction.objectStore('titleScreens');
            const request = titleScreens.put(entry);

            request.onerror = reject;
            request.onsuccess = () => {
                resolve();
            };
        });
    }

    async function getTitleScreen(romHash: string): Promise<TitleScreenEntry | null> {
        return new Promise<TitleScreenEntry | null>((resolve, reject) => {
            const transaction = db.transaction(['titleScreens'], 'readonly');
            const titleScreens = transaction.objectStore('titleScreens');
            const request = titleScreens.get(romHash);

            request.onerror = reject;
            request.onsuccess = () => {
                if (request.result == null) {
                    resolve(null);
                } else {
                    resolve(request.result);
                }
            };
        });
    }

    async function listTitleScreens(): Promise<TitleScreenEntry[]> {
        return new Promise<TitleScreenEntry[]>((resolve, _reject) => {
            const transaction = db.transaction(['titleScreens'], 'readonly');
            const titleScreens = transaction.objectStore('titleScreens');
            const request = titleScreens.getAll();

            request.onerror = () => {
                resolve([]);
            };

            request.onsuccess = () => {
                resolve(request.result);
            };
        });
    }

    return {
        rom: { get: getROM, insert: insertROM, list: listROMs },
        save: { get: getSave, getLast: getLastSave, insert: insertSave, list: listSaves },
        titleScreen: { get: getTitleScreen, insert: insertTitleScreen, list: listTitleScreens },
    };
};

export const createStore = async () => {
    const db = await createDatabase();

    const serialize = (store: StoreData): string => {
        return JSON.stringify({
            ...store,
            controls: store.controls.ref,
            lastState: store.lastState != null ? Binary.serialize(store.lastState) : null,
        });
    };

    const deserialize = (string: string): StoreData => {
        let store = JSON.parse(string);

        if ('version' in store && store.version !== STORE_VERSION) {
            store = getDefaultStore();
        } else {
            const controls = createControls();
            controls.update(store.controls);
            store.controls = controls;
            store.lastState = store.lastState != null ? Binary.deserialize(store.lastState) : null;
        }

        return store;
    };

    const store = (() => {
        const savedStore = localStorage.getItem(LOCAL_STORAGE_STORE_KEY);

        if (savedStore != null) {
            return deserialize(savedStore);
        } else {
            return getDefaultStore();
        }
    })();

    const listeners = array<{ key: keyof StoreData, handler: (value: any, previous: any) => void }>();

    const get = <K extends keyof StoreData>(key: K): StoreData[K] => {
        return store[key];
    };

    const set = <K extends keyof StoreData>(key: K, value: StoreData[K]): void => {
        const prev = store[key];
        store[key] = value;

        listeners.forEach(({ key: listenerKey, handler }) => {
            if (listenerKey === key) {
                handler(value, prev);
            }
        });
    };

    const subscribe = <K extends keyof StoreData>(key: K, handler: (value: StoreData[K], previous: StoreData[K]) => void): void => {
        listeners.push({ key, handler });
    };

    const save = () => {
        localStorage.setItem(LOCAL_STORAGE_STORE_KEY, serialize(store));
    };


    return {
        ref: store,
        subscribe,
        get,
        set,
        save,
        db,
    };
};
