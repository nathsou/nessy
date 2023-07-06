
export type SavedEvent = { timestamp: number };
export type LoadedEvent = { timestamp: number };
export type LoadRequestEvent = { timestamp: number };
export type SaveRequestEvent = {};
export type LoadLastRequestEvent = {};

export type EventMapping = {
    saved: SavedEvent,
    loaded: LoadedEvent,
    loadRequest: LoadRequestEvent,
    saveRequest: SaveRequestEvent,
    loadLastRequest: LoadLastRequestEvent,
};

export type Events = SavedEvent;
type EventType = keyof EventMapping;

const createEventEmitter = () => {
    let id = 0;
    const eventTypeById = new Map<number, EventType>();
    const listeners: { [K in EventType]: Array<{ id: number, handler: (event: EventMapping[K]) => void }> } = {
        saved: [],
        loaded: [],
        loadRequest: [],
        saveRequest: [],
        loadLastRequest: [],
    };

    const emit = <K extends EventType>(type: K, event: EventMapping[K]): void => {
        listeners[type].forEach(({ handler }) => handler(event));
    };

    const on = <K extends EventType>(type: K, handler: (event: EventMapping[K]) => void): number => {
        id += 1;
        listeners[type].push({ id, handler });
        eventTypeById.set(id, type);
        return id;
    };

    const remove = (id: number): void => {
        const type = eventTypeById.get(id);

        if (type != null) {
            eventTypeById.delete(id);
            const index = listeners[type].findIndex((listener) => listener.id === id);
            if (index !== -1) {
                listeners[type].splice(index, 1);
            }
        }
    };

    return { emit, on, remove };
};

export const events = createEventEmitter();
