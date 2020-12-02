use super::opcodes::{INST_ADDR_MODE_NAMES, INST_LENGTHS};
use std::collections::HashMap;
use std::u8;

#[derive(Debug)]
struct InstInfo {
    name: String,
    line_nb: usize,
    opcode: u8,
    mode: usize,
    arg: String,
}

pub struct Program {
    pub bytes: Vec<u8>,
    pub addr: usize,
}

pub struct Assembler {
    opcodes: HashMap<String, [Option<u8>; 12]>,
    labels: HashMap<String, usize>,
    program_counter: usize,
    instructions: Vec<InstInfo>,
    current_line: usize,
}

impl Assembler {
    pub fn new() -> Assembler {
        let mut opcodes = HashMap::new();
        /* Name, IMM, ZP,   ZPX,  ZPY,  ABS,  ABSX, ABSY, IND,  INDX, INDY, IMP,  REL */
        opcodes.insert(
            "ADC".to_string(),
            [
                Some(0x69),
                Some(0x65),
                Some(0x75),
                None,
                Some(0x6d),
                Some(0x7d),
                Some(0x79),
                None,
                Some(0x61),
                Some(0x71),
                None,
                None,
            ],
        );
        opcodes.insert(
            "AND".to_string(),
            [
                Some(0x29),
                Some(0x25),
                Some(0x35),
                None,
                Some(0x2d),
                Some(0x3d),
                Some(0x39),
                None,
                Some(0x21),
                Some(0x31),
                None,
                None,
            ],
        );
        opcodes.insert(
            "ASL".to_string(),
            [
                None,
                Some(0x06),
                Some(0x16),
                None,
                Some(0x0e),
                Some(0x1e),
                None,
                None,
                None,
                None,
                Some(0x0a),
                None,
            ],
        );
        opcodes.insert(
            "BIT".to_string(),
            [
                None,
                Some(0x24),
                None,
                None,
                Some(0x2c),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        );
        opcodes.insert(
            "BPL".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x10),
            ],
        );
        opcodes.insert(
            "BMI".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x30),
            ],
        );
        opcodes.insert(
            "BVC".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x50),
            ],
        );
        opcodes.insert(
            "BVS".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x70),
            ],
        );
        opcodes.insert(
            "BCC".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x90),
            ],
        );
        opcodes.insert(
            "BCS".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xb0),
            ],
        );
        opcodes.insert(
            "BNE".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xd0),
            ],
        );
        opcodes.insert(
            "BEQ".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xf0),
            ],
        );
        opcodes.insert(
            "BRK".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x00),
                None,
            ],
        );
        opcodes.insert(
            "CMP".to_string(),
            [
                Some(0xc9),
                Some(0xc5),
                Some(0xd5),
                None,
                Some(0xcd),
                Some(0xdd),
                Some(0xd9),
                None,
                Some(0xc1),
                Some(0xd1),
                None,
                None,
            ],
        );
        opcodes.insert(
            "CPX".to_string(),
            [
                Some(0xe0),
                Some(0xe4),
                None,
                None,
                Some(0xec),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        );
        opcodes.insert(
            "CPY".to_string(),
            [
                Some(0xc0),
                Some(0xc4),
                None,
                None,
                Some(0xcc),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        );
        opcodes.insert(
            "DEC".to_string(),
            [
                None,
                Some(0xc6),
                Some(0xd6),
                None,
                Some(0xce),
                Some(0xde),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        );
        opcodes.insert(
            "EOR".to_string(),
            [
                Some(0x49),
                Some(0x45),
                Some(0x55),
                None,
                Some(0x4d),
                Some(0x5d),
                Some(0x59),
                None,
                Some(0x41),
                Some(0x51),
                None,
                None,
            ],
        );
        opcodes.insert(
            "CLC".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x18),
                None,
            ],
        );
        opcodes.insert(
            "SEC".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x38),
                None,
            ],
        );
        opcodes.insert(
            "CLI".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x58),
                None,
            ],
        );
        opcodes.insert(
            "SEI".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x78),
                None,
            ],
        );
        opcodes.insert(
            "CLV".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xb8),
                None,
            ],
        );
        opcodes.insert(
            "CLD".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xd8),
                None,
            ],
        );
        opcodes.insert(
            "SED".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xf8),
                None,
            ],
        );
        opcodes.insert(
            "INC".to_string(),
            [
                None,
                Some(0xe6),
                Some(0xf6),
                None,
                Some(0xee),
                Some(0xfe),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        );
        opcodes.insert(
            "JMP".to_string(),
            [
                None,
                None,
                None,
                None,
                Some(0x4c),
                None,
                None,
                Some(0x6c),
                Some(0x7c),
                None,
                None,
                None,
            ],
        );
        opcodes.insert(
            "JSR".to_string(),
            [
                None,
                None,
                None,
                None,
                Some(0x20),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        );
        opcodes.insert(
            "LDA".to_string(),
            [
                Some(0xa9),
                Some(0xa5),
                Some(0xb5),
                None,
                Some(0xad),
                Some(0xbd),
                Some(0xb9),
                None,
                Some(0xa1),
                Some(0xb1),
                None,
                None,
            ],
        );
        opcodes.insert(
            "LDX".to_string(),
            [
                Some(0xa2),
                Some(0xa6),
                None,
                Some(0xb6),
                Some(0xae),
                None,
                Some(0xbe),
                None,
                None,
                None,
                None,
                None,
            ],
        );
        opcodes.insert(
            "LDY".to_string(),
            [
                Some(0xa0),
                Some(0xa4),
                Some(0xb4),
                None,
                Some(0xac),
                Some(0xbc),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        );
        opcodes.insert(
            "LSR".to_string(),
            [
                None,
                Some(0x46),
                Some(0x56),
                None,
                Some(0x4e),
                Some(0x5e),
                None,
                None,
                None,
                None,
                Some(0x4a),
                None,
            ],
        );
        opcodes.insert(
            "NOP".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xea),
                None,
            ],
        );
        opcodes.insert(
            "ORA".to_string(),
            [
                Some(0x09),
                Some(0x05),
                Some(0x15),
                None,
                Some(0x0d),
                Some(0x1d),
                Some(0x19),
                None,
                Some(0x01),
                Some(0x11),
                None,
                None,
            ],
        );
        opcodes.insert(
            "TAX".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xaa),
                None,
            ],
        );
        opcodes.insert(
            "TXA".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x8a),
                None,
            ],
        );
        opcodes.insert(
            "DEX".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xca),
                None,
            ],
        );
        opcodes.insert(
            "INX".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xe8),
                None,
            ],
        );
        opcodes.insert(
            "TAY".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xa8),
                None,
            ],
        );
        opcodes.insert(
            "TYA".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x98),
                None,
            ],
        );
        opcodes.insert(
            "DEY".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x88),
                None,
            ],
        );
        opcodes.insert(
            "INY".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xc8),
                None,
            ],
        );
        opcodes.insert(
            "ROR".to_string(),
            [
                None,
                Some(0x66),
                Some(0x76),
                None,
                Some(0x6e),
                Some(0x7e),
                None,
                None,
                None,
                None,
                Some(0x6a),
                None,
            ],
        );
        opcodes.insert(
            "ROL".to_string(),
            [
                None,
                Some(0x26),
                Some(0x36),
                None,
                Some(0x2e),
                Some(0x3e),
                None,
                None,
                None,
                None,
                Some(0x2a),
                None,
            ],
        );
        opcodes.insert(
            "RTI".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x40),
                None,
            ],
        );
        opcodes.insert(
            "RTS".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x60),
                None,
            ],
        );
        opcodes.insert(
            "SBC".to_string(),
            [
                Some(0xe9),
                Some(0xe5),
                Some(0xf5),
                None,
                Some(0xed),
                Some(0xfd),
                Some(0xf9),
                None,
                Some(0xe1),
                Some(0xf1),
                None,
                None,
            ],
        );
        opcodes.insert(
            "STA".to_string(),
            [
                None,
                Some(0x85),
                Some(0x95),
                None,
                Some(0x8d),
                Some(0x9d),
                Some(0x99),
                None,
                Some(0x81),
                Some(0x91),
                None,
                None,
            ],
        );
        opcodes.insert(
            "TXS".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x9a),
                None,
            ],
        );
        opcodes.insert(
            "TSX".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0xba),
                None,
            ],
        );
        opcodes.insert(
            "PHA".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x48),
                None,
            ],
        );
        opcodes.insert(
            "PLA".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x68),
                None,
            ],
        );
        opcodes.insert(
            "PHP".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x08),
                None,
            ],
        );
        opcodes.insert(
            "PLP".to_string(),
            [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0x28),
                None,
            ],
        );
        opcodes.insert(
            "STX".to_string(),
            [
                None,
                Some(0x86),
                None,
                Some(0x96),
                Some(0x8e),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        );
        opcodes.insert(
            "STY".to_string(),
            [
                None,
                Some(0x84),
                Some(0x94),
                None,
                Some(0x8c),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        );

        Assembler {
            opcodes,
            labels: HashMap::new(),
            program_counter: 0x0000,
            instructions: vec![],
            current_line: 0,
        }
    }

    fn check_label(&mut self, line: &str) -> bool {
        if line.chars().last().unwrap() == ':' {
            let label = line[0..line.len() - 1].to_string();
            // println!("found a label: {}", label);
            self.labels.insert(label, self.current_line);

            return true;
        }

        self.current_line += 1;

        false
    }

    fn check_immediate(&self, mode: &str) -> bool {
        mode.len() > 2 && &mode[0..1] == "#"
    }

    fn check_zp(&self, mode: &str) -> bool {
        mode.len() > 1 && mode.len() <= 3 && &mode[0..1] == "$"
    }

    fn check_abs(&self, mode: &str) -> bool {
        mode.len() == 5 && &mode[0..1] == "$"
    }

    fn check_implied(&self, mode: &str) -> bool {
        mode.len() == 0
    }

    fn check_relative(&self, inst_name: &str) -> bool {
        inst_name == "BCC"
            || inst_name == "BCS"
            || inst_name == "BEQ"
            || inst_name == "BMI"
            || inst_name == "BNE"
            || inst_name == "BPL"
            || inst_name == "BVS"
    }

    fn get_addressing_mode(&mut self, arg: &str, inst_name: &str) -> Option<u8> {
        if self.labels.contains_key(&arg.to_string()) {
            if self.check_relative(inst_name) {
                return Some(11); // Relative
            } else {
                return Some(4); // Absolute
            }
        }

        if self.check_immediate(arg) {
            return Some(0);
        }

        if self.check_zp(arg) {
            return Some(1);
        }

        if self.check_abs(arg) {
            return Some(4);
        }

        if self.check_implied(arg) {
            return Some(10);
        }

        if self.check_relative(inst_name) {
            return Some(11);
        }

        None
    }

    fn parse_u8(&self, num: &str) -> u8 {
        if &num[0..1] == "$" {
            u8::from_str_radix(&num[1..num.len()], 16).unwrap()
        } else {
            u8::from_str_radix(&num[0..num.len()], 10).unwrap()
        }
    }

    fn parse_u16(&self, num: &str) -> [u8; 2] {
        if &num[0..1] == "$" {
            [
                u8::from_str_radix(&num[3..5], 16).unwrap(),
                u8::from_str_radix(&num[1..3], 16).unwrap(),
            ]
        } else {
            [
                u8::from_str_radix(&num[2..4], 10).unwrap(),
                u8::from_str_radix(&num[0..2], 10).unwrap(),
            ]
        }
    }

    fn parse_immediate(&self, arg: &str) -> Vec<u8> {
        vec![self.parse_u8(&arg[1..arg.len()])]
    }

    fn parse_zp(&self, arg: &str) -> Vec<u8> {
        vec![self.parse_u8(arg)]
    }

    fn is_label(&self, arg: &str) -> bool {
        self.labels.contains_key(arg)
    }

    fn parse_abs(&self, arg: &str) -> Vec<u8> {
        if self.is_label(arg) {
            let addr = self.labels.get(arg).unwrap();
            vec![(addr & 0xff) as u8, ((addr >> 8) as u8) & 0xff]
        } else {
            let mut bytes = vec![];
            bytes.extend_from_slice(&self.parse_u16(arg));

            bytes
        }
    }

    fn parse_rel(&self, arg: &str) -> Vec<u8> {
        let addr: u16;

        if self.is_label(arg) {
            addr = *self.labels.get(arg).unwrap() as u16;
        } else {
            panic!("Relative addressing mode not implemented when not using labels");
        };

        let delta = (addr as i16) - (self.program_counter as i16) - 2;

        if delta > 127 || delta < -128 {
            panic!("Unreachable label: {}", arg);
        }

        vec![delta as u8]
    }

    fn parse_inst(&self, inst: &InstInfo) -> Vec<u8> {
        let mut bytes = vec![inst.opcode];

        bytes.append(
            &mut (match inst.mode {
                0 => self.parse_immediate(&inst.arg),
                1 => self.parse_zp(&inst.arg),
                4 => self.parse_abs(&inst.arg),
                10 => vec![], // Implied
                11 => self.parse_rel(&inst.arg),
                _ => panic!(
                    "Unhandled addressing mode: {}",
                    INST_ADDR_MODE_NAMES[inst.mode]
                ),
            }),
        );

        bytes
    }

    fn preprocess_line(&mut self, line: &str) {
        let inst_name = &line[0..3];
        let arg = line[3..line.len()].trim();
        let mode = self
            .get_addressing_mode(arg, inst_name)
            .expect(&format!("unknown addressing mode for '{}'", arg)) as usize;
        let opcode = self.opcodes.get(inst_name).unwrap()[mode].expect(&format!(
            "Invalid mode {} for instruction {}",
            INST_ADDR_MODE_NAMES[mode as usize], inst_name
        ));

        let line_nb = self.instructions.len();

        self.instructions.push(InstInfo {
            name: inst_name.to_string(),
            line_nb,
            opcode,
            mode,
            arg: arg.to_string(),
        });

        // let mut bytes = vec![opcode];
        // bytes.append(&mut self.parse_arg(mode, arg));

        // println!(
        //     "{} -> {} : {}",
        //     line,
        //     INST_ADDR_MODE_NAMES[mode as usize],
        //     bytes
        //         .iter()
        //         .map(|b| format!("{:02x}", b))
        //         .collect::<Vec<_>>()
        //         .join(" ")
        // );
    }

    fn replace_labels(&mut self) {
        let mut pc = self.program_counter;
        let mut line_addresses = vec![];

        for inst in &self.instructions {
            line_addresses.push(pc);
            pc += INST_LENGTHS[inst.opcode as usize] as usize
        }

        let mut labels_addresses = HashMap::new();

        for (key, val) in &self.labels {
            labels_addresses.insert(key.to_string(), line_addresses[*val]);
        }

        self.labels = labels_addresses;
    }

    pub fn assemble(&mut self, asm: &str, program_counter: usize) -> Program {
        self.program_counter = program_counter;
        let mut bytes = vec![];
        let lines: Vec<&str> = asm
            .split("\n")
            .map(|line| line.trim())
            .filter(|line| line.len() != 0 && !self.check_label(line))
            .collect();

        println!("{:?}", self.labels);

        for line in lines {
            self.preprocess_line(line);
        }

        self.replace_labels();

        for inst in &self.instructions {
            bytes.append(&mut self.parse_inst(inst));
            self.program_counter += INST_LENGTHS[inst.opcode as usize] as usize;
        }

        // println!(
        //     "{}",
        //     bytes
        //         .iter()
        //         .map(|b| format!("{:02x}", b))
        //         .collect::<Vec<_>>()
        //         .join(" ")
        // );

        Program {
            bytes,
            addr: program_counter,
        }
    }
}
