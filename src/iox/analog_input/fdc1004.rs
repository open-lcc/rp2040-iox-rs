#![no_std]

use core::fmt;
use defmt::export::u8;
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c;
use ux::i24;

#[derive(Debug)]
pub enum FDCError<E>{
    MeasurementNotComplete,
    I2CError(E)
}

#[derive(Default, Copy, Clone, Debug)]
pub enum OutputRate {
    #[default]
    SPS100,
    SPS200,
    SPS400,
}

static CAPDAC_MAX: u8 = 0x1F;

#[derive(Copy, Clone, Debug)]
pub enum Channel {
    CIN1,
    CIN2,
    CIN3,
    CIN4,
    CAPDAC,
    DISABLED
}

#[derive(Copy, Clone)]
pub enum Measurement {
    Measurement1,
    Measurement2,
    Measurement3,
    Measurement4,
}

#[derive(Copy, Clone, Debug)]
pub enum Register {
    Measurement1MSB,
    Measurement1LSB,
    Measurement2MSB,
    Measurement2LSB,
    Measurement3MSB,
    Measurement3LSB,
    Measurement4MSB,
    Measurement4LSB,
    Measurement1Config,
    Measurement2Config,
    Measurement3Config,
    Measurement4Config,
    FdcConf,
    OffsetCalCIN1,
    OffsetCalCIN2,
    OffsetCalCIN3,
    OffsetCalCIN4,
    GainCalCIN1,
    GainCalCIN2,
    GainCalCIN3,
    GainCalCIN4,
    ManufacturerId,
    DeviceId,
}

impl Register {
    pub(crate) fn to_u8(&self) -> u8 {
        match self {
            Register::Measurement1MSB => 0x00,
            Register::Measurement1LSB => 0x01,
            Register::Measurement2MSB => 0x02,
            Register::Measurement2LSB => 0x03,
            Register::Measurement3MSB => 0x04,
            Register::Measurement3LSB => 0x05,
            Register::Measurement4MSB => 0x06,
            Register::Measurement4LSB => 0x07,
            Register::Measurement1Config => 0x08,
            Register::Measurement2Config => 0x09,
            Register::Measurement3Config => 0x0A,
            Register::Measurement4Config => 0x0B,
            Register::FdcConf => 0x0C,
            Register::OffsetCalCIN1 => 0x0D,
            Register::OffsetCalCIN2 => 0x0E,
            Register::OffsetCalCIN3 => 0x0F,
            Register::OffsetCalCIN4 => 0x10,
            Register::GainCalCIN1 => 0x11,
            Register::GainCalCIN2 => 0x12,
            Register::GainCalCIN3 => 0x13,
            Register::GainCalCIN4 => 0x14,
            Register::ManufacturerId => 0xFE,
            Register::DeviceId => 0xFF,
        }
    }
}

static FDC_REGISTER: u8 = 0x0C;

static ATTOFARADS_UPPER_WORD:i32 = 488;
static PICOFARADS_PER_CAPDAC: f32 = 3.125;

pub(crate) enum SuccessfulMeasurement {
    MeasurementInRange(MeasuredCapacitance),
    Underflow,
    Overflow,
}

pub(crate) struct MeasuredCapacitance {
    pub(crate) value: i24,
    pub(crate) capdac: u8,
}

impl MeasuredCapacitance {
    pub(crate) fn new(value: i24, capdac: u8) -> Self {
        MeasuredCapacitance {
            value,
            capdac,
        }
    }

    pub(crate) fn to_weird(&self) -> f32 {
        let val: i32 = self.value.into();

        let mut cap = val * ATTOFARADS_UPPER_WORD;
        cap /= 1000;
        cap += self.capdac as i32 * 3125i32;

        cap as f32
    }

    pub(crate) fn to_pf(&self) -> f32 {
        let vali32 : i32 = self.value.into();
        let val: f32 = vali32 as f32;

        let mut pf = val / 524_288f32;

        pf += PICOFARADS_PER_CAPDAC * (self.capdac as f32);
        pf
    }
}

#[derive(Debug)]
struct MeasurementConfiguration {
    channel_a: Channel,
    channel_b: Channel,
    offset_capacitance: u8,
}

impl MeasurementConfiguration {
    pub(crate) fn new(channel_a: Channel, channel_b: Channel, offset_capacitance: u8) -> Self {
        MeasurementConfiguration {
            channel_a,
            channel_b,
            offset_capacitance,
        }
    }

    pub(crate) fn to_u16(&self) -> u16 {
        let mut val = 0;

        val |= match self.channel_a {
            Channel::CIN1 => 0x0000,
            Channel::CIN2 => 0x2000,
            Channel::CIN3 => 0x4000,
            Channel::CIN4 => 0x6000,
            _ => 0x0000,
        };

        val |= match self.channel_b {
            Channel::CIN1 => 0x0000,
            Channel::CIN2 => 0x0400,
            Channel::CIN3 => 0x0800,
            Channel::CIN4 => 0x0C00,
            Channel::CAPDAC => 0x1000,
            Channel::DISABLED => 0x1C00,
        };

        val |= (self.offset_capacitance as u16) << 5;

        val
    }
}

