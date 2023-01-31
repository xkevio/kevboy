use eframe::egui::{Context, Key};

use crate::{
    cpu::interrupts::{Interrupt, InterruptHandler},
    mmu::mmio::MMIO,
};

pub struct Joypad {
    joyp: u8,
    prev_joyp: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ButtonType {
    Action,
    Direction,
    None,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            joyp: 0xCF,
            prev_joyp: 0xCF,
        }
    }
}

impl MMIO for Joypad {
    fn read(&mut self, _address: u16) -> u8 {
        self.joyp
    }

    fn write(&mut self, _address: u16, value: u8) {
        self.joyp = 0xC0 | value | (self.joyp & 0xF); // bit 7 and 6 unused and always 1
    }
}

impl Joypad {
    pub fn tick(&mut self, ctx: &Context, interrupt_handler: &mut InterruptHandler) {
        self.reset_pressed_keys();

        match self.get_button_type() {
            ButtonType::Action => self.handle_key_input(ctx, &[Key::P, Key::O, Key::Q, Key::Enter]),
            ButtonType::Direction => self.handle_key_input(ctx, &[Key::D, Key::A, Key::W, Key::S]),
            _ => {}
        }

        if (self.prev_joyp & 0xF == 0xF) && (self.joyp & 0xF != 0xF) {
            interrupt_handler.request_interrupt(Interrupt::Joypad);
        }

        self.prev_joyp = self.joyp;
    }

    pub fn reset_pressed_keys(&mut self) {
        self.joyp |= 0xF;
    }

    /// `key1`: A or Right
    ///
    /// `key2`: B or Left
    ///
    /// `key3`: Shift or Up
    ///
    /// `key4`: Start or Down
    fn handle_key_input(&mut self, ctx: &Context, keys: &[Key; 4]) {
        for (bit, key) in keys.iter().enumerate() {
            if ctx.input().key_down(*key) {
                self.joyp &= !(0x1 << bit as u8);
            }
        }
    }

    fn get_button_type(&self) -> ButtonType {
        if self.joyp & 0x20 == 0 {
            ButtonType::Action
        } else if self.joyp & 0x10 == 0 {
            ButtonType::Direction
        } else {
            ButtonType::None
        }
    }
}
