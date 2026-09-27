Independent ROM results on unchanged production source. Raw diagnostics are in the adjacent logs.

| ROM | Result | CPU cycles to result |
|---|---|---:|
| `mmc3_test_2/rom_singles/1-clocking.nes` | FAIL 3 | 837636 |
| `mmc3_test_2/rom_singles/2-details.nes` | FAIL 2 | 868354 |
| `mmc3_test_2/rom_singles/3-A12_clocking.nes` | FAIL 4 | 837635 |
| `mmc3_test_2/rom_singles/4-scanline_timing.nes` | FAIL 3 | 2207744 |
| `mmc3_test_2/rom_singles/5-MMC3.nes` | FAIL 2 | 837633 |
| `mmc3_test_2/rom_singles/6-MMC3_alt.nes` | FAIL 2 | 898048 |
| `cpu_interrupts_v2/rom_singles/1-cli_latency.nes` | FAIL 3 | 540674 |
| `cpu_interrupts_v2/rom_singles/2-nmi_and_brk.nes` | FAIL 1 | 2863104 |
| `cpu_interrupts_v2/rom_singles/3-nmi_and_irq.nes` | FAIL 1 | 3429376 |
| `cpu_interrupts_v2/rom_singles/4-irq_and_dma.nes` | FAIL 1 | 2059265 |
| `cpu_interrupts_v2/rom_singles/5-branch_delays_irq.nes` | FAIL 1 | 4590592 |
| `instr_misc/rom_singles/01-abs_x_wrap.nes` | PASS | 328706 |
| `instr_misc/rom_singles/02-branch_wrap.nes` | PASS | 328706 |
| `instr_misc/rom_singles/03-dummy_reads.nes` | FAIL 3 | 567296 |
| `instr_misc/rom_singles/04-dummy_reads_apu.nes` | FAIL 2 | 2383874 |
| `apu_test/rom_singles/1-len_ctr.nes` | FAIL 4 | 570368 |
| `apu_test/rom_singles/2-len_table.nes` | FAIL 1 | 420864 |
| `apu_test/rom_singles/3-irq_flag.nes` | PASS | 600064 |
| `apu_test/rom_singles/4-jitter.nes` | FAIL 2 | 480256 |
| `apu_test/rom_singles/5-len_timing.nes` | FAIL 3 | 600065 |
| `apu_test/rom_singles/6-irq_flag_timing.nes` | FAIL 2 | 510976 |
| `apu_test/rom_singles/7-dmc_basics.nes` | FAIL 2 | 510978 |
| `apu_test/rom_singles/8-dmc_rates.nes` | FAIL 3 | 480256 |
| `ppu_vbl_nmi/rom_singles/01-vbl_basics.nes` | PANIC | 238591 |
| `ppu_vbl_nmi/rom_singles/02-vbl_set_time.nes` | FAIL 1 | 4798469 |
| `ppu_vbl_nmi/rom_singles/03-vbl_clear_time.nes` | PASS | 4768768 |
| `ppu_vbl_nmi/rom_singles/04-nmi_control.nes` | FAIL 11 | 1076225 |
| `ppu_vbl_nmi/rom_singles/05-nmi_timing.nes` | FAIL 1 | 6109185 |
| `ppu_vbl_nmi/rom_singles/06-suppression.nes` | FAIL 1 | 5811200 |
| `ppu_vbl_nmi/rom_singles/07-nmi_on_timing.nes` | FAIL 1 | 5155841 |
| `ppu_vbl_nmi/rom_singles/08-nmi_off_timing.nes` | FAIL 1 | 6794241 |
| `ppu_vbl_nmi/rom_singles/09-even_odd_frames.nes` | PASS | 2267136 |
| `ppu_vbl_nmi/rom_singles/10-even_odd_timing.nes` | FAIL 3 | 2357249 |
| `instr_test-v5/official_only.nes` | FAIL 1 | 2174976 |
| `instr_test-v5/rom_singles/12-jmp_jsr.nes` | FAIL 1 | 447490 |
| `instr_test-v5/rom_singles/13-rts.nes` | FAIL 1 | 388096 |
| `instr_test-v5/rom_singles/14-rti.nes` | FAIL 1 | 388096 |
| `instr_test-v5/rom_singles/15-brk.nes` | FAIL 1 | 775169 |
| `instr_test-v5/rom_singles/16-special.nes` | FAIL 6 | 480256 |
| `cpu_dummy_writes/cpu_dummy_writes_oam.nes` | PANIC | 5526008 |
| `cpu_dummy_writes/cpu_dummy_writes_ppumem.nes` | PANIC | 693675 |
