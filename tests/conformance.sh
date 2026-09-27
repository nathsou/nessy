#!/usr/bin/env bash
# Public hardware tests; commercial game ROMs are neither required nor bundled.
set -euo pipefail
roms="${1:?Pass the path to christopherpow/nes-test-roms}"
revision=95d8f621ae55cee0d09b91519a8989ae0e64753b
if [[ "$(git -C "$roms" rev-parse HEAD)" != "$revision" ]]; then
  echo "Expected test ROM revision $revision" >&2
  exit 1
fi
cargo run --release --locked --example audit_rom -- \
  "$roms"/instr_test-v5/{official_only,all_instrs}.nes \
  "$roms"/instr_misc/rom_singles/*.nes \
  "$roms"/cpu_dummy_writes/*.nes \
  "$roms"/apu_test/rom_singles/*.nes \
  "$roms"/mmc3_test_2/rom_singles/{1-clocking,2-details,3-A12_clocking,4-scanline_timing,5-MMC3}.nes \
  "$roms"/cpu_interrupts_v2/rom_singles/*.nes \
  "$roms"/ppu_vbl_nmi/rom_singles/*.nes
