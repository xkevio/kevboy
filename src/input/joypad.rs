use eframe::egui::{Context, Key};

use crate::{mmu::mmio::MMIO, cpu::interrupts::{InterruptHandler, Interrupt}};

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
        if self.get_button_type() == Some(ButtonType::Action) {
            // A
            if ctx.input().key_down(Key::O) {
                if self.joyp & 0x1 != 0 {
                    println!("A was pressed!");
                    // interrupt_handler.request_interrupt(Interrupt::Joypad);
                }

                self.joyp &= !(0x1);
            }

            // B
            if ctx.input().key_down(Key::P) {
                if self.joyp & 0x2 != 0 {
                    println!("B was pressed!");
                    // interrupt_handler.request_interrupt(Interrupt::Joypad);
                }

                self.joyp &= !(0x2);
            }

            // Select
            if ctx.input().key_down(Key::Q) {
                if self.joyp & 0x4 != 0 {
                    println!("Select was pressed!");
                    // interrupt_handler.request_interrupt(Interrupt::Joypad);
                }

                self.joyp &= !(0x4);
            }

            // Start
            if ctx.input().key_down(Key::Enter) {
                if self.joyp & 0x8 != 0 {
                    println!("Start was pressed!");
                    // interrupt_handler.request_interrupt(Interrupt::Joypad);
                }

                self.joyp &= !(0x8);
            }
        }

        if self.get_button_type() == Some(ButtonType::Direction) {
            // Right
            if ctx.input().key_down(Key::D) {
                if self.joyp & 0x1 != 0 {
                    println!("Right was pressed!");
                    // interrupt_handler.request_interrupt(Interrupt::Joypad);
                }

                self.joyp &= !(0x1);
            }

            // B
            if ctx.input().key_down(Key::A) {
                if self.joyp & 0x2 != 0 {
                    println!("Left was pressed!");
                    // interrupt_handler.request_interrupt(Interrupt::Joypad);
                }

                self.joyp &= !(0x2);
            }

            // Select
            if ctx.input().key_down(Key::W) {
                if self.joyp & 0x4 != 0 {
                    println!("Up was pressed!");
                    // interrupt_handler.request_interrupt(Interrupt::Joypad);
                }

                self.joyp &= !(0x4);
            }

            // Down
            if ctx.input().key_down(Key::S) {
                if self.joyp & 0x8 != 0 {
                    println!("Down was pressed!");
                    // interrupt_handler.request_interrupt(Interrupt::Joypad);
                }

                self.joyp &= !(0x8);
            }
        }
    }

    pub fn reset_pressed_keys(&mut self) {
        self.joyp |= 0xF;
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
