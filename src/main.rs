use std::{any::type_name, io};



struct ProcessorStatus {
    flags: [bool; 8],
    //0: C carry
    //1: Z zero
    //2: I interrupt disable
    //3: D decimal mode
    //4: B break command
    //5: V overflow flag
    //6: N negative_flag
}

fn log_error(tag: &str, error: &str) {
    println!("{}: {}", tag, error);
}

fn compl2_is_pos(byte: u8) -> bool {
    return byte & 0b10000000 > 0;
}

fn compl2_to_abs(byte: u8) -> u8 {
    return (byte & 0b01111111) - byte >> 7;
}

fn compl2_greater_abs(byte_a: u8, byte_b: u8) -> bool {
    return compl2_to_abs(byte_a) > compl2_to_abs(byte_b);
}


impl ProcessorStatus {
    fn set_carry(&mut self, bit: bool) {
        self.flags[0] = bit;
    }
    fn get_carry(&mut self) -> bool {
        return self.flags[0];
    }

    fn set_zero(&mut self, bit: bool) {
        self.flags[1] = bit;
    }
    fn get_zero(&mut self) -> bool {
        return self.flags[1];
    }

    fn set_interrupt(&mut self, bit: bool) {
        self.flags[2] = bit;
    }
    fn get_interrupt(&mut self) -> bool {
        return self.flags[2];
    }

    fn set_decimal(&mut self, bit: bool) {
        self.flags[3] = bit;
    }
    fn get_decimal(&mut self) -> bool {
        return self.flags[3];
    }

    fn set_break(&mut self, bit: bool) {
        self.flags[4] = bit;
    }
    fn get_break(&mut self) -> bool {
        return self.flags[4];
    }

    fn set_overflow(&mut self, bit: bool) {
        self.flags[5] = bit;
    }
    fn get_overflow(&mut self) -> bool {
        return self.flags[5];
    }

    fn set_negative(&mut self, bit: bool) {
        self.flags[6] = bit;
    }
    fn get_negative(&mut self) -> bool {
        return self.flags[6];
    }
}

const STACK_ADDRESS_MSB: u16 = 0x0100;
const ZEROPAGE_ADDRESS_MSB: u16 = 0x0000;
const MEMORY_SIZE: usize = 0x10000;

enum AddressingModes {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,

    _INVALID
}

struct ADDRESSER {
    mode: AddressingModes,
    low_address: u16,
    high_adress: u16,
    full_address: u16,
    offset: u8,
    memory: [u8; MEMORY_SIZE],
    implicit: bool,
    accumulator: bool,
    immediate: bool,
    relative: bool,

}

impl ADDRESSER {
    fn reset_flags(&mut self) {
        self.implicit = false;
        self.accumulator = false;
        self.immediate = false;
        self.relative = false;
    }

    fn reset_addresses(&mut self) {
        self.low_address = 0;
        self.high_adress = 0;
        self.full_address = 0;
        self.offset = 0;
    }


    fn set_offset(&mut self, offset: u8) {
        self.offset = offset;
    }
    fn set_low(&mut self, low_addr: u8) {
        self.low_address = low_addr as u16;
    }
    fn set_high(&mut self, high_addr: u8) {
        self.high_adress = (high_addr as u16) << 8;
    }
    fn set_mode(&mut self, mode: AddressingModes) {
        self.mode = mode;
    }
    fn set_full(&mut self, full_addr: u16) {
        self.full_address = full_addr;
    }

    fn get_fulladdress(&self) -> u16 {
        return self.full_address;
    }


    fn zeropage(&mut self) {
        self.full_address = ZEROPAGE_ADDRESS_MSB | (self.low_address as u8).wrapping_add(self.offset) as u16;
    }
    fn absolute(&mut self) {
        self.full_address = self.full_address.wrapping_add(self.offset as u16);
    }
    fn indirect(&mut self) {
        let new_address: u16 = self.memory[self.full_address as usize] as u16 |
            ((self.memory[(self.full_address.wrapping_add(1)) as usize] as u16) << 8);

        self.full_address = new_address;
    }

    fn indexed_indirect(&mut self) {
        self.full_address = self.high_adress | (self.low_address as u8).wrapping_add(self.offset) as u16;
        return self.indirect();
    }

    fn indirect_indexed(&mut self) {
        self.full_address = (self.high_adress | self.low_address).wrapping_add(self.offset as u16);
        return self.indirect();
    }

