use std::{fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    cpu::{cpu::CPU, registers::Registers},
    mmu::bus::Bus,
};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonTest {
    name: String,
    initial: State,
    r#final: State,
    cycles: Vec<Value>,
}

impl JsonTest {
    pub fn run(&self, opcode: u8, cb: bool) -> (bool, State, State) {
        let opcode = if cb {
            u16::from_be_bytes([0xCB, opcode])
        } else {
            opcode as u16
        };

        let mut cpu = CPU::new();
        cpu.registers = self.initial.clone().into();

        let mut bus = Bus::new(&self.initial.ram);
        cpu.tick(&mut bus, Some(opcode));

        let after_state = State {
            pc: cpu.registers.PC,
            sp: cpu.registers.SP,
            a: cpu.registers.A,
            b: cpu.registers.B,
            c: cpu.registers.C,
            d: cpu.registers.D,
            e: cpu.registers.E,
            f: cpu.registers.F,
            h: cpu.registers.H,
            l: cpu.registers.L,
            ime: None,
            ie: None,
            ram: bus.space,
        };

        (
            after_state == self.r#final,
            after_state.clone(),
            self.r#final.clone(),
        )
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct State {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    #[serde(skip_deserializing, default = "empty")]
    ime: Option<u8>,
    #[serde(skip_deserializing, default = "empty")]
    ie: Option<u8>,
    ram: Vec<Vec<u16>>,
}

impl From<State> for Registers {
    fn from(value: State) -> Self {
        Self {
            A: value.a,
            F: value.f,
            B: value.b,
            C: value.c,
            D: value.d,
            E: value.e,
            H: value.h,
            L: value.l,
            SP: value.sp,
            PC: value.pc,
        }
    }
}

pub fn parse_tests<P: AsRef<Path>>(json_file: P) -> Vec<JsonTest> {
    let file = File::open(json_file).unwrap();
    let reader = BufReader::new(file);

    let tests: Value = serde_json::from_reader(reader).unwrap();
    let test_array = tests.as_array().unwrap();

    test_array
        .iter()
        .map(|j| serde_json::from_value(j.clone()).unwrap())
        .collect::<Vec<_>>()
}

pub fn empty() -> Option<u8> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x01() {
        let mut passed = true;

        let tests = parse_tests("01.json");
        for test in tests {
            let (test_pass, is_state, should_state) = test.run(0x01, false);
            passed &= test_pass;
            if !passed {
                dbg!(is_state);
                dbg!(should_state);
                break;
            }
        }

        assert_eq!(passed, true)
    }
}
