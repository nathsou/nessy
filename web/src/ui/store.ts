const array = <T>(): T[] => [];
const union = <T>(value: T): T => value;

export type StoreData = typeof DEFAULT_STORE;
export type Store = Awaited<ReturnType<typeof createStore>>;
export const LOCAL_STORAGE_STORE_KEY = 'nessy.store';

const DEFAULT_STORE = {
    version: 1,
    frameCount: 0,
    rom: union<string | null>(null),
    interface: {
        primary: union<'library' | 'controls'>('controls'),
        secondary: union<string | number>(0),
    },
    controls: {
        up: 'w',
        left: 'a',
        down: 's',
        right: 'd',
        a: 'l',
        b: 'k',
        start: 'Enter',
        select: 'Space',
    },
};

const Binary = {
    async hash(binary: BufferSource): Promise<string> {
        const digest = await crypto.subtle.digest('SHA-256', binary);
        const hexes = [];

        const view = new DataView(digest);
        for (let i = 0; i < view.byteLength; i += 4) {
            hexes.push(('00000000' + view.getUint32(i).toString(16)).slice(-8));
        }

        return hexes.join('');
    },
};

export type RomEntry = {
    hash: string,
    name: string,
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
            resolve(db);
        };

        request.onsuccess = () => {
            resolve(request.result);
        };
    });

    const insertROM = async (name: string, data: Uint8Array): Promise<string> => {
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

    const getROM = async (hash: string): Promise<RomEntry> => {
        const transaction = db.transaction(['roms'], 'readonly');
        const roms = transaction.objectStore('roms');
        const request = roms.get(hash);

        return new Promise<RomEntry>((resolve, reject) => {
            request.onerror = reject;
            request.onsuccess = () => {
                if (request.result == null) {
                    reject(new Error(`ROM with hash ${hash} not found`));
                } else {
                    resolve(request.result);
                }
            };
        });
    };

    const listROMs = async (): Promise<RomEntry[]> => {
        const transaction = db.transaction(['roms'], 'readonly');
        const roms = transaction.objectStore('roms');
        const request = roms.getAll();

        return new Promise<RomEntry[]>((resolve, _reject) => {
            request.onerror = () => {
                resolve([]);
            };
            request.onsuccess = () => {
                resolve(request.result);
            };
        });
    };

    return {
        rom: { get: getROM, insert: insertROM, list: listROMs },
    };
};

export const createStore = async () => {
    const serialize = (store: StoreData): string => {
        return JSON.stringify(store);
    };

    const deserialize = (string: string): StoreData => {
        return JSON.parse(string);
    };

    const store = (() => {
        const savedStore = localStorage.getItem(LOCAL_STORAGE_STORE_KEY);

        if (savedStore != null) {
            return deserialize(savedStore);
        } else {
            return DEFAULT_STORE;
        }
    })();

    const listeners = array<{ key: keyof StoreData, handler: (value: any) => void }>();

    const get = <K extends keyof StoreData>(key: K): StoreData[K] => {
        return store[key];
    };

    const set = <K extends keyof StoreData>(key: K, value: StoreData[K]): void => {
        store[key] = value;

        listeners.forEach(({ key: listenerKey, handler }) => {
            if (listenerKey === key) {
                handler(value);
            }
        });
    };

    const subscribe = <K extends keyof StoreData>(key: K, handler: (value: StoreData[K]) => void): void => {
        listeners.push({ key, handler });
    };

    const save = () => {
        localStorage.setItem(LOCAL_STORAGE_STORE_KEY, serialize(store));
    };

    const db = await createDatabase();

    return {
        ref: store,
        subscribe,
        get,
        set,
        save,
        db,
    };
};
