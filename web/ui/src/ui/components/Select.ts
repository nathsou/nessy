import { Action } from "../ui";
import { Text, TextSettings } from "./Text";

type SelectSettings<Opt extends string> = {
    name: string,
    options: Opt[],
    initialOption?: Opt,
    onChange: (option: Opt, index: number) => void,
    text?: TextSettings,
};

export const Select = <Opt extends string>({ name, options, onChange, initialOption = options[0], text: textSettings }: SelectSettings<Opt>) => {
    const comp = Text(name, textSettings);
    let activeIndex: number;

    const setActiveIndex = (index: number): void => {
        activeIndex = index;
        comp.update(name + ': ' + options[activeIndex]);
        onChange(options[index], index);
    };

    setActiveIndex(options.indexOf(initialOption));

    return {
        ...comp,
        onAction(action: Action): boolean {
            if (comp.state.active) {
                switch (action) {
                    case 'select':
                        setActiveIndex(activeIndex === 0 ? options.length - 1 : activeIndex - 1);
                        return true;
                    case 'start':
                    case 'a':
                        setActiveIndex((activeIndex + 1) % options.length);
                        return true;
                }
            }

            return false;
        },
    };
};
