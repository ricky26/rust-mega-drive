use core::ptr::{write_volatile, read_volatile};

const FM_BASE: *mut u8 = 0xa04000 as _;
const FM_LFO: u8 = 0x22;
const FM_TIMER_A_HI: u8 = 0x24;
const FM_TIMER_A_LO: u8 = 0x25;
const FM_TIMER_B: u8 = 0x25;
const FM_TIMER_CTRL: u8 = 0x27;
const FM_KEY_ON: u8 = 0x28;
const FM_DAC_DATA: u8 = 0x2a;
const FM_DAC_ENABLE: u8 = 0x2b;
const FM_MULTIPLY: u8 = 0x30;
const FM_TOTAL_LEVEL: u8 = 0x40;
const FM_ATTACK_RATE: u8 = 0x50;
const FM_DECAY_RATE: u8 = 0x60;
const FM_SUSTAIN_RATE: u8 = 0x70;
const FM_RELEASE_RATE: u8 = 0x80;
const FM_SSGEG: u8 = 0x90;
const FM_FREQUENCY_LO: u8 = 0xa0;
const FM_FREQUENCY_HI: u8 = 0xa4;
const FM_ALGORITHM: u8 = 0xb0;
const FM_PANNING: u8 = 0xb4;

static FM_SPECIAL_FREQUENCY_LO: [u8; 4] = [0xa9, 0xaa, 0xa8, 0xa2];
static FM_SPECIAL_FREQUENCY_HI: [u8; 4] = [0xad, 0xae, 0xac, 0xa6];

static ALL_CHANNELS: [u8; 6] = [0, 1, 2, 4, 5, 6];
const NUM_CHANNELS: u8 = 6;
const NUM_OPERATORS: u8 = 4;

/// An enum representing the 4 supported panning values for a bank.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Panning {
    None = 0b00,
    Left = 0b10,
    Right = 0b01,
    Both = 0b11,
}

/// Note frequencies for use with the YM2612.
///
/// These are slightly inaccurate since the NTSC & PAL versions of the console
/// operate the FM chip at slightly different frequencies.
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum Note {
    C = 644,
    CSharp = 681,
    D = 722,
    DSharp = 765,
    E = 810,
    F = 858,
    FSharp = 910,
    G = 964,
    GSharp = 1021,
    A = 1081,
    ASharp = 1146,
    B = 1214,
}

impl From<Note> for u16 {
    fn from(v: Note) -> Self {
        v as u16
    }
}

/// Frequency enumeration for the Low-Frequency Oscillator.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum LFORate {
    F3_82Hz = 0b000,
    F5_33Hz = 0b001,
    F5_77Hz = 0b010,
    F6_11Hz = 0b011,
    F6_60Hz = 0b100,
    F9_23Hz = 0b101,
    F46_11Hz = 0b110,
    F69_22Hz = 0b111,
}

impl From<LFORate> for u8 {
    fn from(v: LFORate) -> Self {
        v as u8
    }
}

/// A struct for formatting the timer config used by the FM chip.
#[derive(Clone, Copy, Debug)]
pub struct TimerConfig(u8);

impl TimerConfig {
    /// Create a new timer config with CH3 in normal mode and both timers disabled.
    pub fn new() -> TimerConfig {
        TimerConfig(0)
    }

    /// Configure channel 3 'special mode'.
    pub fn ch3_special_mode(self, v: bool) -> TimerConfig {
        let f = if v { 0x40 } else { 0 };
        TimerConfig((self.0 & 0x3f) | f)
    }

    /// Enable or disable timer A.
    ///
    /// If `e` is set, the timer is enabled and will count.
    /// If `r` is set, the timer will set the completed flag when it wraps.
    pub fn enable_timer_a(self, e: bool, r: bool) -> TimerConfig {
        let fe = if e { 4 } else { 0 };
        let fr = if r { 1 } else { 0 };
        TimerConfig((self.0 & 0xfa) | fe | fr)
    }

    /// Enable or disable timer B.
    ///
    /// If `e` is set, the timer is enabled and will count.
    /// If `r` is set, the timer will set the completed flag when it wraps.
    pub fn enable_timer_b(self, e: bool, r: bool) -> TimerConfig {
        let fe = if e { 8 } else { 0 };
        let fr = if r { 2 } else { 0 };
        TimerConfig((self.0 & 0xf5) | fe | fr)
    }

    /// Reset timer A.
    pub fn reset_timer_a(self, v: bool) -> TimerConfig {
        let v = if v {
            self.0 | 0x10
        } else {
            self.0 &! 0x10
        };
        TimerConfig(v)
    }

    /// Reset timer B.
    pub fn reset_timer_b(self, v: bool) -> TimerConfig {
        let v = if v {
            self.0 | 0x20
        } else {
            self.0 &! 0x20
        };
        TimerConfig(v)
    }
}

