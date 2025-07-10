#[cfg(test)]
mod tests {
    use crate::cpu::mappers::mmc3::MMC3;
    use crate::cpu::mappers::Mapper;
    use crate::cpu::rom::{Cart, Mirroring};

    fn create_test_cart() -> Cart {
        Cart {
            bytes: vec![0; 0x10000], // 64KB ROM
            hash: [0; 32],
            prg_rom_size: 2, // 32KB
            chr_rom_size: 1, // 8KB
            mirroring: Mirroring::Horizontal,
            mapper_id: 4,
            battery: false,
            trainer: false,
            prg_rom_start: 16,
            chr_rom_start: 0x8000,
        }
    }

    #[test]
    fn test_mmc3_initialization() {
        let cart = create_test_cart();
        let mut mmc3 = MMC3::new(&cart);
        
        // Test that it doesn't panic and can be created
        assert!(!mmc3.is_asserting_irq());
    }

    #[test]
    fn test_mmc3_irq_assertions() {
        let cart = create_test_cart();
        let mut mmc3 = MMC3::new(&cart);
        
        // Initially should not assert IRQ
        assert!(!mmc3.is_asserting_irq());
        
        // Test IRQ enable/disable
        mmc3.write(&mut create_test_cart(), 0xE001, 0x01); // Enable IRQ
        mmc3.write(&mut create_test_cart(), 0xE000, 0x00); // Disable IRQ
        
        assert!(!mmc3.is_asserting_irq());
    }

    #[test]
    fn test_mmc3_prg_ram_access() {
        let cart = create_test_cart();
        let mut mmc3 = MMC3::new(&cart);
        let mut cart = create_test_cart();
        
        // Test PRG RAM write/read
        mmc3.write(&mut cart, 0x6000, 0x42);
        assert_eq!(mmc3.read(&mut cart, 0x6000), 0x42);
        
        mmc3.write(&mut cart, 0x7FFF, 0x84);
        assert_eq!(mmc3.read(&mut cart, 0x7FFF), 0x84);
    }

    #[test]
    fn test_mmc3_bank_register_writes() {
        let cart = create_test_cart();
        let mut mmc3 = MMC3::new(&cart);
        let mut cart = create_test_cart();
        
        // Test bank select register
        mmc3.write(&mut cart, 0x8000, 0x06); // Select PRG bank 0
        mmc3.write(&mut cart, 0x8001, 0x02); // Set bank to 2
        
        // Test mirroring
        mmc3.write(&mut cart, 0xA000, 0x01); // Horizontal mirroring
        assert_eq!(cart.mirroring, Mirroring::Horizontal);
        
        mmc3.write(&mut cart, 0xA000, 0x00); // Vertical mirroring
        assert_eq!(cart.mirroring, Mirroring::Vertical);
    }

    #[test]
    fn test_mmc3_irq_registers() {
        let cart = create_test_cart();
        let mut mmc3 = MMC3::new(&cart);
        let mut cart = create_test_cart();
        
        // Test IRQ reload register
        mmc3.write(&mut cart, 0xC000, 0x05); // Set reload value
        mmc3.write(&mut cart, 0xC001, 0x00); // Clear counter
        
        // Test IRQ enable/disable
        mmc3.write(&mut cart, 0xE001, 0x01); // Enable IRQ
        mmc3.write(&mut cart, 0xE000, 0x00); // Disable IRQ
        
        // Should not assert IRQ when disabled
        assert!(!mmc3.is_asserting_irq());
    }
}