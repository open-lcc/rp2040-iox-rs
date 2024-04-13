#![no_std]

use embedded_hal_async::i2c::I2c;

static CONVERSION_REGISTER : u8 = 0b00;
static CONFIG_REGISTER     : u8 = 0b01;
static LO_THRESH_REGISTER  : u8 = 0b10;
static HI_THRESH_REGISTER  : u8 = 0b11;

#[derive(Debug)]
pub enum ADSError{
    WrongAddress,
    ModeSetToSingle,
}

#[derive(Debug)]
enum OSW{
    StartConversion,
    Idle,
}

#[derive(Debug,PartialEq)]
enum OSR{
    PerformingConversion,
    DeviceIdle,
}

impl OSW {
    fn bits(&self) -> u16{
        match self {
            OSW::StartConversion => 1<<15,
            OSW::Idle => 0,
        }
    }
}

impl Default for OSW{
    fn default() -> Self {
        OSW::Idle
    }
}

impl OSR {
    fn from_bits(bits: u16) -> Self{
        match bits >> 15 & 0b01 {
            0 => OSR::PerformingConversion,
            _ => OSR::DeviceIdle,
        }
    }
}

impl Default for OSR{
    fn default() -> Self {
        OSR::DeviceIdle
    }
}

#[derive(Debug)]
pub enum InputMultiplexer{
    AIN0AIN1,
    AIN0AIN3,
    AIN1AIN3,
    AIN2AIN3,
    AIN0GND,
    AIN1GND,
    AIN2GND,
    AIN3GND,
}

impl InputMultiplexer {
    fn bits(&self) -> u16{
        match self {
            InputMultiplexer::AIN0AIN1 => 0b000 << 12,
            InputMultiplexer::AIN0AIN3 => 0b001 << 12,
            InputMultiplexer::AIN1AIN3 => 0b010 << 12,
            InputMultiplexer::AIN2AIN3 => 0b011 << 12,
            InputMultiplexer::AIN0GND  => 0b100 << 12,
            InputMultiplexer::AIN1GND  => 0b101 << 12,
            InputMultiplexer::AIN2GND  => 0b110 << 12,
            InputMultiplexer::AIN3GND  => 0b111 << 12,
        }
    }

    fn from_bits(bits: u16) -> Self{
        match bits >> 12 & 0b111{
            0b000 => InputMultiplexer::AIN0AIN1,
            0b001 => InputMultiplexer::AIN0AIN3,
            0b010 => InputMultiplexer::AIN1AIN3,
            0b011 => InputMultiplexer::AIN2AIN3,
            0b100 => InputMultiplexer::AIN0GND,
            0b101 => InputMultiplexer::AIN1GND,
            0b110 => InputMultiplexer::AIN2GND,
            _     => InputMultiplexer::AIN3GND,
        }
    }
}

impl Default for InputMultiplexer{
    fn default() -> Self {
        InputMultiplexer::AIN0AIN1
    }
}

#[derive(Debug)]
pub enum ProgramableGainAmplifier{
    V6_144,
    V4_096,
    V2_048,
    V1_024,
    V0_512,
    V0_256,
}

impl ProgramableGainAmplifier{
    fn bits(&self) -> u16{
        match self {
            ProgramableGainAmplifier::V6_144 => 0b000 << 9,
            ProgramableGainAmplifier::V4_096 => 0b001 << 9,
            ProgramableGainAmplifier::V2_048 => 0b010 << 9,
            ProgramableGainAmplifier::V1_024 => 0b011 << 9,
            ProgramableGainAmplifier::V0_512 => 0b100 << 9,
            ProgramableGainAmplifier::V0_256 => 0b101 << 9,
        }
    }

