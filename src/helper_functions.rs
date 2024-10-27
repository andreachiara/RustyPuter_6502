

pub fn compl2_is_pos(byte: u8) -> bool {
    return byte & 0b10000000 > 0;
}

pub fn compl2_to_abs(byte: u8) -> u8 {
    return (byte & 0b01111111) - byte >> 7;
}

pub fn compl2_greater_abs(byte_a: u8, byte_b: u8) -> bool {
    return compl2_to_abs(byte_a) > compl2_to_abs(byte_b);
}