use embassy_rp::gpio::{AnyPin, Output};
use embassy_time::Timer;

pub(crate) struct DualC595ShiftRegister<'a> {
    serial_pin: Output<'a, AnyPin>,
    shift_register_clock_pin: Output<'a, AnyPin>,
    storage_register_clock_pin: Output<'a, AnyPin>,
}

impl<'a> DualC595ShiftRegister<'a> {
    pub(crate) fn new(
        serial_pin: Output<'a, AnyPin>,
        shift_register_clock_pin: Output<'a, AnyPin>,
        storage_register_clock_pin: Output<'a, AnyPin>,
    ) -> Self {
        DualC595ShiftRegister {
            serial_pin,
            shift_register_clock_pin,
            storage_register_clock_pin,
        }
    }

    pub(crate) async fn write(&mut self, data: u16) {
        for i in 0..16 {
            match data & (1 << i) != 0 {
                true => self.serial_pin.set_high(),
                false => self.serial_pin.set_low(),
            }
            Timer::after_micros(1).await;
            self.shift_register_clock_pin.set_high();
            Timer::after_micros(1).await;
            self.shift_register_clock_pin.set_low();
        }

        self.serial_pin.set_low();

        self.storage_register_clock_pin.set_high();
        Timer::after_micros(5).await;
        self.storage_register_clock_pin.set_low();
    }

    pub(crate) async fn clear(&mut self) {
        self.write(0).await;
    }
}