#[derive(Default, Debug)]
struct FDCConfiguration {
    reset: bool,
    rate: OutputRate,
    repeat: bool,
    initiate_measurement1: bool,
    initiate_measurement2: bool,
    initiate_measurement3: bool,
    initiate_measurement4: bool,
    measurement1_done: bool,
    measurement2_done: bool,
    measurement3_done: bool,
    measurement4_done: bool,
}

impl FDCConfiguration {
    pub(crate) fn from_u16(d: u16) -> Self {
        let mut config = FDCConfiguration::default();

        config.reset = d & (1u16 << 15) != 0;
        config.rate = match d & 0x0C00 {
            0x0400 => OutputRate::SPS100,
            0x0800 => OutputRate::SPS200,
            0x0C00 => OutputRate::SPS400,
            _ => OutputRate::SPS100,
        };
        config.repeat = d & (1u16 << 8) != 0;
        config.initiate_measurement1 = d & (1u16 << 7) != 0;
        config.initiate_measurement2 = d & (1u16 << 6) != 0;
        config.initiate_measurement3 = d & (1u16 << 5) != 0;
        config.initiate_measurement4 = d & (1u16 << 4) != 0;
        config.measurement1_done = d & (1u16 << 3) != 0;
        config.measurement2_done = d & (1u16 << 2) != 0;
        config.measurement3_done = d & (1u16 << 1) != 0;
        config.measurement4_done = d & (1u16 << 0) != 0;

        config
    }

    pub(crate) fn rate(&mut self, rate: OutputRate) -> &mut Self {
        self.rate = rate;
        self
    }

    pub(crate) fn initiate_measurement1(&mut self, initiate: bool) -> &mut Self {
        self.initiate_measurement1 = initiate;
        self
    }

    pub(crate) fn initiate_measurement2(&mut self, initiate: bool) -> &mut Self {
        self.initiate_measurement2 = initiate;
        self
    }

    pub(crate) fn initiate_measurement3(&mut self, initiate: bool) -> &mut Self {
        self.initiate_measurement3 = initiate;
        self
    }

    pub(crate) fn initiate_measurement4(&mut self, initiate: bool) -> &mut Self {
        self.initiate_measurement4 = initiate;
        self
    }

    pub(crate) fn reset(&mut self, reset: bool) -> &mut Self {
        self.reset = reset;
        self
    }

    pub(crate) fn repeat(&mut self, repeat: bool) -> &mut Self {
        self.reset = repeat;
        self
    }

    pub(crate) fn to_u16(&self) -> u16 {
        let mut val = 0;

        val |= if self.reset { 1u16 << 15 } else { 0x0000 };
        val |= match self.rate {
            OutputRate::SPS100 => 0x0400,
            OutputRate::SPS200 => 0x0800,
            OutputRate::SPS400 => 0x0C00,
        };
        val |= if self.repeat { 1u16 << 8 } else { 0x0000 };
        val |= if self.initiate_measurement1 { 1u16 << 7 } else { 0x0000 };
        val |= if self.initiate_measurement2 { 1u16 << 6 } else { 0x0000 };
        val |= if self.initiate_measurement3 { 1u16 << 5 } else { 0x0000 };
        val |= if self.initiate_measurement4 { 1u16 << 4 } else { 0x0000 };
        val |= if self.measurement1_done { 1u16 << 3 } else { 0x0000 };
        val |= if self.measurement2_done { 1u16 << 2 } else { 0x0000 };
        val |= if self.measurement3_done { 1u16 << 1 } else { 0x0000 };
        val |= if self.measurement4_done { 1u16 << 0 } else { 0x0000 };

        val
    }
}

pub (crate) struct FDC1004 {
    address: u8,
    output_rate: OutputRate,
}

impl FDC1004 {
    pub(crate) fn new(address: u8, output_rate: OutputRate) -> Self {
        FDC1004 {
            address,
            output_rate,
        }
    }

    pub(crate) async fn read_capacitance<I2C, E>(&mut self, i2c: &mut I2C, channel: Channel) -> SuccessfulMeasurement where I2C: I2c<Error = E>, E: fmt::Debug {
        let mut capdac: u8 = 0x00;

        loop {
            let m = self.measure_channel(i2c, channel, capdac).await.unwrap();
//            log::info!("Measurment: {:?} Capdac: {:?}", m, capdac);
            if m < i24::max_value() && m > i24::min_value() {
                return SuccessfulMeasurement::MeasurementInRange(MeasuredCapacitance::new(m, capdac));
            }

            if m == i24::max_value() && capdac < CAPDAC_MAX {
                capdac += 1;
            } else if m == i24::min_value() && capdac > 0 {
                capdac -= 1;
            } else {
                return match capdac {
                    0 => SuccessfulMeasurement::Underflow,
                    _ => SuccessfulMeasurement::Overflow
                };
            }
        }


        //m as f32 / 0x80_0000 as f32
        return SuccessfulMeasurement::Overflow;
    }

