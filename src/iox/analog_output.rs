use embassy_rp::gpio::AnyPin;
use embassy_rp::pwm::{Channel, Config, Pwm};
use fixed::traits::ToFixed;


pub(crate) struct PwmSlice<'a, T: Channel> {
    frequency: u32,
    duty_cycle_a: f32,
    duty_cycle_b: f32,
    pwm: Pwm<'a, T>
}

impl<'a, T: Channel> PwmSlice<'a, T> {
    pub(crate) fn new(frequency: u32, duty_cycle_a: f32, duty_cycle_b: f32, pwm: Pwm<'a, T>) -> Self {
        let mut slice = PwmSlice {
            frequency,
            duty_cycle_a,
            duty_cycle_b,
            pwm
        };
        
        slice.update_pwm_config();
        
        slice
    }
    
    pub(crate) fn set_frequency(&mut self, frequency: u32) {
        self.frequency = frequency;
        self.update_pwm_config();
    }
    
    pub(crate) fn set_duty_cycle_a(&mut self, duty_cycle_a: f32) {
        self.duty_cycle_a = duty_cycle_a;
        self.update_pwm_config();
    }
    
    pub(crate) fn set_duty_cycle_b(&mut self, duty_cycle_b: f32) {
        self.duty_cycle_b = duty_cycle_b;
        self.update_pwm_config();
    }
    
    fn update_pwm_config(&mut self){
        let mut c: Config = Default::default();
        
        let (top, div) = calculate_top_div(self.frequency, false);
        c.top = top;
        c.divider = div.to_fixed();
        
        let duty_a = (self.duty_cycle_a as f32 / 100.0 * top as f32) as u16;
        let duty_b = (self.duty_cycle_b as f32 / 100.0 * top as f32) as u16;
        
        c.compare_a = duty_a;
        c.compare_b = duty_b;
        
        self.pwm.set_config(&c);
    }
}

fn calculate_top_div(freq: u32, phase_correct: bool) -> (u16, fixed::FixedU16<fixed::types::extra::U4>)
{
    let freq_cpu: u32 = 125_000_000;
    
    let div = match freq {
        2000.. => 1,
        200..=2000 => 10,
        20..=200 => 100,
        10..=20 => 200,
        _ => 255
    };
    
    let div_fixed: fixed::FixedU16<fixed::types::extra::U4> = div.to_fixed();
    
    let mut top = (freq_cpu / freq / div) - 1;
    let _actual_frequency = freq_cpu / ((top + 1) * div);
    
    if phase_correct {
        top /= 2;
    }

    (top as u16, div_fixed)
}

/*use embassy_rp::gpio::{AnyPin, Output};
use embassy_rp::pwm::{Channel, Config, Pwm};
use crate::binary_output::{BinaryOutput, SioOutput};

pub(crate) trait AnalogOutput {
    fn set_deferred(&mut self, val: u16);

    fn flush(&mut self);
}

pub(crate) struct PwmOutput<'a, T: Channel> {
    deferred_val: u16,
    pwm: Pwm<'a, T>,
}

impl<'a, T: Channel> PwmOutput<'a, T> {
    pub(crate) fn new(pwm: Pwm<'a, T>) -> Self {
        PwmOutput {
            deferred_val: 0,
            pwm
        }
    }
}

impl<T: Channel> AnalogOutput for PwmOutput<'_, T> {
    fn set_deferred(&mut self, val: u16) {
        self.deferred_val = val;
    }

    fn flush(&mut self) {
        self.pwm.set_config()
    }
}
*/


/*    let p = embassy_rp::init(Default::default());

    let mut c: Config = Default::default();
    c.top = 0x8000;
    c.compare_b = 8;
    let mut pwm = Pwm::new_output_b(p.PWM_CH4, p.PIN_25, c.clone());

    loop {
        info!("current LED duty cycle: {}/32768", c.compare_b);
        Timer::after_secs(1).await;
        c.compare_b = c.compare_b.rotate_left(4);
        pwm.set_config(&c);
    }*/