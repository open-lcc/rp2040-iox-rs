use embassy_rp::gpio;
use embassy_rp::gpio::AnyPin;
use gpio::Output;
use crate::iox::binary_output::dual_c595_shift_register::DualC595ShiftRegister;
use crate::iox::Flushable;
use core::cell::{Cell, RefCell};

pub(crate) mod dual_c595_shift_register;

pub(crate) trait BinaryOutput {
    fn set_deferred(&mut self, val: bool);
}

pub(crate) struct SioOutput<'a> {
    deferred_val: bool,
    pin: Output<'a, AnyPin>,
}

impl<'a> SioOutput<'a> {
    pub(crate) fn new(pin: Output<'a, AnyPin>) -> Self {
        SioOutput {
            deferred_val: false,
            pin,
        }
    }
}

impl Flushable for SioOutput<'_> {
    async fn flush(&mut self) {
        match self.deferred_val {
            true => self.pin.set_high(),
            false => self.pin.set_low(),
        }
    }
}

impl BinaryOutput for SioOutput<'_> {
    fn set_deferred(&mut self, val: bool) {
        self.deferred_val = val;
    }
}

pub(crate) struct ShiftRegister<'a> {
    out: ShiftRegisterOutputs,
    reg: DualC595ShiftRegister<'a>
}

impl<'a> ShiftRegister<'a> {
    pub(crate) fn new(reg: DualC595ShiftRegister<'a>) -> Self {
        ShiftRegister {
            out: ShiftRegisterOutputs::new(),
            reg
        }
    }
    
    pub(crate) fn clear(&mut self) {
        self.out.clear();
    }
    
    pub(crate) async fn clear_then_flush(&mut self) {
        self.clear();
        self.flush().await;
    }
    
    pub(crate) async fn flush_then_clear(&mut self) {
        self.flush().await;
        self.clear();
    }
    
    pub(crate) fn set_all_outputs(&mut self, value: bool) {
        for i in 0..16 {
            self.set_output(i, value);
        }
    }

    pub(crate) fn set_output(&mut self, index: usize, value: bool) {
        self.out.set_output(index, value);
    }
    
    pub(crate) async fn flush(&mut self) {
        self.reg.write(self.out.get_value()).await; 
    }
}

#[derive(Debug)]
pub(crate) struct ShiftRegisterOutputs {
    outputs: [bool; 16]
}

fn bitmask_for_index(index: usize) -> u16 {
    1 << index
}

impl ShiftRegisterOutputs {
    pub(crate) fn new() -> Self {
        let mut outputs = [false; 16];

        ShiftRegisterOutputs {
            outputs
        }
    }
    
    pub(crate) fn set_output(&mut self, index: usize, value: bool) {
        self.outputs[index] = value;
    }
    
    pub(crate) fn clear(&mut self) {
        self.outputs = [false; 16];
    }

    pub(crate) fn get_value(&self) -> u16 {
        let mut value = 0;
        for i in 0..16 {
            if self.outputs[i] {
                value |= bitmask_for_index(i);
            }
        }

        value
    }
}