/// A driver for the YM2612.
///
/// Whilst normally controlled by the Z80, this driver can be used to operate
/// the YM2612 from the 68k.
pub struct FM;

impl FM {
    /// Initialise the FM hardware and return an object for manipulating it.
    ///
    /// Whilst this is memory safe (and thus the function is not unsafe),
    /// initialising multiple FMs will result in conflicts.
    pub fn new() -> FM {
        let fm = FM;
        fm.enable_lfo(None);

        // Disable special mode channel 3.
        fm.configure_timers(TimerConfig::new());
        fm.enable_dac(false);

        for ch in fm.channels() {
            ch.set_panning(Panning::None, 0, 0);
            ch.set_key(false);

            for op in ch.operators() {
                op.set_ssgeg(0);
            }
        }

        fm
    }

    fn write_reg_bank(&self, second: bool, addr: u8, value: u8) {
        let reg_offset = if second { 2 } else { 0 };
        unsafe {
            let base = FM_BASE.offset(reg_offset);

            // Busy spin until idle.
            while (read_volatile(FM_BASE) & 0x80) != 0 {}

            write_volatile(base, addr);
            write_volatile(base.offset(1), value);
        }
    }

    fn write_reg(&self, addr: u8, value: u8) {
        self.write_reg_bank(false, addr, value);
    }

    /// Enable or disable the LFO unit.
    ///
    /// This enables and sets the frequency of the low-frequency-oscillator.
    /// This is not enough to employ it - the LFO needs to be abled per
    /// operator.
    pub fn enable_lfo(&self, rate: Option<LFORate>) {
        let v = rate.map_or(0, |v| 8 | (u8::from(v) & 7));
        self.write_reg(FM_LFO, v);
    }

    /// Enables or disables the DAc, which replaces channel 6.
    pub fn enable_dac(&self, enabled: bool) {
        let v = if enabled { 0x80 } else { 0 };
        self.write_reg(FM_DAC_ENABLE, v);
    }

    /// Write a single sample to the DAC.
    pub fn dac_write(&self, v: u8) {
        self.write_reg(FM_DAC_DATA, v);
    }

    /// Configure the frequency of timer A.
    ///
    /// The interval is calculated as `(1024 - f) * 18 us`.
    pub fn set_timer_a(&self, f: u16) {
        let hi = (f >> 2) as u8;
        let lo = (f & 3) as u8;

        self.write_reg(FM_TIMER_A_HI, hi);
        self.write_reg(FM_TIMER_A_LO, lo);
    }

    /// Configure the frequency of timer B.
    ///
    /// The interval is calculated as `(256 - f) * 288 us`.
    pub fn set_timer_b(&self, f: u8) {
        self.write_reg(FM_TIMER_B, f);
    }

    /// Configure the timers.
    pub fn configure_timers(&self, c: TimerConfig) {
        self.write_reg(FM_TIMER_CTRL, c.0)
    }

    /// Check whether the timers have completed.
    pub fn timer_status(&self) -> (bool, bool) {
        let v = unsafe { read_volatile(FM_BASE) };
        let a = (v & 1) != 0;
        let b = (v & 2) != 0;
        (a, b)
    }

    /// Fetch a single FM channel.
    pub fn channel(&self, channel: u8) -> Channel {
        Channel(FM, channel & 7)
    }

    /// Fetch all of the channels of the FM chip.
    pub fn channels(&self) -> impl Iterator<Item=Channel> {
        (0..NUM_CHANNELS).map(|c| Channel(FM, c))
    }
}

/// A single FM channel.
///
/// This is a hardware voice. It can only be playing one note at a time.
pub struct Channel(FM, u8);

impl Channel {
    fn write_reg(&self, base: u8, value: u8) {
        let channel = self.1;
        let (channel, second) = if channel > 3 {
            (channel - 3, true)
        } else {
            (channel, false)
        };
        let addr = base + (channel & 3);
        self.0.write_reg_bank(second, addr, value);
    }

    /// Set whether the key is down for a channel.
    ///
    /// This version allows setting key down individually per-operator, however
    /// generally `set_key` alone is sufficient.
    pub fn set_key_raw(&self, operator_mask: u8) {
        let v = (ALL_CHANNELS[self.1 as usize] & 7) | (operator_mask << 4);
        self.0.write_reg(FM_KEY_ON, v);
    }

    /// Set whether the key for a particular channel is down.
    pub fn set_key(&self, down: bool) {
        let mask = if down { 0xf } else { 0 };
        self.set_key_raw(mask);
    }

