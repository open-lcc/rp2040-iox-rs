use embassy_rp::gpio::{AnyPin, Output};
use embassy_rp::i2c;
use embassy_rp::i2c::Async;
use embassy_rp::peripherals::{PWM_CH4, PWM_CH6, USB, I2C0};
use embassy_rp::pwm::{Channel, Pwm};
pub mod apec_r0b;

pub struct IOExpanderBoardIO<'a, LedPwmChT, Cn92ChT, Cn93ChT, Cn94ChT> where LedPwmChT: Channel, Cn92ChT: Channel, Cn93ChT: Channel, Cn94ChT: Channel {
    pub serial_pin: AnyPin,
    pub shift_register_clock_pin: AnyPin,
    pub storage_register_clock_pin: AnyPin,
    pub srclr_pin: Option<AnyPin>,
    pub ng_pin: Option<AnyPin>,
    pub cn9_4_pin: Option<AnyPin>,
    pub cn9_3_pin: Option<AnyPin>,
    pub cn9_2_pin: Option<AnyPin>,
    pub led_pin: Option<AnyPin>,
    pub rp2040_serial_boot_pin: AnyPin,
    pub txs0108e_oe_pin: AnyPin,
    
    pub i2c0: i2c::I2c<'a, I2C0, Async>,
    
    pub led_pwm: Option<Pwm<'a, LedPwmChT>>,
    pub cn9_2_pwm: Option<Pwm<'a, Cn92ChT>>,
    pub cn9_3_pwm: Option<Pwm<'a, Cn93ChT>>,
    pub cn9_4_pwm: Option<Pwm<'a, Cn94ChT>>,
    
    pub usb: USB,
}