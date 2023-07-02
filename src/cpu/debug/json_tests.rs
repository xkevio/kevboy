use std::{fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    cpu::{cpu::CPU, registers::Registers},
    mmu::bus::Bus,
};

macro_rules! generate_tests {
    () => {};
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonTest {
    name: String,
    initial: State,
    r#final: State,
    cycles: Vec<Value>,
}

impl JsonTest {
    pub fn run(&self) -> (bool, State, State) {
        let mut cpu = CPU::new();
        cpu.registers = self.initial.clone().into();

        let mut bus = Bus::new(&self.initial.ram);
        cpu.tick(&mut bus);

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

    macro_rules! generate_tests {
        ($($n:ident),*) => {
            $(
                #[test]
                fn $n() {
                    let mut passed = true;

                    let tests = parse_tests("json_tests/".to_string() + &stringify!($n).replace('x', "") + ".json");
                    for test in tests {
                        let (test_pass, is_state, should_state) = test.run();
                        passed &= test_pass;
                        if !passed {
                            dbg!(is_state);
                            dbg!(should_state);
                            break;
                        }
                    }

                    assert_eq!(passed, true)
                }
            )*
        };
    }

    generate_tests!(
        xcb00,
        xcb01,
        xcb02,
        xcb03,
        xcb04,
        xcb05,
        xcb06,
        xcb07,
        xcb08,
        xcb09,
        xcb0a,
        xcb0b,
        xcb0c,
        xcb0d,
        xcb0e,
        xcb0f,
        xcb10,
        xcb11,
        xcb12,
        xcb13,
        xcb14,
        xcb15,
        xcb16,
        xcb17,
        xcb18,
        xcb19,
        xcb1a,
        xcb1b,
        xcb1c,
        xcb1d,
        xcb1e,
        xcb1f,
        xcb20,
        xcb21,
        xcb22,
        xcb23,
        xcb24,
        xcb25,
        xcb26,
        xcb27,
        xcb28,
        xcb29,
        xcb2a,
        xcb2b,
        xcb2c,
        xcb2d,
        xcb2e,
        xcb2f,
        xcb30,
        xcb31,
        xcb32,
        xcb33,
        xcb34,
        xcb35,
        xcb36,
        xcb37,
        xcb38,
        xcb39,
        xcb3a,
        xcb3b,
        xcb3c,
        xcb3d,
        xcb3e,
        xcb3f,
        xcb40,
        xcb41,
        xcb42,
        xcb43,
        xcb44,
        xcb45,
        xcb46,
        xcb47,
        xcb48,
        xcb49,
        xcb4a,
        xcb4b,
        xcb4c,
        xcb4d,
        xcb4e,
        xcb4f,
        xcb50,
        xcb51,
        xcb52,
        xcb53,
        xcb54,
        xcb55,
        xcb56,
        xcb57,
        xcb58,
        xcb59,
        xcb5a,
        xcb5b,
        xcb5c,
        xcb5d,
        xcb5e,
        xcb5f,
        xcb60,
        xcb61,
        xcb62,
        xcb63,
        xcb64,
        xcb65,
        xcb66,
        xcb67,
        xcb68,
        xcb69,
        xcb6a,
        xcb6b,
        xcb6c,
        xcb6d,
        xcb6e,
        xcb6f,
        xcb70,
        xcb71,
        xcb72,
        xcb73,
        xcb74,
        xcb75,
        xcb76,
        xcb77,
        xcb78,
        xcb79,
        xcb7a,
        xcb7b,
        xcb7c,
        xcb7d,
        xcb7e,
        xcb7f,
        xcb80,
        xcb81,
        xcb82,
        xcb83,
        xcb84,
        xcb85,
        xcb86,
        xcb87,
        xcb88,
        xcb89,
        xcb8a,
        xcb8b,
        xcb8c,
        xcb8d,
        xcb8e,
        xcb8f,
        xcb90,
        xcb91,
        xcb92,
        xcb93,
        xcb94,
        xcb95,
        xcb96,
        xcb97,
        xcb98,
        xcb99,
        xcb9a,
        xcb9b,
        xcb9c,
        xcb9d,
        xcb9e,
        xcb9f,
        xcba0,
        xcba1,
        xcba2,
        xcba3,
        xcba4,
        xcba5,
        xcba6,
        xcba7,
        xcba8,
        xcba9,
        xcbaa,
        xcbab,
        xcbac,
        xcbad,
        xcbae,
        xcbaf,
        xcbb0,
        xcbb1,
        xcbb2,
        xcbb3,
        xcbb4,
        xcbb5,
        xcbb6,
        xcbb7,
        xcbb8,
        xcbb9,
        xcbba,
        xcbbb,
        xcbbc,
        xcbbd,
        xcbbe,
        xcbbf,
        xcbc0,
        xcbc1,
        xcbc2,
        xcbc3,
        xcbc4,
        xcbc5,
        xcbc6,
        xcbc7,
        xcbc8,
        xcbc9,
        xcbca,
        xcbcb,
        xcbcc,
        xcbcd,
        xcbce,
        xcbcf,
        xcbd0,
        xcbd1,
        xcbd2,
        xcbd3,
        xcbd4,
        xcbd5,
        xcbd6,
        xcbd7,
        xcbd8,
        xcbd9,
        xcbda,
        xcbdb,
        xcbdc,
        xcbdd,
        xcbde,
        xcbdf,
        xcbe0,
        xcbe1,
        xcbe2,
        xcbe3,
        xcbe4,
        xcbe5,
        xcbe6,
        xcbe7,
        xcbe8,
        xcbe9,
        xcbea,
        xcbeb,
        xcbec,
        xcbed,
        xcbee,
        xcbef,
        xcbf0,
        xcbf1,
        xcbf2,
        xcbf3,
        xcbf4,
        xcbf5,
        xcbf6,
        xcbf7,
        xcbf8,
        xcbf9,
        xcbfa,
        xcbfb,
        xcbfc,
        xcbfd,
        xcbfe,
        xcbff
    );
}
