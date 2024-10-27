use std::{any::type_name, io};

use crate::logging::{log_error, log_warning, log_info};


#[derive(Debug)]
pub enum AddressingModes {
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
    XInd,
    IndY,

    _INVALID
}


const STACK_ADDRESS_MSB: u16 = 0x0100;
const ZEROPAGE_ADDRESS_MSB: u16 = 0x0000;
const MEMORY_SIZE: usize = 0x10000;

pub struct ADDRESSER {
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

    bytes_to_pull: usize,

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
        self.bytes_to_pull = 0;
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

    pub fn calc_address(&mut self, low_addr: u8, high_addr: u8, full_addr: u16, offset: u8, mode: AddressingModes) {
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
            AddressingModes::XInd => {
                self.indexed_indirect();
            },
            AddressingModes::IndY => {
                self.indirect_indexed();
            },
            AddressingModes::_INVALID => {
                log_error("Get Address", "mode is invalid");
            }
        }
    }

    pub fn deref_byte(&self, index: usize) -> Result<u8, io::Error> {
        if index >= MEMORY_SIZE {
            log_error("get_memory", "Index is greater than memory");
            return Result::Err(io::Error::new(io::ErrorKind::InvalidInput, "index is greater than memory size"))
        }
        return Result::Ok(self.memory[index]);
    }
    pub fn deref_word(&self, index: usize) -> Result<u16, io::Error> {
        let idx_plus_one = (index as u16).wrapping_add(1) as usize;
        let word: u16 = self.deref_byte(index)? as u16 + (self.deref_byte(idx_plus_one)? as u16) << 8;

        return Ok(word);
    }
    pub fn deref_n_bytes(&self, index: usize, num_bytes: usize) -> Result<u32, io::Error> {
        let mut result: u32 = 0;
        for byte in 0..num_bytes {
            result |= (self.deref_byte(index + byte)? as u32) << (8 * byte);
        }
        return Ok(result);
    }

    pub fn calc_address_and_deref_byte(&mut self, low_addr: u8, high_addr: u8, full_addr: u16, offset: u8, mode: AddressingModes) -> Result<u8, io::Error> {
        self.calc_address(low_addr, high_addr, full_addr, offset, mode);
        let byte = self.deref_byte(self.full_address as usize)?;
        return Ok(byte);
    }

    pub fn calc_address_and_deref_word(&mut self, low_addr: u8, high_addr: u8, full_addr: u16, offset: u8, mode: AddressingModes) -> Result<u16, io::Error> {
        self.calc_address(low_addr, high_addr, full_addr, offset, mode);
        let word = self.deref_word(self.full_address as usize)?;
        return Ok(word);
    }

    pub fn get_opcode_arguments(&mut self, mode: AddressingModes, pc: usize, pulled_bytes: &usize) -> Result<u32, io::Error>{
        match self.mode {
            AddressingModes::Implicit => {
                self.bytes_to_pull = 0;
            },
            AddressingModes::Accumulator => {
                self.bytes_to_pull = 0;
            },
            AddressingModes::Immediate => {
                self.bytes_to_pull = 1;
            },
            AddressingModes::ZeroPage => {
                self.bytes_to_pull = 1;
            },
            AddressingModes::ZeroPageX => {
                self.bytes_to_pull = 1;
            },
            AddressingModes::ZeroPageY => {
                self.bytes_to_pull = 1;
            },
            AddressingModes::Relative => {
                self.bytes_to_pull = 1;
            },
            AddressingModes::Absolute => {
                self.bytes_to_pull = 2;
            },
            AddressingModes::AbsoluteX => {
                self.bytes_to_pull = 2;
            },
            AddressingModes::AbsoluteY => {
                self.bytes_to_pull = 2;
            },
            AddressingModes::Indirect => {
                self.bytes_to_pull = 1;
            },
            AddressingModes::XInd => {
                self.bytes_to_pull = 1;
            },
            AddressingModes::IndY => {
                self.bytes_to_pull = 1;
            },
            AddressingModes::_INVALID => {
                self.bytes_to_pull = 0;
            }
        }
        if self.bytes_to_pull > 0 {
            let args = self.deref_n_bytes(pc + 1, self.bytes_to_pull)?;

            return Ok(args);
        }

        return Ok(0);
    }



}