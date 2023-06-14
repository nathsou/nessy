use std::u8;

pub fn parse_hex(hex: &str) -> Vec<u8> {
    hex.split(' ')
        .map(|h| u8::from_str_radix(&h, 16).unwrap())
        .collect::<Vec<_>>()
}
