pub const INST_CYCLES: [u32; 256] = [
    7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
];

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AddressingMode {
    Immediate = 0,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Implied,
    Relative,
}

impl From<u8> for AddressingMode {
    fn from(mode: u8) -> Self {
        use AddressingMode::*;
        match mode {
            0 => Immediate,
            1 => ZeroPage,
            2 => ZeroPageX,
            3 => ZeroPageY,
            4 => Absolute,
            5 => AbsoluteX,
            6 => AbsoluteY,
            7 => Indirect,
            8 => IndirectX,
            9 => IndirectY,
            10 => Implied,
            11 => Relative,
            _ => panic!("Invalid addressing mode: {}", mode),
        }
    }
}

impl From<AddressingMode> for u8 {
    fn from(mode: AddressingMode) -> Self {
        use AddressingMode::*;

        match mode {
            Immediate => 0,
            ZeroPage => 1,
            ZeroPageX => 2,
            ZeroPageY => 3,
            Absolute => 4,
            AbsoluteX => 5,
            AbsoluteY => 6,
            Indirect => 7,
            IndirectX => 8,
            IndirectY => 9,
            Implied => 10,
            Relative => 11,
        }
    }
}

impl std::fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AddressingMode::*;
        match self {
            Immediate => write!(f, "Immediate"),
            ZeroPage => write!(f, "ZeroPage"),
            ZeroPageX => write!(f, "ZeroPageX"),
            ZeroPageY => write!(f, "ZeroPageY"),
            Absolute => write!(f, "Absolute"),
            AbsoluteX => write!(f, "AbsoluteX"),
            AbsoluteY => write!(f, "AbsoluteY"),
            Indirect => write!(f, "Indirect"),
            IndirectX => write!(f, "IndirectX"),
            IndirectY => write!(f, "IndirectY"),
            Implied => write!(f, "Implied"),
            Relative => write!(f, "Relative"),
        }
    }
}