    fn from_bits(bits: u16) -> Self{
        match bits >> 9 & 0b111 {
            0b000 => ProgramableGainAmplifier::V6_144,
            0b001 => ProgramableGainAmplifier::V4_096,
            0b010 => ProgramableGainAmplifier::V2_048,
            0b011 => ProgramableGainAmplifier::V1_024,
            0b100 => ProgramableGainAmplifier::V0_512,
            _     => ProgramableGainAmplifier::V0_256,
        }
    }
}

impl Default for ProgramableGainAmplifier{
    fn default() -> Self {
        ProgramableGainAmplifier::V2_048
    }
}

#[derive(Debug, PartialEq)]
pub enum Mode{
    Continuous,
    Signle,
}

impl Mode {
    fn bits(&self) -> u16{
        match self {
            Mode::Continuous => 0,
            Mode::Signle     => 1<<8,
        }
    }

    fn from_bits(bits: u16) -> Self{
        match bits >> 8 & 0b01 {
            1 => Mode::Signle,
            _ => Mode::Continuous,
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Signle
    }
}

#[derive(Debug)]
pub enum DataRate{
    SPS8,
    SPS16,
    SPS32,
    SPS64,
    SPS128,
    SPS250,
    SPS475,
    SPS860,
}

impl DataRate{
    fn bits(&self) -> u16{
        match self {
            DataRate::SPS8   => 0b000,
            DataRate::SPS16  => 0b001 << 5,
            DataRate::SPS32  => 0b010 << 5,
            DataRate::SPS64  => 0b011 << 5,
            DataRate::SPS128 => 0b100 << 5,
            DataRate::SPS250 => 0b101 << 5,
            DataRate::SPS475 => 0b110 << 5,
            DataRate::SPS860 => 0b111 << 5,
        }
    }

    fn from_bits(bits: u16) -> Self{
        match bits >> 5 & 0b111 {
            0b000 => DataRate::SPS8,
            0b001 => DataRate::SPS16,
            0b010 => DataRate::SPS32,
            0b011 => DataRate::SPS64,
            0b100 => DataRate::SPS128,
            0b101 => DataRate::SPS250,
            0b110 => DataRate::SPS475,
            _     => DataRate::SPS860,
        }
    }
}

impl Default for DataRate{
    fn default() -> Self {
        DataRate::SPS128
    }
}

#[derive(Debug)]
pub enum ComparatorMode{
    Traditional,
    Window,
}

impl ComparatorMode{
    fn bits(&self) -> u16{
        match self {
            ComparatorMode::Traditional => 0,
            ComparatorMode::Window      => 1<<4,
        }
    }

    fn from_bits(bits: u16) -> Self{
        match bits >> 4 & 0b01 {
            1 => ComparatorMode::Window,
            _ => ComparatorMode::Traditional,
        }
    }
}

impl Default for ComparatorMode{
    fn default() -> Self {
        ComparatorMode::Traditional
    }
}

#[derive(Debug)]
pub enum ComparatorPolarity{
    ActiveLow,
    ActiveHigh,
}

impl ComparatorPolarity{
    fn bits(&self) -> u16{
        match self {
            ComparatorPolarity::ActiveLow  => 0,
            ComparatorPolarity::ActiveHigh => 1<<3,
        }
    }

    fn from_bits(bits: u16) -> Self{
        match bits >> 3 & 0b01 {
            1 => ComparatorPolarity::ActiveHigh,
            _ => ComparatorPolarity::ActiveLow,
        }
    }
}

impl Default for ComparatorPolarity{
    fn default() -> Self {
        ComparatorPolarity::ActiveLow
    }
}

#[derive(Debug)]
pub enum LatchingComparator{
    NonLatching,
    Latching,
}

impl LatchingComparator{
    fn bits(&self) -> u16{
        match self {
            LatchingComparator::NonLatching  => 0,
            LatchingComparator::Latching => 1<<2,
        }
    }