    fn calc_address(&mut self, low_addr: u8, high_addr: u8, full_addr: u16, offset: u8, mode: AddressingModes) {
        self.reset_flags();

        self.set_low(low_addr);
        self.set_high(high_addr);
        self.set_full(full_addr);
        self.set_offset(offset);
        self.set_mode(mode);

        match self.mode {
            AddressingModes::Implicit => {
                self.implicit = true;
            },
            AddressingModes::Accumulator => {
                self.accumulator = true;
            },
            AddressingModes::Immediate => {
                self.immediate = true;
            },
            AddressingModes::ZeroPage => {
                self.zeropage();
            },
            AddressingModes::ZeroPageX => {
                self.zeropage();
            },
            AddressingModes::ZeroPageY => {
                self.zeropage();
            },
            AddressingModes::Relative => {
                self.relative = true;
            },
            AddressingModes::Absolute => {
                self.absolute();
            },
            AddressingModes::AbsoluteX => {
                self.absolute();
            },
            AddressingModes::AbsoluteY => {
                self.absolute();
            },
            AddressingModes::Indirect => {
                self.indirect();
            },
            AddressingModes::IndexedIndirect => {
                self.indexed_indirect();
            },
            AddressingModes::IndirectIndexed => {
                self.indirect_indexed();
            },
            AddressingModes::_INVALID => {
                log_error("Get Address", "mode is invalid");

            }
        }
    }

    fn deref_byte(&mut self, index: usize) -> Result<u8, io::Error> {
        if index >= MEMORY_SIZE {
            log_error("get_memory", "Index is greater than memory");
            return Result::Err(io::Error::new(io::ErrorKind::InvalidInput, "index is greater than memory size"))
        }
        return Result::Ok(self.memory[index]);
    }
    fn deref_word(&mut self, index: usize) -> Result<u16, io::Error> {
        let idx_plus_one = (index as u16).wrapping_add(1) as usize;
        let word: u16 = self.deref_byte(index)? as u16 + (self.deref_byte(idx_plus_one)? as u16) << 8;

        return Ok(word);
    }

    fn calc_address_and_deref_byte(&mut self, low_addr: u8, high_addr: u8, full_addr: u16, offset: u8, mode: AddressingModes) -> Result<u8, io::Error> {
        self.calc_address(low_addr, high_addr, full_addr, offset, mode);
        let byte = self.deref_byte(self.full_address as usize)?;
        return Ok(byte);
    }

    fn calc_address_and_deref_word(&mut self, low_addr: u8, high_addr: u8, full_addr: u16, offset: u8, mode: AddressingModes) -> Result<u16, io::Error> {
        self.calc_address(low_addr, high_addr, full_addr, offset, mode);
        let word = self.deref_word(self.full_address as usize)?;
        return Ok(word);
    }



}

struct Cpu6502 {
    pc: u16, //program counter
    sc: u16, //stack counter. It only holds the LSB since stack is between $0x0100 and $0x01ff. Declared as u16 for ease of use
    //6502 cpu DOES NOT DETECT STACK OVERFLOW
    //push to stack = sc decremented.
    accu: u8, //accumulator register
    idx_x: u8, //index register X
    idx_y: u8, //index register y
    status: ProcessorStatus,
    addresser: ADDRESSER,

}

impl Cpu6502 {

    fn inst_adc(&mut self, byte: u8) { //ADd with Carry A, Z, C, N = A + M + C
        let mut carry: bool = false;

        let expected_pos: bool = (compl2_is_pos(self.accu) && compl2_is_pos(byte)) ||
            (compl2_is_pos(byte) && compl2_greater_abs(byte, self.accu)) ||
            (compl2_is_pos(self.accu) && compl2_greater_abs(self.accu, byte));

        if (self.accu as u16 + byte as u16 + self.status.get_carry() as u16) > 0xFF {
            carry = true;
        }
        self.accu = self.accu.wrapping_add(byte).wrapping_add(self.status.get_carry() as u8);

        self.status.set_carry(carry);
        self.status.set_zero(self.accu == 0);
        self.status.set_overflow(compl2_is_pos(self.accu) == expected_pos);
        self.status.set_negative(!compl2_is_pos(self.accu));
    }
}


fn main() {
    println!("Hello, world!");
}