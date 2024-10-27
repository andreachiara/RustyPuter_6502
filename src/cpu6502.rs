use core::num;
use std::ops::Add;
use std::ptr::addr_eq;
use std::{any::type_name, io};


use crate::addresser::{AddressingModes, ADDRESSER};
use crate::helper_functions::{*};
use crate::logging::{*};



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

enum InstructionNames {
    ADC, //    add with carry
    AND, //    and (with accumulator)
    ASL, //    arithmetic shift left
    BCC, //    branch on carry clear
    BCS, //    branch on carry set
    BEQ, //    branch on equal (zero set)
    BIT, //    bit test
    BMI, //    branch on minus (negative set)
    BNE, //    branch on not equal (zero clear)
    BPL, //    branch on plus (negative clear)
    BRK, //    break / interrupt
    BVC, //    branch on overflow clear
    BVS, //    branch on overflow set
    CLC, //    clear carry
    CLD, //    clear decimal
    CLI, //    clear interrupt disable
    CLV, //    clear overflow
    CMP, //    compare (with accumulator)
    CPX, //    compare with X
    CPY, //    compare with Y
    DEC, //    decrement
    DEX, //    decrement X
    DEY, //    decrement Y
    EOR, //    exclusive or (with accumulator)
    INC, //    increment
    INX, //    increment X
    INY, //    increment Y
    JMP, //    jump
    JSR, //    jump subroutine
    LDA, //    load accumulator
    LDX, //    load X
    LDY, //    load Y
    LSR, //    logical shift right
    NOP, //    no operation
    ORA, //    or with accumulator
    PHA, //    push accumulator
    PHP, //    push processor status (SR)
    PLA, //    pull accumulator
    PLP, //    pull processor status (SR)
    ROL, //    rotate left
    ROR, //    rotate right
    RTI, //    return from interrupt
    RTS, //    return from subroutine
    SBC, //    subtract with carry
    SEC, //    set carry
    SED, //    set decimal
    SEI, //    set interrupt disable
    STA, //    store accumulator
    STX, //    store X
    STY, //    store Y
    TAX, //    transfer accumulator to X
    TAY, //    transfer accumulator to Y
    TSX, //    transfer stack pointer to X
    TXA, //    transfer X to accumulator
    TXS, //    transfer X to stack pointer
    TYA, //    transfer Y to accumulator
    ILLEGAL, // invalid instruction
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
    wait_cycles: u8, //implementation shortcut: only do next instruction when this is 0 to replicate the multi-cycle instructions

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


    fn get_addr_mode(&mut self) -> Result<AddressingModes, io::Error> {
        let tag = "dispatch_opcodes";

        let opcode = self.addresser.deref_byte(self.pc as usize)?;

        let hi = opcode >> 4;
        let lo = opcode & 0xF;

        let lo_group = lo >> 2;
        let hi_group = hi >> 3;

        let lo_even = lo % 2 == 0;
        let hi_even = hi % 2 == 0;

        match lo_group {
            0x0..=0x3 => {
                if opcode == 0x20 {
                    return Ok(AddressingModes::Absolute);
                }

                if !lo_even {
                    if hi_even {
                        return Ok(AddressingModes::XInd);
                    } else {
                        return Ok(AddressingModes::IndY);
                    }
                }

                if !hi_even {
                    return Ok(AddressingModes::Relative);
                }

                if hi_group == 0 {
                    return Ok(AddressingModes::Implicit);
                } else {
                    return Ok(AddressingModes::Immediate);
                }
            },
            0x4..=0x7 => {
                if hi_even {
                    return Ok(AddressingModes::ZeroPage);
                }
                if hi_group == 0 {
                    return Ok(AddressingModes::ZeroPageX);
                }
                return Ok(AddressingModes::ZeroPageY);
            },
            0x8..=0xB => {
                if lo_even && lo != 0xA {
                    return Ok(AddressingModes::Implicit);
                } else if lo_even {
                    return Ok(AddressingModes::Accumulator);
                }else if hi_even {
                    return Ok(AddressingModes::Immediate);
                } else {
                    return Ok(AddressingModes::AbsoluteY);
                }
            },
            0xC..=0xF => {
                if hi_even {
                    return Ok(AddressingModes::Absolute);
                } else {
                    return Ok(AddressingModes::AbsoluteX);
                }
            },
            _ => {
                log_error(tag, "quartet cannot be greater than 0xF");
                return Err(io::Error::new(io::ErrorKind::InvalidData, "logic error"));
            }

        }

    }


    fn dispatch_opcodes(&mut self) -> Result<(), io::Error> {

        let tag = "dispatch_opcodes";

        let opcode = self.addresser.deref_byte(self.pc as usize)?;

        let hi = opcode >> 4;
        let lo = opcode & 0xF;
        let even_hi = hi % 2 == 0;

        let lo_group = lo >> 2;
        let hi_group = hi >> 3;

        let lo_even = lo % 2 == 0;
        let hi_even = hi % 2 == 0;


        let mut mode: AddressingModes = AddressingModes::Implicit;

        match lo_group {
            0x0..=0x3 => {
                if !lo_even {
                    if hi_even {
                        mode = AddressingModes::XInd;
                    } else {
                        mode = AddressingModes::IndY;
                    }
                } else {

                }
            },
            0x4..=0x7 => {

            },
            0x8..=0xB => {

            },
            0xC..=0xF => {

            },
            _ => {
                log_error(tag, "quartet cannot be greater than 0xF");
            }

        }

        return Ok(());

    }
}