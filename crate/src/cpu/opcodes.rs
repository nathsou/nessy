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

pub const INST_LENGTHS: [u8; 256] = [
    1, 2, 0, 0, 0, 2, 2, 0, 1, 2, 1, 0, 0, 3, 3, 0, 2, 2, 0, 0, 0, 2, 2, 0, 1, 3, 0, 0, 0, 3, 3, 0,
    3, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 0, 2, 2, 0, 1, 3, 0, 0, 0, 3, 3, 0,
    1, 2, 0, 0, 0, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 0, 2, 2, 0, 1, 3, 0, 0, 0, 3, 3, 0,
    1, 2, 0, 0, 0, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 0, 2, 2, 0, 1, 3, 0, 0, 2, 3, 3, 0,
    0, 2, 0, 0, 2, 2, 2, 0, 1, 0, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 0, 3, 0, 0,
    2, 2, 2, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 0, 2, 2, 0, 1, 3, 0, 0, 0, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 0, 2, 2, 0, 1, 3, 0, 0, 0, 3, 3, 0,
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

impl Into<u8> for AddressingMode {
    fn into(self) -> u8 {
        use AddressingMode::*;

        match self {
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

pub const INST_ADDR_MODES: [u8; 256] = [
    10, 8, 0, 0, 0, 1, 1, 0, 10, 0, 10, 0, 0, 4, 4, 0, 11, 9, 0, 0, 0, 2, 2, 0, 10, 6, 0, 0, 0, 5,
    5, 0, 4, 8, 0, 0, 1, 1, 1, 0, 10, 0, 10, 0, 4, 4, 4, 0, 11, 9, 0, 0, 0, 2, 2, 0, 10, 6, 0, 0,
    0, 5, 5, 0, 10, 8, 0, 0, 0, 1, 1, 0, 10, 0, 10, 0, 4, 4, 4, 0, 11, 9, 0, 0, 0, 2, 2, 0, 10, 6,
    0, 0, 0, 5, 5, 0, 10, 8, 0, 0, 0, 1, 1, 0, 10, 0, 10, 0, 7, 4, 4, 0, 11, 9, 0, 0, 0, 2, 2, 0,
    10, 6, 0, 0, 8, 5, 5, 0, 0, 8, 0, 0, 1, 1, 1, 0, 10, 0, 10, 0, 4, 4, 4, 0, 11, 9, 0, 0, 2, 2,
    3, 0, 10, 6, 10, 0, 0, 5, 0, 0, 0, 8, 0, 0, 1, 1, 1, 0, 10, 0, 10, 0, 4, 4, 4, 0, 11, 9, 0, 0,
    2, 2, 3, 0, 10, 6, 10, 0, 5, 5, 6, 0, 0, 8, 0, 0, 1, 1, 1, 0, 10, 0, 10, 0, 4, 4, 4, 0, 11, 9,
    0, 0, 0, 2, 2, 0, 10, 6, 0, 0, 0, 5, 5, 0, 0, 8, 0, 0, 1, 1, 1, 0, 10, 0, 10, 0, 4, 4, 4, 0,
    11, 9, 0, 0, 0, 2, 2, 0, 10, 6, 0, 0, 0, 5, 5, 0,
];

#[rustfmt::skip]
pub const INST_NAMES: [Option<&str>; 256] = [
    Some("BRK"), Some("ORA"), None, None, None, Some("ORA"), Some("ASL"), None, Some("PHP"), Some("ORA"), Some("ASL"), None, None, Some("ORA"),
    Some("ASL"), None, Some("BPL"), Some("ORA"), None, None, None, Some("ORA"), Some("ASL"), None, Some("CLC"), Some("ORA"), None, None, None,
    Some("ORA"), Some("ASL"), None, Some("JSR"), Some("AND"), None, None, Some("BIT"), Some("AND"), Some("ROL"), None, Some("PLP"), Some("AND"),
    Some("ROL"), None, Some("BIT"), Some("AND"), Some("ROL"), None, Some("BMI"), Some("AND"), None, None, None, Some("AND"), Some("ROL"), None,
    Some("SEC"), Some("AND"), None, None, None, Some("AND"), Some("ROL"), None, Some("RTI"), Some("EOR"), None, None, None, Some("EOR"), Some("LSR"),
    None, Some("PHA"), Some("EOR"), Some("LSR"), None, Some("JMP"), Some("EOR"), Some("LSR"), None, Some("BVC"), Some("EOR"), None, None, None,
    Some("EOR"), Some("LSR"), None, Some("CLI"), Some("EOR"), None, None, None, Some("EOR"), Some("LSR"), None, Some("RTS"), Some("ADC"), None,
    None, None, Some("ADC"), Some("ROR"), None, Some("PLA"), Some("ADC"), Some("ROR"), None, Some("JMP"), Some("ADC"), Some("ROR"), None, Some("BVS"),
    Some("ADC"), None, None, None, Some("ADC"), Some("ROR"), None, Some("SEI"), Some("ADC"), None, None, Some("JMP"), Some("ADC"), Some("ROR"), None,
    None, Some("STA"), None, None, Some("STY"), Some("STA"), Some("STX"), None, Some("DEY"), None, Some("TXA"), None, Some("STY"), Some("STA"),
    Some("STX"), None, Some("BCC"), Some("STA"), None, None, Some("STY"), Some("STA"), Some("STX"), None, Some("TYA"), Some("STA"), Some("TXS"),
    None, None, Some("STA"), None, None, Some("LDY"), Some("LDA"), Some("LDX"), None, Some("LDY"), Some("LDA"), Some("LDX"), None, Some("TAY"),
    Some("LDA"), Some("TAX"), None, Some("LDY"), Some("LDA"), Some("LDX"), None, Some("BCS"), Some("LDA"), None, None, Some("LDY"), Some("LDA"),
    Some("LDX"), None, Some("CLV"), Some("LDA"), Some("TSX"), None, Some("LDY"), Some("LDA"), Some("LDX"), None, Some("CPY"), Some("CMP"), None,
    None, Some("CPY"), Some("CMP"), Some("DEC"), None, Some("INY"), Some("CMP"), Some("DEX"), None, Some("CPY"), Some("CMP"), Some("DEC"), None,
    Some("BNE"), Some("CMP"), None, None, None, Some("CMP"), Some("DEC"), None, Some("CLD"), Some("CMP"), None, None, None, Some("CMP"), Some("DEC"),
    None, Some("CPX"), Some("SBC"), None, None, Some("CPX"), Some("SBC"), Some("INC"), None, Some("INX"), Some("SBC"), Some("NOP"), None, Some("CPX"),
    Some("SBC"), Some("INC"), None, Some("BEQ"), Some("SBC"), None, None, None, Some("SBC"), Some("INC"), None, Some("SED"), Some("SBC"), None, None,
    None, Some("SBC"), Some("INC"), None,
];
