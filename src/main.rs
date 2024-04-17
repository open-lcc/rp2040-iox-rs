#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{AnyPin, Level, Output, Pin};
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{PWM_CH4, PWM_CH6, USB};
use embassy_rp::pwm::{Channel, Pwm};
use embassy_rp::usb::{Driver, self};
use embassy_time::Timer;
use embassy_rp::i2c::{self, Async, Config};
use embassy_rp::peripherals::I2C0;
use embedded_hal_async::i2c::I2c;
use log::log;
use {defmt_rtt as _, panic_probe as _};
use iox::analog_input::ads1115::ADS111x;
use crate::iox::analog_input::ads1115::{ADS111xConfig, InputMultiplexer, ProgramableGainAmplifier};
use crate::iox::analog_input::fdc1004;
use crate::iox::analog_input::fdc1004::OutputRate;
use crate::iox::analog_output::PwmSlice;
use crate::iox::binary_output::dual_c595_shift_register::DualC595ShiftRegister;
use crate::iox::binary_output::ShiftRegister;
use libm::logf;

mod iox;
mod board_revisions;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
async fn i2c_task(mut i2c: i2c::I2c<'static, I2C0, Async>) {
    let mut ads1115_1 = ADS111x::new(
        0x48,
        ADS111xConfig::default().pga(ProgramableGainAmplifier::V6_144)
    ).unwrap();
    let mut ads1115_2 = ADS111x::new(
        0x49,
        ADS111xConfig::default().pga(ProgramableGainAmplifier::V6_144)
    ).unwrap();
    let mut fdc1004 = iox::analog_input::fdc1004::FDC1004::new(0x50, OutputRate::SPS100);

    loop {
        let vcc = ads1115_2.read_single_voltage(&mut i2c, Some(InputMultiplexer::AIN0GND)).await.unwrap();
        log::info!("Vcc: {:?}", vcc);

        let v_r_cn5 = ads1115_1.read_single_voltage(&mut i2c, Some(InputMultiplexer::AIN0AIN1)).await.unwrap();
        log::info!("V_R_CN5: {:?}", v_r_cn5);

        let v_r_cn6 = ads1115_1.read_single_voltage(&mut i2c, Some(InputMultiplexer::AIN2AIN3)).await.unwrap();
        log::info!("V_R_CN6: {:?}", v_r_cn6);

        let r1 = 3300f32;
        let r2_cn5 = -1f32*((v_r_cn5*r1)/(v_r_cn5-vcc));
        let r2_cn6 = -1f32*((v_r_cn6*r1)/(v_r_cn6-vcc));

        let c_cn5 = ntc_ohm_to_celsius(r2_cn5, 50000f32, 4016f32);
        let c_cn6 = ntc_ohm_to_celsius(r2_cn6, 50000f32, 4016f32);

        log::info!("R2_CN5: {:?} Ohm, R2_CN6: {:?} Ohm, C CN5: {:?} C CN6: {:?}", r2_cn5, r2_cn6, c_cn5, c_cn6);


        /*        let deviceId = fdc1004.read_u16(&mut i2c, fdc1004::Register::DeviceId).await;
                /*if (deviceId.is_err()) {
                    log::info!("Error reading DeviceId: {:?}", deviceId);
                } else {*/
                    log::info!("DeviceId: {:x}", deviceId.unwrap());
        //        }
         */

        let cap = fdc1004.read_capacitance(&mut i2c, fdc1004::Channel::CIN4).await;
        match cap {
            fdc1004::SuccessfulMeasurement::MeasurementInRange(cap) => log::info!("Cap: {:?}", cap.to_pf()),
            fdc1004::SuccessfulMeasurement::Overflow => log::info!("Overflow"),
            fdc1004::SuccessfulMeasurement::Underflow => log::info!("Underflow"),
        }

        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let board_io = board_revisions::apec_r0b::get_board_io(p);

    let driver = Driver::new(board_io.usb, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let sr = ShiftRegister::new(DualC595ShiftRegister::new(
        Output::new(board_io.serial_pin, Level::Low),
        Output::new(board_io.storage_register_clock_pin, Level::Low),
        Output::new(board_io.shift_register_clock_pin, Level::Low),
    ));

    let mut _srclr: Option<Output<AnyPin>> = None;
    if (board_io.srclr_pin.is_some()) {
        _srclr = Some(Output::new(board_io.srclr_pin.unwrap(), Level::High));
    }
    let mut _ng: Option<Output<AnyPin>> = None;
    if (board_io.ng_pin.is_some()) {
        _ng = Some(Output::new(board_io.ng_pin.unwrap(), Level::Low));
    }

    let _txs108e_oe = Output::new(board_io.txs0108e_oe_pin, Level::High);
    let sr_out = iox::binary_output::ShiftRegisterOutputs::new();
    let led = Output::new(board_io.led_pin.unwrap(), Level::Low);

/*    if (board_io.led_pwm.is_some()) {
        let led_pwm = PwmSlice::new(10_000, 20f32, 20f32, board_io.led_pwm.unwrap());
        unwrap!(spawner.spawn(glow_led(led_pwm)));
    }
    if (board_io.cn9_3_pwm.is_some()) {
        let cn9_3_pwm = PwmSlice::new(10_000, 20f32, 20f32, board_io.cn9_3_pwm.unwrap());
        unwrap!(spawner.spawn(glow_cn6(cn9_3_pwm)));
    }*/

    let mut i2c = board_io.i2c0;
    
    //unwrap!(spawner.spawn(i2c_task(i2c)));
    unwrap!(spawner.spawn(do_stuff(sr, led)));

    loop {
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn glow_cn6(mut led: PwmSlice<'static, PWM_CH6>) {
    let mut counter = 0;
    loop {
        counter += 1;
        led.set_duty_cycle_b(counter as f32 / 10f32);
        Timer::after_millis(10).await;

        if (counter > 300) {
            counter = 0;
        }
    }
}

#[embassy_executor::task]
async fn glow_led(mut led: PwmSlice<'static, PWM_CH4>) {
    let mut counter = 0;
    loop {
        counter += 1;
        led.set_duty_cycle_b((counter % 1000) as f32 / 10f32);
        Timer::after_millis(5).await;
    }
}

#[embassy_executor::task]
async fn do_stuff(
    mut sr: ShiftRegister<'static>,
    mut led: Output<'static, AnyPin>,
) {
    let mut counter = 0;
    loop {
        counter += 1;
        //log::info!("counter: {}", counter);
        
        led.set_low();
        sr.set_output(board_revisions::apec_r0b::shift_register_positions::JP2_FA7, true);
        sr.flush_then_clear().await;

        Timer::after_millis(5000).await;

        led.set_high();
        sr.set_output(board_revisions::apec_r0b::shift_register_positions::JP2_FA8, true);
        sr.flush_then_clear().await;

        Timer::after_millis(5000).await;
    }
}

fn ntc_ohm_to_celsius(ohm: f32, r25: f32, b: f32) -> f32 {
    let ln_ratio = logf(ohm / r25);
    let t_kelvin = 1.0 / (ln_ratio / b + 1.0 / 298.15);
    t_kelvin - 273.15
}