    /// Set the frequency for a bank.
    ///
    /// The frequency is specified in chip-specific units. Use the `Note` enum for
    /// pre-calculated values.
    ///
    /// Octave can be used to increase the octave (the maximum value is 7).
    pub fn set_frequency(&self, frequency: impl Into<u16>, octave: u8) {
        let frequency = frequency.into();
        let lo = frequency as u8;
        let hi = (((frequency >> 8) as u8) & 7) | ((octave & 7) << 3);

        self.write_reg(FM_FREQUENCY_HI, hi);
        self.write_reg(FM_FREQUENCY_LO, lo);
    }

    /// Set the algorithm and feedback for a bank.
    pub fn set_algorithm(&self, algorithm: u8, feedback: u8) {
        let v = (algorithm & 7) | ((feedback & 7) << 3);
        self.write_reg(FM_ALGORITHM, v);
    }

    /// Set the panning, frequency modulation and amplituate modulation for a bank.
    pub fn set_panning(&self, panning: Panning, ams: u8, fms: u8) {
        let v = ((panning as u8) << 6) | ((ams & 3) << 4) | (fms & 7);
        self.write_reg(FM_PANNING, v);
    }

    /// Get one of this channel's operators.
    pub fn operator(&self, operator: u8) -> Operator {
        Operator(FM, self.1, operator & 3)
    }

    /// Get all of the operators for this channel.
    pub fn operators(&self) -> impl Iterator<Item=Operator> {
        let ch = self.1;
        (0..NUM_OPERATORS).map(move |i| Operator(FM, ch, i))
    }
}

/// A single modulator in the channel.
///
/// These are chained together to produce the output waveform.
pub struct Operator(FM, u8, u8);

impl Operator {
    fn write_reg(&self, base: u8, value: u8) {
        let channel = self.1;
        let op = self.2;
        let (channel, second) = if channel > 3 {
            (channel - 3, true)
        } else {
            (channel, false)
        };
        let addr = base | (channel & 3) | ((op & 3) << 2);
        self.0.write_reg_bank(second, addr, value);
    }

    /// Set the operator multiplier & detuning.
    ///
    /// The multiply value is as written, except that 0 means 0.5.
    pub fn set_multiplier(&self, multiply: u8, detune: u8) {
        let v = ((detune & 7) << 4) | (multiply & 0xf);
        self.write_reg(FM_MULTIPLY, v);
    }

    /// Set the total level (volume) of a single operator.
    ///
    /// The maximum value is 127.
    pub fn set_total_level(&self, level: u8) {
        self.write_reg(FM_TOTAL_LEVEL, level & 0x7f);
    }

    /// Set the attack rate and attack rate scaling for an operator.
    ///
    /// The attack rate defines how quickly the operator reaches maximum amplitude
    /// (with 63 being the maximum).
    ///
    /// The attack rate scale is used to increase the attack rate at higher frequencies.
    /// 0 indicates no increase, increasing the value increases the attack rate.
    pub fn set_attack_rate(&self, attack_rate: u8, rate_scale: u8) {
        let v = (attack_rate & 0x1f) | (rate_scale << 6);
        self.write_reg(FM_ATTACK_RATE, v);
    }

    /// Set the decay rate and enable amplitude modulation.
    ///
    /// The higher the decay rate, the steeper the decay, with 31 being the maximum.
    ///
    /// If amon is true, the global amplitude modulation will be applied
    pub fn set_decay_rate(&self, decay_rate: u8, amon: bool) {
        let v = ((amon as u8) << 7) | (decay_rate & 0x1f);
        self.write_reg(FM_DECAY_RATE, v);
    }

    /// Set the sustain rate of the operator (also known as the second decay rate).
    ///
    /// The maximum value is 31. Higher values mean steeper decay.
    pub fn set_sustain_rate(&self, sustain_rate: u8) {
        let v = sustain_rate & 0x1f;
        self.write_reg(FM_SUSTAIN_RATE, v);
    }

    /// Set the release rate and sustain level of an operator.
    pub fn set_release_rate(&self, release_rate: u8, sustain_level: u8) {
        let v = (release_rate & 0xf) | ((sustain_level & 0xf) << 4);
        self.write_reg(FM_RELEASE_RATE, v);
    }

    /// Set the frequency for a single operator.
    ///
    /// This is only valid for channel 3 in 'special' mode.
    pub fn set_frequency_special(&self, frequency: impl Into<u16>, octave: u8) {
        let frequency = frequency.into();
        let lo = frequency as u8;
        let hi = (((frequency >> 8) as u8) & 7) | ((octave & 7) << 3);
        self.0.write_reg(FM_SPECIAL_FREQUENCY_HI[self.2 as usize], hi);
        self.0.write_reg(FM_SPECIAL_FREQUENCY_LO[self.2 as usize], lo);
    }

    /// Set the proprietary field.
    ///
    /// According to the documentation, this should always be set to zero.
    fn set_ssgeg(&self, value: u8) {
        self.write_reg(FM_SSGEG, value);
    }
}
