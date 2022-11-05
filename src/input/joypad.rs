use eframe::egui::{Context, Key};

use crate::{
    cpu::interrupts::{Interrupt, InterruptHandler},
    mmu::mmio::MMIO,
};

#[derive(Default)]
pub struct Joypad {
    joyp: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ButtonType {
    Action,
    Direction,
}

impl MMIO for Joypad {
    fn read(&self, _address: u16) -> u8 {
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
            Some(ButtonType::Action) => {
                self.handle_key_input(ctx, interrupt_handler, Key::P, Key::O, Key::Q, Key::Enter)
            }
            Some(ButtonType::Direction) => {
                self.handle_key_input(ctx, interrupt_handler, Key::D, Key::A, Key::W, Key::S)
            }
            None => {}
        }
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
    fn handle_key_input(
        &mut self,
        ctx: &Context,
        interrupt_handler: &mut InterruptHandler,
        key1: Key,
        key2: Key,
        key3: Key,
        key4: Key,
    ) {
        if ctx.input().key_down(key1) {
            if self.joyp & 0x1 != 0 {
                interrupt_handler.request_interrupt(Interrupt::Joypad);
            }

            self.joyp &= !(0x1);
        }

        if ctx.input().key_down(key2) {
            if self.joyp & 0x2 != 0 {
                interrupt_handler.request_interrupt(Interrupt::Joypad);
            }

            self.joyp &= !(0x2);
        }

        if ctx.input().key_down(key3) {
            if self.joyp & 0x4 != 0 {
                interrupt_handler.request_interrupt(Interrupt::Joypad);
            }

            self.joyp &= !(0x4);
        }

        if ctx.input().key_down(key4) {
            if self.joyp & 0x8 != 0 {
                interrupt_handler.request_interrupt(Interrupt::Joypad);
            }

            self.joyp &= !(0x8);
        }
    }

    fn get_button_type(&self) -> Option<ButtonType> {
        if self.joyp & 0x20 == 0 {
            Some(ButtonType::Action)
        } else if self.joyp & 0x10 == 0 {
            Some(ButtonType::Direction)
        } else {
            None
        }
    }
}
