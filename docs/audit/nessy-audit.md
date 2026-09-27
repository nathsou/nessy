# Nessy audit — 27 September 2026

**Historical baseline.** The findings below describe the original revision; see [fixes and current validation](fixes.md) for their resolution. Code links point to the audited commit.

Audited revision: `7a24b064bb2c608f1b9047fe679a339406ef09a1`.

**The strongest SMB3 suspects are MMC3 IRQ delivery, MMC3 clocking, and the timing of CPU accesses to the PPU.** There are also immediate bank-switching and save-state defects. Basic documented CPU arithmetic is substantially healthier than the interrupt and peripheral integration.

The original audit added tests, a headless ROM runner, and this report without changing production code. No SMB3 ROM or failing gameplay save was present, so the links between confirmed defects and the user's particular SMB3 scenes are hypotheses, not a claimed gameplay reproduction.

**Scope and evidence**

Reviewed all Rust core modules, the four mapper implementations, desktop/web/3DS entry points, web playback, controls, storage, UI components, and the deployment workflow. Executed core tests, independent ROMs, control reproductions, a desktop compilation check, and Clippy. Browser rendering/audio, a full web build, and 3DS execution were not performed. The 3DS port also references local assets absent from the tracked files.

Evidence labels below distinguish **reproduced** (targeted executable check), **ROM** (independent test result), and **inspection** (code path established, not separately exercised). P1 means high-impact compatibility, crash, or state corruption; P2 means narrower compatibility or accuracy failure. These are repair priorities, not security ratings.

| Check | Result |
|---|---|
| Existing tests before the audit | Core built; zero tests ran despite bundled nestest data. |
| Bundled nestest, documented-instruction prefix | 5,003 instruction states and aggregate cycle counts match. |
| ADC/SBC | All 262,144 combinations of operation, accumulator, operand, and carry pass result/N/Z/C/V checks, with decimal flag set to verify NES binary behavior. |
| Added normal Rust tests | 3 pass. |
| Opt-in Rust defect tests | All 36 fail as expected on the audited code; they assert intended behavior, rather than asserting bugs should remain. |
| Web control reproductions | 4 failures reproduced. |
| Independent ROMs on unchanged code | 41 runs: 5 pass, 33 report failures, 3 panic. These are selected diagnostic suites, not a general compatibility score. |
| Desktop | `cargo check --manifest-path desktop/Cargo.toml --locked --offline` passes after fetching locked dependencies. |
| Clippy | Completes with 6 existing warnings, principally unused pulse-channel identity and style suggestions. |

