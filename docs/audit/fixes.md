# Audit repairs and validation

The repair covers the numbered findings in the [baseline audit](nessy-audit.md). CPU bus accesses now advance the PPU and APU individually, so mapper IRQs, register side effects, interrupt entry and DMA use a shared clock. MMC3 clocks from filtered PPU A12 transitions, including pre-render fetches and CPU accesses, instead of a fixed visible-scanline callback.

## Finding coverage

| Findings | Repair and evidence |
|---|---|
| M01–M03 | Persistent MMC3 IRQ line, immediate PRG/CHR mode changes, timed background/sprite address events and A12 filtering. All five common-MMC3 ROMs pass, including scanline timing in both pattern-table arrangements. |
| M04–M06 | Banked CHR RAM, bounded ROM bank selection, PRG RAM protection/open bus, hardwired four-screen mirroring and 4 KiB nametable storage. Direct regression tests cover these cases. |
| C01–C06 | Documented and unofficial instruction behavior, BRK/IRQ/NMI stack state, interrupt polling and hijacking, dummy reads/writes, stack bus order and wrapping arithmetic. Full nestest, exhaustive ADC/SBC, both complete 16-part instruction suites, dummy-access ROMs and all five interrupt ROMs pass. |
| C07–C09 | OAM DMA transfers one byte per get/put pair and counts alignment/stall cycles. DMC reads halt read cycles, retain buffered data, and take priority over OAM gets. Reset preserves registers/flags as appropriate and executes stack/vector cycles. DMA parity, IRQ/DMA and reset regressions pass. |
| P01–P05 | Forced blank/sprite-only rendering, nametable/palette mirroring, PPUDATA buffer and scroll behavior, PPU I/O latch, NMI suppression and odd-frame timing. All ten vblank/NMI ROMs pass. |
| Additional PPU limitations identified in the audit | Sprite evaluation now scans primary/secondary OAM during dots 65–256; overflow uses the diagonal byte-address bug, with cycle-specific tests. Pattern fetches are spread across sprite slots. Grayscale is implemented; RGB emphasis uses an approximation. |
| S01–S03 | Complete mapper, APU, video buffers, sprite and in-flight timing state is saved. Decoding checks bounds, booleans, enum/range values and nesting. Failed loads restore the prior state. Tests compare complete state, video and audio after a mid-frame replay and verify rollback after a late mapper-section failure. Web previews use separate emulator instances. |
| A01–A07 | Level IRQs, four/five-step frame sequencing, delayed `$4017` reset, CPU-rate DMC/noise timers, DMC buffering/IRQ rules, channel length/envelope/sweep behavior, wide cycle counters and bounded sample phase, audio overflow and underrun handling. All eight APU ROMs and direct regressions pass. |
| R01–R03 | ROM header/payload validation; explicit rejection of unsupported formats; consistent RAM windows; immutable CHR ROM; bounded banks; MMC1 consecutive-write filtering and RAM disable. |
| I01 | Both controller ports are routed and strobed, with latched button snapshots. |
| F01–F04 | Audio consumption owns web emulation speed; display refresh only presents frames. Restored bindings replace authoritative preferences and reverse maps. Axis release/disconnect/blur clear input. Keyboard and gamepad UI presses, including button zero, work; Control and Meta shortcuts are handled. Seven control regressions pass. |

## Validation

- 49 Rust tests pass: 47 integration checks and two PPU evaluation tests. None are ignored.
- 7 Node control checks pass.
- 36 independent ROMs pass; [individual outputs](fixed-rom-results.txt) preserve their result text.
- Formatting and Clippy with warnings denied pass.
- Desktop compilation passes.
- WebAssembly release build, Vite production build and strict TypeScript checking pass. The stale web lockfile was refreshed to satisfy its existing wasm-bindgen requirement; web builds now use `--locked`.
- CI runs Rust/control checks and the pinned public ROM suite. Pull requests build the web app without deploying it.

The public ROM collection is pinned to [95d8f621ae55cee0d09b91519a8989ae0e64753b](https://github.com/christopherpow/nes-test-roms/tree/95d8f621ae55cee0d09b91519a8989ae0e64753b). No commercial ROM is bundled or required.

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
node tests/audit_controls.mjs

git clone https://github.com/christopherpow/nes-test-roms.git /tmp/nessy-test-roms
git -C /tmp/nessy-test-roms checkout 95d8f621ae55cee0d09b91519a8989ae0e64753b
bash tests/conformance.sh /tmp/nessy-test-roms

cargo check --manifest-path desktop/Cargo.toml --locked
cd web/ui
npm ci
npm run wasm
npm run copy
npm run build
npx tsc --noEmit
```

## Compatibility and limits

- Save format is now **1**. Format-0 saves lack required state and are rejected explicitly. The web UI starts a fresh session if an old automatic save cannot be loaded; old preview failures do not mutate the running game.
- `CPU::step()` now clocks connected devices itself and returns one instruction, interrupt-entry sequence or DMA transfer. Direct callers must remove their separate `Bus::advance()` call. The public `Nes` methods already do this.
- Mapper 4 models the common Sharp MMC3B/C IRQ behavior. The alternate-silicon `6-MMC3_alt` ROM is intentionally excluded because it requires incompatible zero-counter behavior. MMC6, NES 2.0 submappers and additional cartridge boards remain unsupported. A12 filtering uses a fixed PPU-dot low-time threshold for the chosen CPU/PPU phase; arbitrary power-on phase/revision combinations are not modeled.
- Analog-sensitive unofficial opcodes use deterministic values. RGB emphasis is approximate; transistor-level OAM corruption, analog open-bus decay and arbitrary DMA cancellation races are not claimed as fully modeled.
- No SMB3 game image or failing gameplay save was available. The mapper/CPU causes have independent conformance coverage, but the particular reported SMB3 scenes have not been replayed. Browser interaction/audio playback and physical 3DS execution were not exercised; web/desktop validation here is compilation plus automated logic tests.
- Standalone battery-RAM import/export remains a feature gap; complete save states preserve RAM. The second controller is available in the core, while frontend player-two bindings remain a UI feature.
