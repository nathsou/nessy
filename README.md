# NESSY

<center>
    <img alt="logo" src="resources/logo.png" width="256px" style="image-rendering: pixelated;" />

**A Nintendo Entertainment System (NES) emulator for the web**

</center>


## Screenshots

![Library](resources/metroid.png)
![Library](resources/smb.png)
![Library](resources/tetris.png)

![Library](resources/contra.png)
![Library](resources/zelda.png)
![Library](resources/megaman2.png)

![Library](resources/castlevania.png)
![Library](resources/library.png)
![Library](resources/saves.png)

## Features

- Save states (CTRL+S to save, CTRL+L to load last)
- Customizable controls
- Full Screen mode
- Working audio
- Auto generated ROM backgrounds in the library
- Save state preview in the saves menu

## Todo / Ideas

- Support more mappers (3, 4, 118, ...)
- Reduce audio clicks (blip buffer?)
- Use an AudioWorkletNode instead of a scriptProcessor
- Time travel mode (Save states recorded at regular intervals)
- Replay mode (Replay all inputs from the start)
- Different color palettes
- Better UX
- VR / 3D mode with sprites in front and bg tiles in the background?
- Compile 6502 instructions to WASM directly?