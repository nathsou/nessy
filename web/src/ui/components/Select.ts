import { Text, TextSettings } from "./text";

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
        onKeyDown(key: string): void {
            if (comp.state.active) {
                switch (key) {
                    case 'ArrowLeft':
                        let index = activeIndex - 1;
                        if (index < 0) {
                            index = options.length - 1;
                        }

                        setActiveIndex(index);
                        break;
                    case 'ArrowRight':
                    case 'Enter':
                        setActiveIndex((activeIndex + 1) % options.length);
                        break;
                }
            }
        },
    };
};