    pub(crate) async fn measure_channel<I2C, E>(&mut self, i2c: &mut I2C, channel: Channel, capdac: u8) -> Result<i24, FDCError<E>> where I2C: I2c<Error = E>, E: fmt::Debug {
        let measurement = match channel {
            Channel::CIN1 => Measurement::Measurement1,
            Channel::CIN2 => Measurement::Measurement2,
            Channel::CIN3 => Measurement::Measurement3,
            Channel::CIN4 => Measurement::Measurement4,
            _ => Measurement::Measurement1,
        };

        self.configure_single_measurement(i2c, channel, measurement.clone(), capdac).await;
        self.trigger_single_measurement(i2c, measurement.clone()).await;
        Timer::after_millis(self.sample_delay()).await;

        return self.read_measurement(i2c, measurement).await;
    }

    pub(crate) async fn configure_single_measurement<I2C, E>(&mut self, i2c: &mut I2C, channel: Channel, measurement: Measurement, capdac: u8) where I2C: I2c<Error = E>, E: fmt::Debug {
        let mut config = MeasurementConfiguration::new(channel, Channel::CAPDAC, capdac);

        let reg = match measurement {
            Measurement::Measurement1 => Register::Measurement1Config,
            Measurement::Measurement2 => Register::Measurement2Config,
            Measurement::Measurement3 => Register::Measurement3Config,
            Measurement::Measurement4 => Register::Measurement4Config,
        };

        self.write_u16(i2c, reg, config.to_u16()).await;
    }

    pub(crate) async fn trigger_single_measurement<I2C, E>(&mut self, i2c: &mut I2C, measurement: Measurement) where I2C: I2c<Error = E>, E: fmt::Debug {
        let mut config = FDCConfiguration::default();
        let mut config = config.rate(self.output_rate);
        let mut config = match measurement {
            Measurement::Measurement1 => config.initiate_measurement1(true),
            Measurement::Measurement2 => config.initiate_measurement2(true),
            Measurement::Measurement3 => config.initiate_measurement3(true),
            Measurement::Measurement4 => config.initiate_measurement4(true),
        };

        self.write_u16(i2c, Register::FdcConf, config.to_u16()).await;
    }

    pub(crate) async fn read_measurement<I2C, E>(&mut self, i2c: &mut I2C, measurement: Measurement) -> Result<i24, FDCError<E>> where I2C: I2c<Error = E>, E: fmt::Debug {
        let config = FDCConfiguration::from_u16(self.read_u16(i2c, Register::FdcConf).await.unwrap());

        let ready = match measurement {
            Measurement::Measurement1 => config.measurement1_done,
            Measurement::Measurement2 => config.measurement2_done,
            Measurement::Measurement3 => config.measurement3_done,
            Measurement::Measurement4 => config.measurement4_done,
        };

        if (!ready) {
            return Err(FDCError::MeasurementNotComplete);
        }

        let msb = self.read_u16(i2c, match measurement {
            Measurement::Measurement1 => Register::Measurement1MSB,
            Measurement::Measurement2 => Register::Measurement2MSB,
            Measurement::Measurement3 => Register::Measurement3MSB,
            Measurement::Measurement4 => Register::Measurement4MSB,
        }).await.unwrap() as i32;

        let lsb = self.read_u16(i2c, match measurement {
            Measurement::Measurement1 => Register::Measurement1LSB,
            Measurement::Measurement2 => Register::Measurement2LSB,
            Measurement::Measurement3 => Register::Measurement3LSB,
            Measurement::Measurement4 => Register::Measurement4LSB,
        }).await.unwrap() as i32;

        let mut val24 = i24::default();
        val24 |= i24::new(msb) << 8;
        val24 |= i24::new(lsb) >> 8;

        Ok(val24)
    }

    pub(crate) async fn write_u16<I2C, E>(&mut self, i2c: &mut I2C, reg: Register, data: u16) where I2C: I2c<Error = E>, E: fmt::Debug {
        let data = data.to_be_bytes();
        i2c.write(self.address, &[reg.to_u8(), data[0], data[1]]).await.unwrap();
    }

    pub(crate) async fn read_u16<I2C, E>(&mut self, i2c: &mut I2C, reg: Register) -> Result<u16, E> where I2C: I2c<Error = E>, E: fmt::Debug {
        let mut data: [u8; 2] = [0,0];
        let res = i2c.write_read(self.address, &[reg.to_u8()], &mut data).await;

        if res.is_err() {
            return Err(res.err().unwrap());
        }

        let be = u16::from_be_bytes(data);

        return Ok(be);
    }

    fn sample_delay(&self) -> u64 {
        match self.output_rate {
            OutputRate::SPS100 => 11,
            OutputRate::SPS200 => 6,
            OutputRate::SPS400 => 3,
        }
    }
}