The ROM collection is [christopherpow/nes-test-roms](https://github.com/christopherpow/nes-test-roms) at `95d8f621ae55cee0d09b91519a8989ae0e64753b`. The test authors' sources and expected hardware behavior are included there. Detailed results: [ROM matrix](rom-results.md), [Rust reproductions](regressions.txt), [control reproductions](controls.txt).

**MMC3: first repair targets**

**M01 — P1 — IRQ is acknowledged by polling it. Reproduced.**

[MMC3::is_asserting_irq](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mappers/mmc3.rs#L142) clears `irq_asserted`. [Bus::pull_interrupt](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/bus/mod.rs#L56) calls it before the CPU checks I. An IRQ arriving inside an interrupt-disabled section disappears at the next instruction boundary; CLI cannot recover it. Even an accepted interrupt is acknowledged without the game writing `$E000`. Tests `mmc3_irq_is_level` and `masked_mmc3_irq_survives_until_cli` reproduce both aspects.

Expose a persistent IRQ line; only the appropriate mapper register write should acknowledge it. This is a strong candidate for occasional missed SMB3 splits. The expected acknowledgment mechanism is documented in [NESdev's IRQ reference](https://www.nesdev.org/wiki/IRQ).

**M02 — P1 — `$8000` changes do not immediately change PRG/CHR mapping. Reproduced.**

[Bank-select handling](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mappers/mmc3.rs#L71) stores bits 6/7, but rebuilds offsets only in the `$8001` branch. Program R6=3, then write `$46` to `$8000`: `$8000` still reads bank 3 instead of the fixed second-last bank, and `$C000` is also stale. CHR inversion behaves the same way. Tests `mmc3_prg_mode_is_immediate` and `mmc3_chr_mode_is_immediate` fail.

Recompute derived offsets after both bank-select and bank-data writes. An affected game can execute the wrong code or fetch the wrong graphics between those writes. Whether SMB3 uses that exact sequence in the reported scenes still needs a game trace.

**M03 — P1 — IRQ clocks are disconnected from the PPU address bus. Reproduced + ROM.**

[PPU::tick](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/ppu/mod.rs#L140) calls `step_scanline()` at dot 260 on visible scanlines whenever either rendering bit is enabled. There is no A12 observation or low-time filter in the mapper interface. Consequently:

- Counter clocks caused by `$2006`/`$2007` accesses are missing.
- The pre-render line is omitted.
- The counter advances even when both 8×8 pattern tables are at `$0000` and A12 never rises.
- Pattern-table selection and 8×16 sprite tile addresses cannot determine IRQ timing.

`mmc3_no_irq_when_a12_stays_low` reproduces a false IRQ. The independent `mmc3_test_2` clocking, A12, and scanline-timing ROMs all fail; the timing ROM specifically reports a late scanline-0 IRQ with `$2000=$08`.

Implement qualified PPU A12 edges from actual fetch/address events, including pre-render and CPU-driven PPU accesses. The current renderer also combines both background pattern reads at dot 7 and fetches every sprite's pattern at dot 257 ([background](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/ppu/mod.rs#L166), [sprites](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/ppu/mod.rs#L316)). Correct A12 timing therefore requires splitting those events, including dummy sprite fetches; adding a callback to the current bulk reads is insufficient. [MMC3 hardware reference](https://www.nesdev.org/wiki/MMC3).

**M04 — P1 — CHR storage and bank bounds are incomplete. Reproduced.**

[CHR reads](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mappers/mmc3.rs#L50) always index ROM. A mapper-4 image with CHR size zero has no CHR-RAM backing, and PPU writes are ignored. Selecting CHR bank `$FF` on an 8 KiB CHR image indexes beyond the cartridge buffer rather than mirroring the connected address lines. Tests `mmc3_chr_ram` and `mmc3_chr_banks_wrap` panic. Add banked CHR-RAM for supported boards and bounds/address-line normalization. This affects mapper-4 coverage; it is not evidence of an SMB3 CHR-RAM problem.

**M05 — P2 — PRG-RAM enable/write protection is ignored. Reproduced.**

[`$A001` writes](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mappers/mmc3.rs#L115) have no effect, and all `$6000–$7FFF` accesses remain enabled. Write `$80` to enable, store `$12`, write `$C0` to protect, then store `$34`: the read returns `$34`. Implement enable and protection bits and preserve them in save states. Disabled reads should follow the chosen bus model.

**M06 — P1 — Four-screen cartridges can crash, and `$A000` breaks their wiring. Reproduced.**

The mapper overwrites `Mirroring::FourScreen` on any `$A000` write. Independently, [PPU nametable translation](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/ppu/mod.rs#L493) returns offsets up to 4095 into a 2048-byte allocation. Writing `$2800` panics. Preserve hardwired four-screen mode and provide the extra nametable storage. Both conditions have targeted tests. SMB3 normally does not use four-screen wiring.

**MMC3 details not counted as demonstrated SMB3 bugs.** The decrement/reload rule at [line 149](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mappers/mmc3.rs#L149) is consistent with the common Sharp-style zero-latch behavior. Setting the counter to zero on `$C001` is not, by itself, proof of a wrong next-edge reload in this model. Likewise, zeroed power-up registers should not be judged against an invented deterministic hardware initial value. The two revision-specific ROMs (`5-MMC3` and `6-MMC3_alt`) target different chips; an implementation is not expected to satisfy both variants simultaneously. Bank-count arithmetic uses `u8`, and PRG values lack explicit six-bit masking, but oversized images need a declared board-support policy before those become ordinary MMC3 fixes.

**CPU instructions, interrupts, and timing**

All 151 documented opcodes have explicit handlers. The nestest prefix and exhaustive ADC/SBC results support the ordinary register/flag implementation, indexed address wrapping, and much of the cycle table. They do not establish bus-cycle accuracy or correct interrupt behavior.

| ID / priority | Finding and trigger | Evidence / repair |
|---|---|---|
| C01 / P1 | All 105 remaining opcode slots default to the same one-byte no-op. Multi-byte unofficial NOPs leave operands to be executed as opcodes; stable unofficial arithmetic/load/store instructions and JAM behavior are absent. [Dispatch](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/instructions.rs#L10). | Full nestest first diverges at line 5005: after opcode `$04`, PC is `$C6BE` instead of `$C6BF`. Implement real lengths, addressing modes, cycles, and supported unofficial operations. |
| C02 / P1 | BRK stacks the PC after opcode fetch, missing its padding byte. RTI returns to BRK+1. [BRK](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/instructions.rs#L205). | `brk_return_address` and independent `16-special` reproduce it. Consume/read the padding byte and stack BRK+2. |
| C03 / P2 | IRQ and NMI call PHP, so their stacked status has B=1. [Interrupt entry](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/instructions.rs#L212). | Separate IRQ/NMI stack tests reproduce `$30` where bits 4/5 should be `$20`. Keep PHP/BRK's synthesized B distinct from hardware interrupts. |
| C04 / P1 | IRQ polling uses the current I flag at the start of the next step. CLI/SEI/PLP latency and branch polling windows are absent. [Polling](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/instructions.rs#L185). | `cli_defers_irq_for_one_instruction` fails. CPU-interrupt ROMs also fail, though some are blocked first by APU IRQ defects. Model interrupt sampling at the correct instruction cycle. |
| C05 / P1 | Bus access sequences are incomplete. RMW operations write only the final value; indexed reads omit dummy accesses; untaken branches skip the operand read; `read_word` reads high before low. JSR pushes both bytes before fetching either operand. [RMW](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/instructions.rs#L736), [addressing](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mod.rs#L174), [word reads](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/memory.rs#L5), [JSR](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/instructions.rs#L1198). | RMW on `$2004` writes `$41` into the original OAM byte instead of writing `$40` there and `$41` into the next. Executing JSR at `$01FC` gives `$0301` instead of `$0300` when the push aliases the low operand. Both independent dummy-read tests fail. Implement reads and writes in hardware order, with timing. |
| C06 / P2 | Several address increments use ordinary `+`: next-word high-byte fetch, untaken branches, JSR return address, RTS, generic word reads/writes. [Fetch](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mod.rs#L136), [RTS](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/instructions.rs#L1207). | `rts_wraps_address_space` panics in debug on a stacked `$FFFF`; normal release wraps. Use wrapping arithmetic explicitly so debug and release agree. Ordinary indexed and relative wrapping already pass their independent ROMs in release. |
| C07 / P2 | OAM/DMC stall returns do not update `total_cycles`. [Stalls](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/instructions.rs#L168). | Two DMA transfers separated by a two-cycle NOP both stall 513 cycles; the second should switch parity to 514. Count every CPU cycle, including stalls. OAM transfer itself is also bulk-copied before its delay, so DMA/DMC arbitration remains inaccurate. |
| C08 / P1 | CPU memory effects happen before the corresponding elapsed time is advanced. Interrupt entry and the first handler instruction also execute within one `step`. [Nes::step](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/nes.rs#L17), [CPU::step](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/instructions.rs#L185), [Bus::advance](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/bus/mod.rs#L71). | Inspection plus independent NMI/IRQ/PPU timing failures. A handler's first PPU write can occur before its seven entry cycles have reached the PPU. Advance components at bus-cycle boundaries; total instruction cycles alone cannot make raster writes precise. |
| C09 / P2 | Soft reset replaces SP and all flags with power-on defaults and resets the CPU clock while peripherals retain their state. [Reset](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mod.rs#L92). | SP `$80` becomes `$FD`, instead of `$7D`, in the targeted reset test. Implement the reset sequence, including stack decrement and appropriate CPU/APU reset effects. |

The documented CLI/SEI/PLP sampling distinction is described in [NESdev's CPU interrupt reference](https://www.nesdev.org/wiki/Interrupts).

**Avoid misreading the independent instruction failures.** On unchanged production code, `official_only.nes` stops at `01-basics`, because the APU generates an interrupt in five-step/inhibited mode (A02 below). JMP/JSR, RTS, and RTI singles also fail in this environment. In a separate temporary copy, removing only the five-step `self.frame_interrupt = true` assignment lets the official suite pass tests 1–14 before BRK fails at test 15; the three control-flow singles then pass. This diagnostic experiment was not applied to the working tree. [Experiment log](isolated-experiment.txt). It does not erase the separately reproduced JSR bus-order and debug-overflow bugs.

**PPU and save states**

| ID / priority | Finding | Evidence and consequence |
|---|---|---|
| P01 / P1 | Rendering pixels, background fetches, and scroll updates are all inside `show_background()`. [PPU::step](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/ppu/mod.rs#L156). | Reproduced: forced blank fails to draw the configured backdrop. Inspection: sprite-only rendering also stops drawing; scroll/fetch machinery that should run when either rendering bit is active stops when only sprites are enabled. Stale frame pixels remain visible. |
| P02 / P2 | `$3000–$3EFF` writes are discarded. Palette special aliases are checked before reducing the address modulo 32, so `$3F30` does not alias `$3F00`. [PPUDATA writes](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/ppu/mod.rs#L538). | Both cases reproduced. Normalize the 14-bit external address, nametable mirrors, and palette index before applying special aliases. Rendering can make internal `v` exceed `$3FFF`; CPU-facing access should not simply reject that value. |
| P03 / P2 | Palette reads return directly without refilling the PPUDATA buffer from the underlying nametable. [PPUDATA reads](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/ppu/mod.rs#L512). | Reproduced: store `$42` at `$2F00`, read `$3F00`, then read `$2000`; the buffered result is zero rather than `$42`. Rendering-time `$2007` scroll increments and palette grayscale/open-bus details also need implementation. |
| P04 / P1 | Writing `$2002` panics. Reads from write-only PPU registers return zero rather than the I/O latch, and reads do not consistently update that latch. [Registers](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/ppu/mod.rs#L580). | `$2002` write reproduced directly; independent vblank-basics and dummy-write ROMs abort here. Hardware ignores the status write while updating its I/O bus latch. This is a crash, not an invalid-ROM error. |
| P05 / P2 | NMI/vblank timing and odd-frame enable timing are inaccurate. [Tick/NMI](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/ppu/mod.rs#L106). | Seven of the ten vblank/NMI ROMs report failures; one panics at P04 and two pass. Bulk CPU timing is a contributor; the pending NMI logic has no full edge/suppression model. Preserve the distinction between an observed NMI edge and the current output level. |
| S01 / P1 | Mapper-controlled `Cart::mirroring` is not saved/restored. [PPU save/load](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/ppu/mod.rs#L635), [MMC1 load](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mappers/mmc1.rs#L182). | Reproduced on MMC3: save vertical, switch horizontal, load; it remains horizontal. MMC1 similarly restores control fields without reapplying mirroring. A strong candidate for scrolling corruption specifically after loading SMB3 saves. |
| S02 / P1 | The APU is entirely absent from bus save/load. SpriteData also omits its decoded `chr` pixels; both frame buffers are omitted. [Bus save](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/bus/mod.rs#L136), [sprite save](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/ppu/mod.rs#L615). | APU status restoration reproduced. Mid-frame sprite/frame omissions established by inspection. Web save previews temporarily load, advance, and restore the live emulator ([renderState](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/web/ui/src/main.ts#L222)), so a preview can leave audio, IRQs, mirroring, or partial video from the previewed state. Serialize all emulated state or generate previews in a separate emulator. |
| S03 / P1 | Save decoding indexes untrusted lengths and `expect`s a section terminator; load mutates live state before validating all remaining sections. [Decoder](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/savestate.rs#L46), [header](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/savestate.rs#L180). | Empty input panics, reproduced. A later missing/invalid section can return an error after earlier state was changed, by inspection. Bounds-check decoding, validate values and depth, then commit a completely decoded state. |

Additional explicit accuracy limitations: sprite-overflow hardware behavior is simplified; grayscale/emphasis bits are defined but unused; sprite evaluation/fetch timing is bulk-modeled. These should have dedicated rendering tests after the IRQ/bus foundation is corrected. The audit does not claim visual correctness based on a frame completing.

**Audio**

| ID / priority | Finding | Evidence / repair |
|---|---|---|
| A01 / P1 | APU IRQ output is edge-detected using `prev_irq`, instead of remaining asserted while either flag is set. [IRQ output](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/apu/mod.rs#L334). | Repeated-poll test fails. Like M01, masked IRQs can disappear and unrelated sources can mask each other's transitions. Expose the OR of the flags as a level. |
| A02 / P1 | Five-step mode unconditionally sets a frame IRQ, even with inhibit enabled. `$4017` mode changes also omit the immediate quarter/half-frame action and the hardware reset delay. [Frame sequencer](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/apu/mod.rs#L280), [write](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/apu/mod.rs#L185). | `five_step_has_no_frame_irq` fails. This is the cause isolated in the official-instruction experiment. Independent length/jitter/timing tests fail too. Rebuild the frame-counter event schedule against test-ROM expectations. |
| A03 / P1 | DMC completion tests `interrupt_flag` rather than `irq_enabled`, so a clear flag cannot become set. DMC/noise periods expressed in CPU cycles are used by timers stepped only every other CPU cycle, with an additional period+1 countdown. [DMC completion](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/apu/dmc.rs#L94), [clocking](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/apu/mod.rs#L254), [timer](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/apu/common.rs#L8). | DMC completion reproduced; independent rate test reports rate 0 too long. Noise shares the cadence mismatch by inspection. Correct period units; implement DMC sample buffering, IRQ clear rules, and DMA arbitration rather than treating each byte as a direct shift-register replacement. |
| A04 / P2 | Triangle never enables its length counter's decrement path. Pulse/noise/triangle length-register writes load lengths even when the channel is disabled. [Triangle](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/apu/triangle.rs#L29). | Triangle length=2 remains active past multiple half frames, reproduced. The disabled-load issue follows all three write handlers. Correct channel enable and halt semantics. |
| A05 / P2 | Pulse envelope restart is set by `$4000/$4004`, and is absent on `$4003/$4007`; sweep lacks pulse-1's extra negate subtraction and the shift-zero update guard. [Pulse](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/apu/pulse.rs#L75). | Inspection; these alter note envelopes and pitch. The unused `id` warning reflects a real missing channel distinction, not just cosmetic dead code. |
| A06 / P2 | APU cycle counter overflows after about 40 minutes of NTSC emulated time. [Step](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/apu/mod.rs#L247). | Inspection: checked builds panic at `u32::MAX`; release wraps the absolute sample target while `samples_pushed` remains large, causing a sample every CPU tick until the accounting catches up. Use a wider monotonic clock or phase accumulator. CPU total cycles also use a checked u32 addition. |
| A07 / P2 | Audio ring has no full/overwrite handling, and `fill` leaves the unfilled output tail unchanged. [Push](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/apu/mod.rs#L154), [fill](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/apu/mod.rs#L320). | Inspection: full and empty share the same pointers, dropping queued audio after 8192 unconsumed samples. Define overflow policy and silence any underrun tail. This interacts with web overproduction below. |

**Cartridges, controllers, and frontends**

| ID / priority | Finding | Evidence / repair |
|---|---|---|
| R01 / P1 | ROM loader checks magic through unchecked indexing and never validates payload lengths or zero PRG size. NES 2.0 is not explicitly rejected. [Loader](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/rom.rs#L47). | Empty ROM panic reproduced; truncated payloads defer the crash into emulation. Validate header/payload and supported format/board before constructing a mapper. |
| R02 / P1 | NROM and UNROM allocate 2 KiB PRG RAM but unmasked writes address an 8 KiB window; NROM also lacks CHR-RAM for zero-CHR images. [NROM](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mappers/nrom.rs#L45), [UNROM](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mappers/unrom.rs#L54). | `$6800` writes panic in both targeted tests. An independent dummy-write ROM also crashes in NROM RAM. Make board-specific RAM capacity and mirroring consistent for reads and writes. |
| R03 / P2 | MMC1/UNROM CHR-ROM writes mutate ROM bytes; some bank selections can run beyond small ROM images; MMC1 lacks consecutive-cycle write suppression and PRG-RAM-disable behavior. [MMC1](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mappers/mmc1.rs#L80), [UNROM](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/cpu/mappers/unrom.rs#L54). | Inspection. ROM mutations are also omitted from saves, causing divergent restores. When adding CPU RMW dummy writes, add MMC1's consecutive-write filtering at the same time to avoid new regressions. Board variants/bus conflicts need an explicit support policy. |
| I01 / P2 | `$4017` reads go to the APU, and `$4016` strobe writes reach only joypad1. Joypads read live button bits rather than a latched snapshot. [Bus](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/bus/mod.rs#L99), [Joypad](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/src/bus/controller.rs#L41). | Controller-2 routing reproduced. Joypad2 is documented as TODO, but a public setter already exists. Wire both ports and latch on the strobe transition. |
| F01 / P1 | Default web playback executes one NES frame per display animation callback, with no elapsed-time budget. Audio may independently advance additional emulation on underrun. [Loop](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/web/ui/src/main.ts#L329). | Inspection: a 120 Hz display requests roughly twice the intended frame count. Use one authoritative emulation clock and display completed frames independently. This is already noted in README but affects the default mode. |
| F02 / P2 | Restoring control bindings updates a shadowing function argument and reverse maps, while leaving `.ref`/serialization attached to the old object. Old reverse bindings are never cleared. [Controls update](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/web/ui/src/controls.ts#L127). | Two Node checks reproduce stale serialized bindings and the old key still responding after restoration. Replace/copy the authoritative preferences and rebuild lookup maps. |
| F03 / P2 | Axis input updates only the current sign, so returning a negative axis to zero can leave its button held. Disconnect only clears the controller index. [Axes](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/web/ui/src/controls.ts#L302), [disconnect](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/web/ui/src/controls.ts#L363). | Two Node checks reproduce stuck LEFT and stuck B. Recompute gamepad state from zero each poll, evaluate both signs, and clear state on disconnect/blur. |
| F04 / P2 | The `input` hook's second argument is an optional gamepad index, but main treats it as a pressed boolean. Keyboard calls omit it and gamepad button 0 supplies zero, so these UI actions are dropped. [Hook consumer](https://github.com/nathsou/nessy/blob/7a24b064bb2c608f1b9047fe679a339406ef09a1/web/ui/src/main.ts#L133). | Inspection of both producers and consumer. Make event semantics explicit; do not use truthiness of an input index. CTRL shortcuts shown in the UI also only test the Meta key in the handler. |

The battery flag is parsed, but there is no standalone battery-backed RAM import/export lifecycle in the core; web persistence instead depends on complete save states. This is a feature/support gap rather than evidence of lost RAM during ordinary mapper reads. The workflow builds/deploys the web app but does not run core conformance tests. Add the passing checks to CI; enable each known-defect regression when repaired.

**Repair sequence and acceptance criteria**

1. Fix persistent mapper/APU IRQ levels and the false five-step APU IRQ. Correct BRK return PC and hardware-interrupt stacked B. Promote those regressions to normal tests.
2. Recompute MMC3 banking on `$8000` and `$8001`; restore mirroring in save states. These are localized changes with direct reproductions.
3. Introduce cycle-ordered CPU bus accesses and PPU fetch events, then qualified MMC3 A12 clocking. Validate the selected MMC3 revision, pre-render behavior, alternate pattern tables, and CPU-triggered address transitions. Avoid tuning a single dot constant until SMB3 happens to look right.
4. Complete saved emulated state and run deterministic save/advance/load/replay comparisons, including preview generation. Add robust ROM/save decoding and fix mapper bounds.
5. Complete unofficial instructions, DMA arbitration, PPU/APU edge cases, and frontend timing/controls with the corresponding suites.

For SMB3 validation, capture the exact ROM hash/region, whether the issue occurs from a cold boot or loaded save, the scene, and expected versus actual frame/split position. Trace PPU scanline/dot, A12 edges, IRQ assertion/acknowledgment, CPU I/PC, and writes to `$8000`, `$8001`, `$C000`, `$C001`, `$E000`, `$E001`, `$2000`, `$2005`, and `$2006`. The three leading core failures have independent evidence; the exact scene attribution remains open.

**Reproducing the historical baseline**

The commands and ignored-test status in this section describe the initial audit harness. The current branch enables the repaired regressions; use [fixes.md](fixes.md) for current commands.

```sh
cargo test --all-targets --locked
cargo test --test audit --locked -- --ignored
node tests/audit_controls.mjs
cargo clippy --all-targets --locked
```

The second and third commands intentionally exit with failures until the listed defects are repaired. The overflow reproduction is debug-specific; do not interpret its release pass as a fix. The first command's green result covers the three passing baselines, not the ignored defect checks.

To rerun independent ROMs, obtain the pinned public collection, then use [audit_rom.rs](../../examples/audit_rom.rs):

```sh
git clone https://github.com/christopherpow/nes-test-roms.git /tmp/nessy-test-roms
git -C /tmp/nessy-test-roms checkout 95d8f621ae55cee0d09b91519a8989ae0e64753b
cargo run --release --locked --example audit_rom -- /tmp/nessy-test-roms/mmc3_test_2/rom_singles/*.nes
```

The runner supports the `$6000` status protocol, bounds execution, and reports reset requests/timeouts separately. ROM files are not added to this repository. Raw logs preserve each test's own result text; higher-level suites can fail because of shared peripheral defects before reaching the feature named by the ROM.