    fn from_bits(bits: u16) -> Self{
        match bits >> 2 & 0b01 {
            1 => LatchingComparator::Latching,
            _ => LatchingComparator::NonLatching,
        }
    }
}

impl Default for LatchingComparator{
    fn default() -> Self {
        LatchingComparator::NonLatching
    }
}

#[derive(Debug)]
pub enum ComparatorQueue{
    AsserAfterOne,
    AsserAfterTwo,
    AsserAfterFour,
    Disable,
}

impl ComparatorQueue{
    fn bits(&self) -> u16{
        match self {
            ComparatorQueue::AsserAfterOne  => 0b00,
            ComparatorQueue::AsserAfterTwo  => 0b01,
            ComparatorQueue::AsserAfterFour => 0b10,
            ComparatorQueue::Disable        => 0b11,
        }
    }

    fn from_bits(bits: u16) -> Self{
        match bits & 0b11 {
            0b00 => ComparatorQueue::AsserAfterOne,
            0b01 => ComparatorQueue::AsserAfterTwo,
            0b10 => ComparatorQueue::AsserAfterFour,
            _    => ComparatorQueue::Disable,
        }
    }
}

impl Default for ComparatorQueue{
    fn default() -> Self {
        ComparatorQueue::Disable
    }
}

#[derive(Default, Debug)]
pub struct ADS111xConfig{
    osw: OSW,
    osr: OSR,
    mux: InputMultiplexer,
    pga: ProgramableGainAmplifier,
    mode: Mode,
    dr: DataRate,
    comp_mode: ComparatorMode,
    comp_pol: ComparatorPolarity,
    comp_lat: LatchingComparator,
    comp_que: ComparatorQueue,
}

impl ADS111xConfig{
    fn bits(&self) -> u16{
        self.osw.bits() |
            self.mux.bits() |
            self.pga.bits() |
            self.mode.bits() |
            self.dr.bits() |
            self.comp_mode.bits() |
            self.comp_pol.bits() |
            self.comp_lat.bits() |
            self.comp_que.bits()
    }

    fn from_bits(bits: u16) -> Self{
        ADS111xConfig {
            osr: OSR::from_bits(bits),
            osw: OSW::default(),
            mux: InputMultiplexer::from_bits(bits),
            pga: ProgramableGainAmplifier::from_bits(bits),
            mode: Mode::from_bits(bits),
            dr: DataRate::from_bits(bits),
            comp_mode: ComparatorMode::from_bits(bits),
            comp_pol: ComparatorPolarity::from_bits(bits),
            comp_lat: LatchingComparator::from_bits(bits),
            comp_que: ComparatorQueue::from_bits(bits)}
    }

    pub fn mux(mut self, mux: InputMultiplexer) -> Self{
        self.mux = mux;
        self
    }

    pub fn pga(mut self, pga: ProgramableGainAmplifier) -> Self{
        self.pga = pga;
        self
    }

    pub fn mode(mut self, mode: Mode) -> Self{
        self.mode = mode;
        self
    }

    pub fn dr(mut self, dr: DataRate) -> Self{
        self.dr = dr;
        self
    }

    pub fn comp_mode(mut self, cm: ComparatorMode) -> Self{
        self.comp_mode = cm;
        self
    }

    pub fn comp_pol(mut self, cp: ComparatorPolarity) -> Self{
        self.comp_pol = cp;
        self
    }

    pub fn comp_lat(mut self, cl: LatchingComparator) -> Self{
        self.comp_lat = cl;
        self
    }

    pub fn comp_que(mut self, cq: ComparatorQueue) -> Self{
        self.comp_que = cq;
        self
    }
}

pub(crate) struct ADS111x {
    address: u8,
    config: ADS111xConfig,
}

impl ADS111x
{
    pub fn new(address: u8, config: ADS111xConfig) -> Result<Self, ADSError>{
        match address {
            0b1001000 => {},
            0b1001001 => {},
            0b1001010 => {},
            0b1001011 => {},
            _ => return Err(ADSError::WrongAddress),
        }
        Ok(ADS111x{ address, config} )
    }

