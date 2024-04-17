use embassy_rp::gpio::Pin;
use embassy_rp::{i2c, Peripherals};
use embassy_rp::i2c::Config;
use embassy_rp::peripherals::{PWM_CH4, PWM_CH6, PWM_CH7};
use embassy_rp::pwm::Pwm;
use crate::board_revisions::IOExpanderBoardIO;
use crate::Irqs;

pub mod shift_register_positions {
    pub const CN1_3V3: usize = 0;
    pub const CN1_12V: usize = 1;
    pub const VOUT4: usize = 2;
    pub const VOUT1: usize = 3;
    pub const VOUT3: usize = 4;
    pub const VOUT2: usize = 5;
    pub const CN10: usize = 6;
    pub const CN11: usize = 7;
    pub const CN9_8: usize = 8;
    pub const CN9_6: usize = 9;
    pub const CN4_4: usize = 10;
    const X1: usize = 11;
    pub const JP2_FA7: usize = 12;
    pub const JP2_FA8: usize = 13;
    pub const JP2_FA9: usize = 14;
    pub const JP2_FA10: usize = 15;
}

pub mod pin_functions {
    pub const ESP32_RP2040_UART_TX: usize = 4;
    pub const ESP32_RP2040_UART_RX: usize = 5;
    pub const QWIIC_SDA: usize = 6;
    pub const QWIIC_SCL: usize = 7;
    pub const INT_SDA: usize = 8;
    pub const INT_SCL: usize = 9;
    pub const SR_SRCK: usize = 10;
    pub const RP2040_SERIAL_BOOT: usize = 15;
    pub const CN1_UART_TX: usize = 16;
    pub const CN1_UART_RX: usize = 17;
}

pub(crate) fn get_board_io<'a>(p: Peripherals) -> IOExpanderBoardIO<'a, PWM_CH4, PWM_CH7, PWM_CH6, PWM_CH6> {
    let sda = p.PIN_8;
    let scl = p.PIN_9;

    let mut i2c = i2c::I2c::new_async(p.I2C0, scl, sda, Irqs, Config::default());
    
    let mut io_expander = IOExpanderBoardIO {
        serial_pin: p.PIN_0.degrade(),
        shift_register_clock_pin: p.PIN_3.degrade(),
        storage_register_clock_pin: p.PIN_10.degrade(),
        srclr_pin: Some(p.PIN_1.degrade()),
        ng_pin: Some(p.PIN_2.degrade()),
        cn9_4_pin: Some(p.PIN_12.degrade()),
        cn9_3_pin: None,
        cn9_2_pin: Some(p.PIN_14.degrade()),
        led_pin: Some(p.PIN_25.degrade()),
        rp2040_serial_boot_pin: p.PIN_15.degrade(),
        txs0108e_oe_pin: p.PIN_24.degrade(),
        i2c0: i2c,
        led_pwm: None, //Some(Pwm::new_output_b(p.PWM_CH4, p.PIN_25, Default::default())),
        cn9_2_pwm: None,
        cn9_3_pwm: Some(Pwm::new_output_b(p.PWM_CH6, p.PIN_13, Default::default())),
        cn9_4_pwm: None,
        usb: p.USB,
    };

    io_expander
}