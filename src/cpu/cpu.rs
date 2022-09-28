use crate::{cpu::{registers::{Flag, Registers, Regs}, opcode::{INSTRUCTIONS, CB_INSTRUCTIONS}}, mmu::bus::Bus};

use std::ops::{Add, Sub};

macro_rules! reg8 {
    ($self:ident, $bits:expr, $bus:ident) => {
        match $bits {
            0 => $self.registers.B,
            1 => $self.registers.C,
            2 => $self.registers.D,
            3 => $self.registers.E,
            4 => $self.registers.H,
            5 => $self.registers.L,
            6 => $bus.read_byte($self.registers.get_hl()),
            7 => $self.registers.A,
            _ => panic!("Invalid register!"),
        }
    };
}

pub struct CPU {
    pub registers: Registers,
    pub ime: bool,
    pub halt: bool,
    pub stopped: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers::new_dmg(0x66),
            ime: false,
            halt: false,
            stopped: false,
        }
    }

    // returns m-cycles for now
    #[rustfmt::skip]
    pub fn tick(&mut self, bus: &mut Bus) -> u8 {
        let opcode = self.fetch_operand(bus);

        // let print_value = if INSTRUCTIONS[opcode as usize].bytes == 2 {
        //     bus.read_byte(self.registers.PC) as u16
        // } else if INSTRUCTIONS[opcode as usize].bytes == 3 {
        //     bus.read_16(self.registers.PC)
        // } else {
        //     0
        // };

        // if opcode != 0xCB {
        //     println!("{:#08X}: {:#04X}      {} {:#06X}", self.registers.PC - 1, opcode, INSTRUCTIONS[opcode as usize].name, print_value);
        // } else {
        //     let cb_opcode = bus.read_byte(self.registers.PC);
        //     println!("{:#08X}: {:#04X}      {}", self.registers.PC - 1, cb_opcode, CB_INSTRUCTIONS[cb_opcode as usize].name);
        // }

        if opcode == 0xCB {
            let cb_opcode = self.fetch_operand(bus);

            match cb_opcode {
                0x00 => { self.registers.B = self.rlc(self.registers.B); 2 }
                0x01 => { self.registers.C = self.rlc(self.registers.C); 2 }
                0x02 => { self.registers.D = self.rlc(self.registers.D); 2 }
                0x03 => { self.registers.E = self.rlc(self.registers.E); 2 }
                0x04 => { self.registers.H = self.rlc(self.registers.H); 2 }
                0x05 => { self.registers.L = self.rlc(self.registers.L); 2 }
                0x06 => {
                    bus.write_byte(
                        self.registers.get_hl(),
                        self.rlc(bus.read_byte(self.registers.get_hl())),
                    );
                    4
                }
                0x07 => { self.registers.A = self.rlc(self.registers.A); 2 }
                0x08 => { self.registers.B = self.rrc(self.registers.B); 2 }
                0x09 => { self.registers.C = self.rrc(self.registers.C); 2 }
                0x0A => { self.registers.D = self.rrc(self.registers.D); 2 }
                0x0B => { self.registers.E = self.rrc(self.registers.E); 2 }
                0x0C => { self.registers.H = self.rrc(self.registers.H); 2 }
                0x0D => { self.registers.L = self.rrc(self.registers.L); 2 }
                0x0E => {
                    bus.write_byte(
                        self.registers.get_hl(),
                        self.rrc(bus.read_byte(self.registers.get_hl())),
                    );
                    4
                }
                0x0F => { self.registers.A = self.rl(self.registers.A); 2 }
                0x10 => { self.registers.B = self.rl(self.registers.B); 2 }
                0x11 => { self.registers.C = self.rl(self.registers.C); 2 }
                0x12 => { self.registers.D = self.rl(self.registers.D); 2 }
                0x13 => { self.registers.E = self.rl(self.registers.E); 2 }
                0x14 => { self.registers.H = self.rl(self.registers.H); 2 }
                0x15 => { self.registers.L = self.rl(self.registers.L); 2 }
                0x16 => {
                    bus.write_byte(
                        self.registers.get_hl(),
                        self.rl(bus.read_byte(self.registers.get_hl())),
                    );
                    4
                }
                0x17 => { self.registers.A = self.rl(self.registers.A); 2 }
                0x18 => { self.registers.B = self.rr(self.registers.B); 2 }
                0x19 => { self.registers.C = self.rr(self.registers.C); 2 }
                0x1A => { self.registers.D = self.rr(self.registers.D); 2 }
                0x1B => { self.registers.E = self.rr(self.registers.E); 2 }
                0x1C => { self.registers.H = self.rr(self.registers.H); 2 }
                0x1D => { self.registers.L = self.rr(self.registers.L); 2 }
                0x1E => {
                    bus.write_byte(
                        self.registers.get_hl(),
                        self.rr(bus.read_byte(self.registers.get_hl())),
                    );
                    4
                }
                0x1F => { self.registers.A = self.sla(self.registers.A); 2 }
                0x20 => { self.registers.B = self.sla(self.registers.B); 2 }
                0x21 => { self.registers.C = self.sla(self.registers.C); 2 }
                0x22 => { self.registers.D = self.sla(self.registers.D); 2 }
                0x23 => { self.registers.E = self.sla(self.registers.E); 2 }
                0x24 => { self.registers.H = self.sla(self.registers.H); 2 }
                0x25 => { self.registers.L = self.sla(self.registers.L); 2 }
                0x26 => {
                    bus.write_byte(
                        self.registers.get_hl(),
                        self.sla(bus.read_byte(self.registers.get_hl())),
                    );
                    4
                }
                0x27 => { self.registers.A = self.sla(self.registers.A); 2 }
                0x28 => { self.registers.B = self.sra(self.registers.B); 2 }
                0x29 => { self.registers.C = self.sra(self.registers.C); 2 }
                0x2A => { self.registers.D = self.sra(self.registers.D); 2 }
                0x2B => { self.registers.E = self.sra(self.registers.E); 2 }
                0x2C => { self.registers.H = self.sra(self.registers.H); 2 }
                0x2D => { self.registers.L = self.sra(self.registers.L); 2 }
                0x2E => {
                    bus.write_byte(
                        self.registers.get_hl(),
                        self.sra(bus.read_byte(self.registers.get_hl())),
                    );
                    4
                }
                0x2F => { self.registers.A = self.sra(self.registers.A); 2 }
                0x30 => { self.registers.B = self.swap(self.registers.B); 2 }
                0x31 => { self.registers.C = self.swap(self.registers.C); 2 }
                0x32 => { self.registers.D = self.swap(self.registers.D); 2 }
                0x33 => { self.registers.E = self.swap(self.registers.E); 2 }
                0x34 => { self.registers.H = self.swap(self.registers.H); 2 }
                0x35 => { self.registers.L = self.swap(self.registers.L); 2 }
                0x36 => {
                    bus.write_byte(
                        self.registers.get_hl(),
                        self.swap(bus.read_byte(self.registers.get_hl())),
                    );
                    4
                }
                0x37 => { self.registers.A = self.swap(self.registers.A); 2 }
                0x38 => { self.registers.B = self.srl(self.registers.B); 2 }
                0x39 => { self.registers.C = self.srl(self.registers.C); 2 }
                0x3A => { self.registers.D = self.srl(self.registers.D); 2 }
                0x3B => { self.registers.E = self.srl(self.registers.E); 2 }
                0x3C => { self.registers.H = self.srl(self.registers.H); 2 }
                0x3D => { self.registers.L = self.srl(self.registers.L); 2 }
                0x3E => {
                    bus.write_byte(
                        self.registers.get_hl(),
                        self.srl(bus.read_byte(self.registers.get_hl())),
                    );
                    4
                }
                0x3F => { self.registers.A = self.srl(self.registers.A); 2 }
                0x40..=0x7F => {
                    let bit = (cb_opcode >> 3) & 0b111;
                    let reg = cb_opcode & 0b111;

                    match reg {
                        0 => { self.bit(bit, self.registers.B); 2 }
                        1 => { self.bit(bit, self.registers.C); 2 }
                        2 => { self.bit(bit, self.registers.D); 2 }
                        3 => { self.bit(bit, self.registers.E); 2 }
                        4 => { self.bit(bit, self.registers.H); 2 }
                        5 => { self.bit(bit, self.registers.L); 2 }
                        6 => { self.bit(bit, bus.read_byte(self.registers.get_hl())); 3 }
                        7 => { self.bit(bit, self.registers.A); 2 }
                        _ => panic!("Invalid register!")
                    }
                }
                0x80..=0xBF => {
                    let bit = (cb_opcode >> 3) & 0b111;
                    let reg = cb_opcode & 0b111;

                    match reg {
                        0 => { self.registers.B = self.res(bit, self.registers.B); 2 }
                        1 => { self.registers.C = self.res(bit, self.registers.C); 2 }
                        2 => { self.registers.D = self.res(bit, self.registers.D); 2 }
                        3 => { self.registers.E = self.res(bit, self.registers.E); 2 }
                        4 => { self.registers.H = self.res(bit, self.registers.H); 2 }
                        5 => { self.registers.L = self.res(bit, self.registers.L); 2 }
                        6 => {
                            bus.write_byte(
                                self.registers.get_hl(),
                                self.res(bit, bus.read_byte(self.registers.get_hl()))
                            );
                            4
                        }
                        7 => { self.registers.B = self.res(bit, self.registers.B); 2 }
                        _ => panic!("Invalid register!")
                    }
                }
                0xC0..=0xFF => {
                    let bit = (cb_opcode >> 3) & 0b111;
                    let reg = cb_opcode & 0b111;

                    match reg {
                        0 => { self.registers.B = self.set(bit, self.registers.B); 2 }
                        1 => { self.registers.C = self.set(bit, self.registers.C); 2 }
                        2 => { self.registers.D = self.set(bit, self.registers.D); 2 }
                        3 => { self.registers.E = self.set(bit, self.registers.E); 2 }
                        4 => { self.registers.H = self.set(bit, self.registers.H); 2 }
                        5 => { self.registers.L = self.set(bit, self.registers.L); 2 }
                        6 => {
                            bus.write_byte(
                                self.registers.get_hl(),
                                self.set(bit, bus.read_byte(self.registers.get_hl()))
                            );
                            4
                        }
                        7 => { self.registers.B = self.set(bit, self.registers.B); 2 }
                        _ => panic!("Invalid register!")
                    }
                }
            }
        } else {
            match opcode {
                0x00 => { self.nop(); 1 }
                0x01 | 0x11 | 0x21 | 0x31 => {
                    let reg = (opcode >> 4) & 0b11;

                    let low = self.fetch_operand(bus);
                    let high = self.fetch_operand(bus);

                    match reg {
                        0 => { self.ld16(Regs::BC, low, high) }
                        1 => { self.ld16(Regs::DE, low, high) }
                        2 => { self.ld16(Regs::HL, low, high) }
                        3 => { self.ld16(Regs::SP, low, high) }
                        _ => { panic!("Invalid register!") }
                    }

                    3
                }
                0x02 | 0x12 | 0x22 | 0x32 => {
                    let reg = (opcode >> 4) & 0b11;

                    match reg {
                        0 => { self.st_a(bus, self.registers.get_bc()) }
                        1 => { self.st_a(bus, self.registers.get_de()) }
                        2 => {
                            self.st_a(bus, self.registers.get_hl());
                            self.inc16(Regs::HL);
                        }
                        3 => {
                            self.st_a(bus, self.registers.get_hl());
                            self.dec16(Regs::HL);
                        }
                        _ => { panic!("Invalid register!") }
                    }

                    2
                }
                0x03 | 0x13 | 0x23 | 0x33 => {
                    let reg = (opcode >> 4) & 0b11;

                    match reg {
                        0 => { self.inc16(Regs::BC) }
                        1 => { self.inc16(Regs::DE) }
                        2 => { self.inc16(Regs::HL) }
                        3 => { self.inc16(Regs::SP) }
                        _ => { panic!("Invalid register!") }
                    }

                    2
                }
                0x04 | 0x14 | 0x24 | 0x34 | 0x0C | 0x1C | 0x2C | 0x3C => {
                    let reg = (opcode >> 3) & 0b111;

                    match reg {
                        0 => { self.registers.B = self.inc8(self.registers.B); 1 }
                        1 => { self.registers.C = self.inc8(self.registers.C); 1 }
                        2 => { self.registers.D = self.inc8(self.registers.D); 1 }
                        3 => { self.registers.E = self.inc8(self.registers.E); 1 }
                        4 => { self.registers.H = self.inc8(self.registers.H); 1 }
                        5 => { self.registers.L = self.inc8(self.registers.L); 1 }
                        6 => {
                            bus.write_byte(
                                self.registers.get_hl(),
                                self.inc8(bus.read_byte(self.registers.get_hl()))
                            );
                            3
                        }
                        7 => { self.registers.A = self.inc8(self.registers.A); 1 }
                        _ => { panic!("Invalid register!") }
                    }
                }
                0x05 | 0x15 | 0x25 | 0x35 | 0x0D | 0x1D | 0x2D | 0x3D => {
                    let reg = (opcode >> 3) & 0b111;

                    match reg {
                        0 => { self.registers.B = self.dec8(self.registers.B); 1 }
                        1 => { self.registers.C = self.dec8(self.registers.C); 1 }
                        2 => { self.registers.D = self.dec8(self.registers.D); 1 }
                        3 => { self.registers.E = self.dec8(self.registers.E); 1 }
                        4 => { self.registers.H = self.dec8(self.registers.H); 1 }
                        5 => { self.registers.L = self.dec8(self.registers.L); 1 }
                        6 => {
                            bus.write_byte(
                                self.registers.get_hl(),
                                self.dec8(bus.read_byte(self.registers.get_hl()))
                            );
                            3
                        }
                        7 => { self.registers.A = self.dec8(self.registers.A); 1 }
                        _ => { panic!("Invalid register!") }
                    }
                }
                0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E => {
                    let reg = (opcode >> 3) & 0b111;
                    let value = self.fetch_operand(bus);
                    self.ld8(bus, reg, value);

                    // 3 cycles if LD (HL), u8
                    if reg == 6 { 3 } else { 2 }
                }
                0x07 => { self.rlca(); 1 }
                0x08 => {
                    let address = bus.read_16(self.registers.PC);
                    bus.write_16(address, self.registers.SP);

                    self.registers.PC += 2; // TODO
                    5
                }
                0x09 | 0x19 | 0x29 | 0x39 => {
                    let reg = (opcode >> 4) & 0b111;

                    match reg {
                        0 => { self.add_hl(Regs::BC) }
                        1 => { self.add_hl(Regs::DE) }
                        2 => { self.add_hl(Regs::HL) }
                        3 => { self.add_hl(Regs::SP) }
                        _ => { panic!("Invalid register!") }
                    }

                    2
                }
                0x0A | 0x1A | 0x2A | 0x3A => {
                    let reg = (opcode >> 4) & 0b111;

                    match reg {
                        0 => { self.ld_a(bus.read_byte(self.registers.get_bc())) }
                        1 => { self.ld_a(bus.read_byte(self.registers.get_de())) }
                        2 => {
                            self.ld_a(bus.read_byte(self.registers.get_hl()));
                            self.inc16(Regs::HL);
                        }
                        3 => {
                            self.ld_a(bus.read_byte(self.registers.get_hl()));
                            self.dec16(Regs::HL);
                        }
                        _ => { panic!("Invalid register!") }
                    }

                    2
                }
                0x0B | 0x1B | 0x2B | 0x3B => {
                    let reg = (opcode >> 4) & 0b111;

                    match reg {
                        0 => { self.dec16(Regs::BC) }
                        1 => { self.dec16(Regs::DE) }
                        2 => { self.dec16(Regs::HL) }
                        3 => { self.dec16(Regs::SP) }
                        _ => { panic!("Invalid register!") }
                    }

                    2
                }
                0x0F => { self.rrca(); 1 }
                0x10 => { println!("STOP, not implemented"); bus.timer.div = 0;  1 } // TODO
                0x17 => { self.rla(); 1 }
                0x18 => {
                    let value = self.fetch_operand(bus);
                    self.jr(value); 
                    
                    3 
                }
                0x1F => { self.rra(); 1 }
                0x20 | 0x28 | 0x30 | 0x38 => {
                    let condition = (opcode >> 3) & 0x11;
                    let value = self.fetch_operand(bus);

                    match condition {
                        0 | 1 => self.jr_flag(Flag::Zero, value, condition != 0),
                        2 | 3 => self.jr_flag(Flag::Carry, value, condition != 2),
                        _ => panic!("Invalid condition!")
                    }
                }
                0x27 => { self.daa(); 1 }
                0x2F => { self.cpl_a(); 1 }
                0x37 => { self.scf(); 1 }
                0x3F => { self.ccf(); 1 }
                0x40..=0x7F => {
                    let dest = (opcode >> 3) & 0b111;
                    let source = opcode & 0b111;

                    // LD (HL), (HL) doesn't exist, 0x76 is HALT
                    if dest == 6 && source == 6 {
                        println!("HALT");
                        return 1;
                    }

                    match source {
                        0b110 => { self.ld8(bus, dest, reg8!(self, source, bus)); 2 }
                        _ => { 
                            self.ld8(bus, dest, reg8!(self, source, bus));
                            
                            // (HL)
                            if dest == 0b110 {
                                2
                            } else {
                                1
                            }
                        }
                    }
                }
                0x80..=0xBF => {
                    let reg8 = reg8!(self, opcode & 0b111, bus);
                    let cycles = if opcode & 0b111 == 0b110 { 2 } else { 1 };

                    match opcode {
                        0x80..=0x87 => { self.add_a(reg8); cycles }
                        0x88..=0x8F => { self.adc_a(reg8); cycles }
                        0x90..=0x97 => { self.sub_a(reg8); cycles }
                        0x98..=0x9F => { self.sbc_a(reg8); cycles }
                        0xA0..=0xA7 => { self.and_a(reg8); cycles }
                        0xA8..=0xAF => { self.xor_a(reg8); cycles }
                        0xB0..=0xB7 => { self.or_a(reg8); cycles }
                        0xB8..=0xBF => { self.cp_a(reg8); cycles }
                        _ => { panic!("Out of range!") }
                    }
                }
                0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                    let condition = (opcode >> 3) & 0b11;

                    match condition {
                        0 | 1 => self.ret_flag(bus, Flag::Zero, condition != 0),
                        2 | 3 => self.ret_flag(bus, Flag::Carry, condition != 2),
                        _ => panic!("Invalid condition!")
                    }
                }
                0xC1 => { self.pop(Regs::BC, bus); 3 }
                0xD1 => { self.pop(Regs::DE, bus); 3 }
                0xE1 => { self.pop(Regs::HL, bus); 3 }
                0xF1 => { self.pop(Regs::AF, bus); 3 }
                0xC2 | 0xCA | 0xD2 | 0xDA => {
                    let condition = (opcode >> 3) & 0b11;
                    let value = bus.read_16(self.registers.PC);

                    self.registers.PC += 2; // TODO

                    match condition {
                        0 | 1 => self.jp_flag(Flag::Zero, value, condition != 0),
                        2 | 3 => self.jp_flag(Flag::Carry, value, condition != 2),
                        _ => panic!("Invalid condition!")
                    }
                }
                0xC3 => { 
                    self.jp(bus.read_16(self.registers.PC));
                    4 
                }
                0xC4 | 0xCC | 0xD4 | 0xDC => {
                    let condition = (opcode >> 3) & 0b11;
                    let value = bus.read_16(self.registers.PC);

                    self.registers.PC += 2; // TODO

                    match condition {
                        0 | 1 => self.call_flag(bus, Flag::Zero, value, condition != 0),
                        2 | 3 => self.call_flag(bus, Flag::Carry, value, condition != 2),
                        _ => panic!("Invalid condition!")
                    }
                }
                0xC5 => { self.push(Regs::BC, bus); 4 }
                0xD5 => { self.push(Regs::DE, bus); 4 }
                0xE5 => { self.push(Regs::HL, bus); 4 }
                0xF5 => { self.push(Regs::AF, bus); 4 }
                0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
                    let value = self.fetch_operand(bus);

                    match opcode {
                        0xC6 => self.add_a(value),
                        0xCE => self.adc_a(value),
                        0xD6 => self.sub_a(value),
                        0xDE => self.sbc_a(value),
                        0xE6 => self.and_a(value),
                        0xEE => self.xor_a(value),
                        0xF6 => self.or_a(value),
                        0xFE => self.cp_a(value),
                        _ => panic!("Invalid instruction!")
                    }

                    2
                }
                0xC7 | 0xD7 | 0xE7 | 0xF7 | 0xCF | 0xDF | 0xEF | 0xFF => {
                    let rst_vec = (opcode >> 3) & 0b111;
                    self.rst(bus, rst_vec);

                    4
                }
                0xC9 => { self.ret(bus); 4 }
                0xCD => { 
                    let address = bus.read_16(self.registers.PC);

                    self.registers.PC += 2;
                    self.call(bus, address);

                    6
                }
                0xD9 => { self.reti(bus); 4 }
                0xE0 => {
                    let address = 0xFF00 + (self.fetch_operand(bus) as u16);
                    bus.write_byte(address, self.registers.A);

                    3
                }
                0xF0 => {
                    let address = 0xFF00 + (self.fetch_operand(bus) as u16);
                    self.registers.A = bus.read_byte(address);

                    3
                }
                0xE2 => {
                    let address = 0xFF00 + (self.registers.C as u16);
                    bus.write_byte(address, self.registers.A);

                    2
                }
                0xF2 => {
                    let address = 0xFF00 + (self.registers.C as u16);
                    self.registers.A = bus.read_byte(address);

                    2
                }
                0xF3 => { self.di(); 1 }
                0xFB => { self.ei(); 1 }
                0xE8 => {
                    // wrapping_add, adding unsigned to signed
                    let value = self.fetch_operand(bus);
                    self.add_sp(value);

                    4
                }
                0xE9 => { self.jp_hl(); 1 }
                0xEA => {
                    let address = bus.read_16(self.registers.PC);
                    bus.write_byte(address, self.registers.A);

                    self.registers.PC += 2; // TODO

                    4
                }
                0xFA => {
                    let address = bus.read_16(self.registers.PC);
                    self.registers.A = bus.read_byte(address);

                    self.registers.PC += 2; // TODO

                    4
                }
                0xF8 => {
                    let old_sp = self.registers.SP;
                    let value = self.fetch_operand(bus);

                    self.add_sp(value);

                    self.registers.set_hl(self.registers.SP);
                    self.registers.SP = old_sp;

                    3
                }
                0xF9 => { self.registers.SP = self.registers.get_hl(); 2 }
                _ => { panic!("Illegal or invalid opcode: {:#04X}", opcode) }
            }
        }

    }

    /// Read next operand at PC and increase PC after.
    fn fetch_operand(&mut self, bus: &Bus) -> u8 {
        let operand = bus.read_byte(self.registers.PC);

        // print!(" {:#04X} ", operand);

        // println!("PC: {:#08X} executing {:#04X}", self.registers.PC, operand);

        self.registers.PC += 1;

        // println!("PC: {:#08X} after increase by one", self.registers.PC);

        operand
    }

    fn rlc(&mut self, reg8: u8) -> u8 {
        if reg8 & (1 << 7) == 0 {
            self.registers.unset_flag(Flag::Carry);
        } else {
            self.registers.set_flag(Flag::Carry);
        }

        let reg8 = reg8.rotate_left(1);

        if reg8 == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);

        reg8
    }

    fn rrc(&mut self, reg8: u8) -> u8 {
        if reg8 & 0b1 == 0 {
            self.registers.unset_flag(Flag::Carry);
        } else {
            self.registers.set_flag(Flag::Carry);
        }

        let reg8 = reg8.rotate_right(1);

        if reg8 == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);

        reg8
    }

    fn rl(&mut self, reg8: u8) -> u8 {
        let bit7 = reg8 & (1 << 7);
        let mut reg8 = reg8 << 1;

        if self.registers.get_flag(Flag::Carry) {
            reg8 |= 1;
        } else {
            reg8 &= !(1);
        }

        if bit7 != 0 {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        if reg8 == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);

        reg8
    }

    fn rr(&mut self, reg8: u8) -> u8 {
        let bit0 = reg8 & 0b1;
        let mut reg8 = reg8 >> 1;

        if self.registers.get_flag(Flag::Carry) {
            reg8 |= 1 << 7;
        } else {
            reg8 &= !(1 << 7);
        }

        if bit0 != 0 {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        if reg8 == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);

        reg8
    }

    fn sla(&mut self, reg8: u8) -> u8 {
        if reg8 & (1 << 7) != 0 {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        let mut reg8 = reg8 << 1;
        reg8 &= !(1);

        if reg8 == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);

        reg8
    }

    fn sra(&mut self, reg8: u8) -> u8 {
        if reg8 & 0b1 != 0 {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        let reg8 = reg8 >> 1;

        if reg8 == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);

        reg8
    }

    fn swap(&mut self, reg8: u8) -> u8 {
        let higher_four_bits = reg8 & 0xF0;
        let reg8 = (reg8 << 4) | higher_four_bits;

        if reg8 == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);
        self.registers.unset_flag(Flag::Carry);

        reg8
    }

    fn srl(&mut self, reg8: u8) -> u8 {
        if reg8 & 0b1 != 0 {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        let mut reg8 = reg8 >> 1;
        reg8 &= !(1 << 7);

        if reg8 == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);

        reg8
    }

    fn bit(&mut self, bit: u8, reg8: u8) {
        let reg_bit = reg8 & (1 << bit);

        if reg_bit != 0 {
            self.registers.unset_flag(Flag::Zero);
        } else {
            self.registers.set_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);
        self.registers.set_flag(Flag::HalfCarry);
    }

    #[inline(always)]
    fn res(&mut self, bit: u8, reg8: u8) -> u8 {
        reg8 & !(1 << bit)
    }

    #[inline(always)]
    fn set(&mut self, bit: u8, reg8: u8) -> u8 {
        reg8 | 1 << bit
    }

    fn ld8(&mut self, bus: &mut Bus, reg8_code: u8, value: u8) {
        match reg8_code {
            0 => self.registers.B = value,
            1 => self.registers.C = value,
            2 => self.registers.D = value,
            3 => self.registers.E = value,
            4 => self.registers.H = value,
            5 => self.registers.L = value,
            6 => {
                bus.write_byte(self.registers.get_hl(), value);
            }
            7 => self.registers.A = value,
            _ => {
                panic!("Invalid register!")
            }
        }
    }

    fn ld_a(&mut self, value: u8) {
        self.registers.A = value;
    }

    fn st_a(&mut self, bus: &mut Bus, value: u16) {
        bus.write_byte(value, self.registers.A);
    }

    fn add_sp(&mut self, value: u8) {
        let signed_operand = value.wrapping_neg() as u16;

        if self._check_if_half_carry_16(self.registers.SP, signed_operand, u16::wrapping_add) {
            self.registers.set_flag(Flag::HalfCarry);
        } else {
            self.registers.unset_flag(Flag::HalfCarry);
        }

        let (result, cy) = self.registers.SP.overflowing_add(signed_operand);
        self.registers.SP = result;

        if cy {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }
    }

    /// Load 2 bytes of data into `reg16`.
    ///
    /// `low` is the first byte of immediate data
    ///
    /// `high` is the second byte of immediate data
    fn ld16(&mut self, reg16: Regs, low: u8, high: u8) {
        match reg16 {
            Regs::BC => {
                self.registers.C = low;
                self.registers.B = high;
            }
            Regs::DE => {
                self.registers.E = low;
                self.registers.D = high;
            }
            Regs::HL => {
                self.registers.L = low;
                self.registers.H = high;
            }
            Regs::SP => {
                self.registers.SP = (high as u16) << 8 | low as u16;
            }
            _ => {}
        }
    }

    fn inc8(&mut self, value: u8) -> u8 {
        let value = value + 1;

        if value == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);

        if self._check_if_half_carry(value - 1, 1, Add::add) {
            self.registers.set_flag(Flag::HalfCarry);
        }

        // println!("New value after increasing by one: {:#04X}", value);
        // println!("Zero flag: {}", self.registers.get_flag(Flag::Zero));

        value
    }

    fn dec8(&mut self, value: u8) -> u8 {
        let value = value - 1;

        if value == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.set_flag(Flag::Substraction);

        if self._check_if_half_carry(value + 1, 1, Sub::sub) {
            self.registers.set_flag(Flag::HalfCarry);
        }

        value
    }

    fn inc16(&mut self, reg16: Regs) {
        match reg16 {
            Regs::BC => self.registers.set_bc(self.registers.get_bc() + 1),
            Regs::DE => self.registers.set_de(self.registers.get_de() + 1),
            Regs::HL => self.registers.set_hl(self.registers.get_hl() + 1),
            Regs::SP => self.registers.SP += 1,
            _ => {}
        }
    }

    fn dec16(&mut self, reg16: Regs) {
        match reg16 {
            Regs::BC => self.registers.set_bc(self.registers.get_bc() - 1),
            Regs::DE => self.registers.set_de(self.registers.get_de() - 1),
            Regs::HL => self.registers.set_hl(self.registers.get_hl() - 1),
            Regs::SP => self.registers.SP -= 1,
            _ => {}
        }
    }

    fn add_hl(&mut self, reg16: Regs) {
        let carry: bool;
        let half_carry: bool;

        match reg16 {
            Regs::BC => {
                half_carry = self._check_if_half_carry_16(
                    self.registers.get_hl(),
                    self.registers.get_bc(),
                    Add::add,
                );

                let (result, cy) = self
                    .registers
                    .get_hl()
                    .overflowing_add(self.registers.get_bc());
                self.registers.set_hl(result);

                carry = cy;
            }
            Regs::DE => {
                half_carry = self._check_if_half_carry_16(
                    self.registers.get_hl(),
                    self.registers.get_de(),
                    Add::add,
                );

                let (result, cy) = self
                    .registers
                    .get_hl()
                    .overflowing_add(self.registers.get_de());
                self.registers.set_hl(result);

                carry = cy;
            }
            Regs::HL => {
                half_carry = self._check_if_half_carry_16(
                    self.registers.get_hl(),
                    self.registers.get_hl(),
                    Add::add,
                );

                let (result, cy) = self
                    .registers
                    .get_hl()
                    .overflowing_add(self.registers.get_hl());
                self.registers.set_hl(result);

                carry = cy;
            }
            Regs::SP => {
                half_carry = self._check_if_half_carry_16(
                    self.registers.get_hl(),
                    self.registers.SP,
                    Add::add,
                );

                let (result, cy) = self.registers.get_hl().overflowing_add(self.registers.SP);
                self.registers.set_hl(result);

                carry = cy;
            }
            _ => {
                carry = false;
                half_carry = false;
            }
        }

        if carry {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        if half_carry {
            self.registers.set_flag(Flag::HalfCarry);
        } else {
            self.registers.unset_flag(Flag::HalfCarry);
        }

        self.registers.unset_flag(Flag::Substraction);
    }

    fn add_a(&mut self, value: u8) {
        if self._check_if_half_carry(self.registers.A, value, Add::add) {
            self.registers.set_flag(Flag::HalfCarry);
        }

        let (result, carry) = self.registers.A.overflowing_add(value);
        self.registers.A = result;

        if self.registers.A == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        if carry {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        self.registers.unset_flag(Flag::Substraction);
    }

    fn adc_a(&mut self, value: u8) {
        let adc_value = value + (self.registers.get_flag(Flag::Carry) as u8);

        if self._check_if_half_carry(self.registers.A, adc_value, Add::add) {
            self.registers.set_flag(Flag::HalfCarry);
        }

        let (result, carry) = self.registers.A.overflowing_add(adc_value);
        self.registers.A = result;

        if self.registers.A == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        if carry {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        self.registers.unset_flag(Flag::Substraction);
    }

    fn sub_a(&mut self, value: u8) {
        if self._check_if_half_carry(self.registers.A, value, Sub::sub) {
            self.registers.set_flag(Flag::HalfCarry);
        }

        let (result, carry) = self.registers.A.overflowing_sub(value);
        self.registers.A = result;

        if self.registers.A == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        if carry {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        self.registers.set_flag(Flag::Substraction);
    }

    fn sbc_a(&mut self, value: u8) {
        let sbc_value = value + (self.registers.get_flag(Flag::Carry) as u8);

        if self._check_if_half_carry(self.registers.A, sbc_value, Sub::sub) {
            self.registers.set_flag(Flag::HalfCarry);
        }

        let (result, carry) = self.registers.A.overflowing_sub(sbc_value);
        self.registers.A = result;

        if self.registers.A == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        if carry {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        self.registers.set_flag(Flag::Substraction);
    }

    fn and_a(&mut self, value: u8) {
        self.registers.A &= value;

        if self.registers.A == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);
        self.registers.set_flag(Flag::HalfCarry);
        self.registers.unset_flag(Flag::Carry);
    }

    fn xor_a(&mut self, value: u8) {
        self.registers.A ^= value;

        if self.registers.A == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);
        self.registers.unset_flag(Flag::Carry);
    }

    fn or_a(&mut self, value: u8) {
        self.registers.A |= value;
        // println!("Register A after ORING with {:#04X}: {:#04X}", value, self.registers.A);

        if self.registers.A == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);
        self.registers.unset_flag(Flag::Carry);
    }

    fn cp_a(&mut self, value: u8) {
        if self._check_if_half_carry(self.registers.A, value, Sub::sub) {
            self.registers.set_flag(Flag::HalfCarry);
        }

        let (result, carry) = self.registers.A.overflowing_sub(value);

        // A and value are equal
        if result == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        if carry {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        self.registers.set_flag(Flag::Substraction);
    }

    fn cpl_a(&mut self) {
        self.registers.A = !self.registers.A;
        self.registers.set_flag(Flag::Substraction);
        self.registers.set_flag(Flag::HalfCarry);
    }

    fn ccf(&mut self) {
        self.registers.F ^= 1 << 4;
        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);
    }

    fn scf(&mut self) {
        self.registers.set_flag(Flag::Carry);

        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);
    }

    fn jr(&mut self, value: u8) { // FIXME:
        // println!("value as u8: {:#04X}", value);

        let value = value.wrapping_neg() as i8 * (-1);
        // dbg!(value);
        // dbg!(value as u16);

        self.registers.PC = self.registers.PC.wrapping_add(value as u16);
    }

    fn jp_hl(&mut self) {
        self.registers.PC = self.registers.get_hl();
    }

    fn nop(&mut self) {
        // self.registers.PC += 1;
    }

    fn rlca(&mut self) {
        if self.registers.A & (1 << 7) == 0 {
            self.registers.unset_flag(Flag::Carry);
        } else {
            self.registers.set_flag(Flag::Carry);
        }

        self.registers.A = self.registers.A.rotate_left(1);

        self.registers.unset_flag(Flag::Zero);
        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);
    }

    fn rla(&mut self) {
        let bit7 = self.registers.A & (1 << 7);
        self.registers.A = self.registers.A << 1;

        if self.registers.get_flag(Flag::Carry) {
            self.registers.A |= 1;
        } else {
            self.registers.A &= !(1);
        }

        if bit7 != 0 {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        self.registers.unset_flag(Flag::Zero);
        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);
    }

    // TODO: different cycle counts
    /// Only for `Zero` and `Carry` flag.
    ///
    /// Maps to `jr nz`, `jr nc`, `jr z` and `jr c`.  
    ///
    /// Returns the m-cycles it took depending on the condition
    fn jr_flag(&mut self, flag: Flag, value: u8, condition: bool) -> u8 {
        if self.registers.get_flag(flag) == condition {
            self.jr(value);
            return 3;
        }

        2
    }

    fn rrca(&mut self) {
        if self.registers.A & 0b1 == 0 {
            self.registers.unset_flag(Flag::Carry);
        } else {
            self.registers.set_flag(Flag::Carry);
        }

        self.registers.A = self.registers.A.rotate_right(1);

        self.registers.unset_flag(Flag::Zero);
        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);
    }

    fn rra(&mut self) {
        let bit0 = self.registers.A & 0b1;
        self.registers.A = self.registers.A >> 1;

        if self.registers.get_flag(Flag::Carry) {
            self.registers.A |= 1 << 7;
        } else {
            self.registers.A &= !(1 << 7);
        }

        if bit0 != 0 {
            self.registers.set_flag(Flag::Carry);
        } else {
            self.registers.unset_flag(Flag::Carry);
        }

        self.registers.unset_flag(Flag::Zero);
        self.registers.unset_flag(Flag::Substraction);
        self.registers.unset_flag(Flag::HalfCarry);
    }

    /// Turn register A into a binary-coded decimal (BCD)
    ///
    /// https://forums.nesdev.org/viewtopic.php?t=15944
    fn daa(&mut self) {
        if self.registers.get_flag(Flag::Substraction) {
            if self.registers.get_flag(Flag::Carry) {
                self.registers.A -= 0x60;
            }
            if self.registers.get_flag(Flag::HalfCarry) {
                self.registers.A -= 0x6;
            }
        } else {
            if self.registers.get_flag(Flag::Carry) || self.registers.A > 0x99 {
                self.registers.A += 0x60;
                self.registers.set_flag(Flag::Carry);
            }
            if self.registers.get_flag(Flag::HalfCarry) || (self.registers.A & 0x0F) > 0x09 {
                self.registers.A += 0x6;
            }
        }

        if self.registers.A == 0 {
            self.registers.set_flag(Flag::Zero);
        } else {
            self.registers.unset_flag(Flag::Zero);
        }

        self.registers.unset_flag(Flag::HalfCarry);
    }

    fn jp(&mut self, value: u16) {
        // println!("JUMPING TO: {:#06X}", value);
        self.registers.PC = value;
    }

    /// `value` is a 16 bit immediate operand.
    /// Second byte (after opcode) is the lower byte, third is higher byte.
    fn jp_flag(&mut self, flag: Flag, value: u16, condition: bool) -> u8 {
        if self.registers.get_flag(flag) == condition {
            self.registers.PC = value;
            return 4;
        }

        3
    }

    fn pop(&mut self, reg16: Regs, bus: &mut Bus) {
        match reg16 {
            Regs::BC => {
                self.registers.C = bus.read_byte(self.registers.SP);
                self.registers.SP += 1;
                self.registers.B = bus.read_byte(self.registers.SP);
            }
            Regs::DE => {
                self.registers.E = bus.read_byte(self.registers.SP);
                self.registers.SP += 1;
                self.registers.D = bus.read_byte(self.registers.SP);
            }
            Regs::HL => {
                self.registers.L = bus.read_byte(self.registers.SP);
                self.registers.SP += 1;
                self.registers.H = bus.read_byte(self.registers.SP);
            }
            Regs::AF => {
                self.registers.F = bus.read_byte(self.registers.SP);
                self.registers.SP += 1;
                self.registers.A = bus.read_byte(self.registers.SP);

                // clear out lower nibble since it should always be zero
                self.registers.F &= !(1 | 1 << 1 | 1 << 2 | 1 << 3)
            }
            _ => {}
        }

        self.registers.SP += 1;
    }

    fn push(&mut self, reg16: Regs, bus: &mut Bus) {
        match reg16 {
            Regs::BC => {
                self.registers.SP -= 1;
                bus.write_byte(self.registers.SP, self.registers.B);

                self.registers.SP -= 1;
                bus.write_byte(self.registers.SP, self.registers.C);
            }
            Regs::DE => {
                self.registers.SP -= 1;
                bus.write_byte(self.registers.SP, self.registers.D);

                self.registers.SP -= 1;
                bus.write_byte(self.registers.SP, self.registers.E);
            }
            Regs::HL => {
                self.registers.SP -= 1;
                bus.write_byte(self.registers.SP, self.registers.H);

                self.registers.SP -= 1;
                bus.write_byte(self.registers.SP, self.registers.L);
            }
            Regs::AF => {
                self.registers.SP -= 1;
                bus.write_byte(self.registers.SP, self.registers.A);

                self.registers.SP -= 1;
                bus.write_byte(self.registers.SP, self.registers.F);
            }
            _ => {}
        }
    }

    fn call(&mut self, bus: &mut Bus, value: u16) {
        self.registers.SP -= 1;
        bus.write_byte(self.registers.SP, ((self.registers.PC) >> 8) as u8);
        self.registers.SP -= 1;
        bus.write_byte(self.registers.SP, self.registers.PC as u8);

        // println!("SP: {:#08X}", bus.read_16(self.registers.SP));

        self.registers.PC = value;
    }

    fn call_flag(&mut self, bus: &mut Bus, flag: Flag, value: u16, condition: bool) -> u8 {
        if self.registers.get_flag(flag) == condition {
            self.call(bus, value);
            return 6;
        }

        3
    }

    fn ei(&mut self) {
        self.ime = true;
    }

    fn di(&mut self) {
        self.ime = false;
    }

    fn ret(&mut self, bus: &Bus) {
        let lower_byte = bus.read_byte(self.registers.SP);
        let higher_byte = bus.read_byte(self.registers.SP + 1);

        self.registers.PC = (higher_byte as u16) << 8 | lower_byte as u16;
        self.registers.SP += 2;
    }

    fn ret_flag(&mut self, bus: &Bus, flag: Flag, condition: bool) -> u8 {
        if self.registers.get_flag(flag) == condition {
            self.ret(bus);
            return 5;
        }

        2
    }

    fn reti(&mut self, bus: &Bus) {
        self.ei();
        self.ret(bus);
    }

    fn rst(&mut self, bus: &mut Bus, value: u8) {
        self.call(bus, value as u16);
    }

    /// https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
    ///
    /// `F` should really **only** be `std::ops::Add:add` or `std::ops::Sub::sub`
    fn _check_if_half_carry<F: Fn(u8, u8) -> u8>(&self, a: u8, b: u8, op: F) -> bool {
        op(a & 0xF, b & 0xF) & 0x10 == 0x10
    }

    fn _check_if_half_carry_16<F: Fn(u16, u16) -> u16>(&self, a: u16, b: u16, op: F) -> bool {
        op(a & 0xFFF, b & 0xFFF) & 0x100 == 0x100
    }
}