    ///Writes self configuration to device
    ///Config can be used to update configuration
    pub async fn write_config<I2C, E> (&mut self, i2c: &mut I2C, config: Option<ADS111xConfig>) -> Result<(), E> where I2C: I2c<Error = E> {
        if let Some(conf) = config{
            self.config = conf;
        }
        self.config.osw = OSW::Idle;
        let conf = self.config.bits().to_be_bytes();
        i2c.write(self.address, &[CONFIG_REGISTER, conf[0], conf[1]]).await
    }

    pub async fn read_config<I2C, E>(&mut self, i2c: &mut I2C) -> Result<ADS111xConfig, E> where I2C: I2c<Error = E> {
        self.config.osw = OSW::Idle;
        let mut conf = [0, 0];

        i2c.write_read(self.address, &[CONFIG_REGISTER], &mut conf).await.and(Ok(ADS111xConfig::from_bits(u16::from_be_bytes(conf))))
    }

    /// Perform single read when mode set to single
    /// ADC is in low power state until requested and will go back after conversion
    /// Will block until converstion is ready
    /// Mux can be used to reconfigure what ADC input to read
    pub async fn read_single_voltage<I2C, E>(&mut self, i2c: &mut I2C, mux: Option<InputMultiplexer>) -> Result<f32, E> where I2C: I2c<Error = E> {
        if let Some(m) = mux{
            self.config.mux = m;
        }
        self.config.osw = OSW::StartConversion;
        let config = self.config.bits().to_be_bytes();
        let mut conf = [0, 0];

        i2c.write_read(self.address, &[CONFIG_REGISTER, config[0], config[1]], &mut conf).await?;

        while OSR::from_bits(u16::from_be_bytes(conf)) == OSR::PerformingConversion{
            i2c.write_read(self.address, &[CONFIG_REGISTER], &mut conf).await?;
        }

        self.read_voltage(i2c).await
    }

    pub async fn check_cnversion_ready<I2C, E>(&mut self, i2c: &mut I2C) -> Result<bool, E> where I2C: I2c<Error = E> {
        Ok(self.read_config(i2c).await?.osr == OSR::DeviceIdle)
    }

    /// Reads conversion
    /// does not block to wait for conversion
    /// will return 0 when conversion was still ongoing
    /// You can use check_coversion_ready if needed
    /// only works when Mode is Continuous
    pub async fn read_voltage<I2C, E>(&mut self, i2c: &mut I2C) -> Result<f32, E> where I2C: I2c<Error = E> {
        let mut voltage = [0, 0];
        i2c.write_read(self.address, &[CONVERSION_REGISTER], &mut voltage).await?;
        let val = i16::from_be_bytes(voltage);
        let pga = match self.config.pga{
            ProgramableGainAmplifier::V0_256 => 0.256f32,
            ProgramableGainAmplifier::V0_512 => 0.512f32,
            ProgramableGainAmplifier::V1_024 => 1.024f32,
            ProgramableGainAmplifier::V2_048 => 2.048f32,
            ProgramableGainAmplifier::V4_096 => 4.096f32,
            ProgramableGainAmplifier::V6_144 => 6.144f32,
        };

        Ok(f32::from(val) * pga / 32768f32)
    }

    pub async fn set_low_treshold<I2C, E>(&mut self, i2c: &mut I2C, low_tresh: i16) -> Result<(), E> where I2C: I2c<Error = E> {
        let lt = low_tresh.to_be_bytes();
        i2c.write(self.address, &[LO_THRESH_REGISTER, lt[0], lt[1]]).await
    }

    pub async fn set_high_treshold<I2C, E>(&mut self, i2c: &mut I2C, high_tresh: i16) -> Result<(), E> where I2C: I2c<Error = E> {
        let ht = high_tresh.to_be_bytes();
        i2c.write(self.address, &[HI_THRESH_REGISTER, ht[0], ht[1]]).await
    }
}