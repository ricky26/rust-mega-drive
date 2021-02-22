use core::ptr::{write_volatile, read_volatile};

const FM_BASE: *mut u8 = 0xa04000 as _;
const FM_LFO: u8 = 0x22;
const FM_TIMER_CTRL: u8 = 0x27;
const FM_KEY_ON: u8 = 0x28;
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

impl Into<u16> for Note {
    fn into(self) -> u16 {
        self as u16
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
        let mut fm = FM;
        fm.enable_lfo(None);

        // Disable special mode channel 3.
        fm.write_reg(FM_TIMER_CTRL, 0);
        // Disable DAC.
        fm.write_reg(FM_DAC_ENABLE, 0);

        for ch in 0..NUM_CHANNELS {
            fm.set_panning(ch, Panning::None, 0, 0);
            fm.set_key(ch, false);

            for op in 0..NUM_OPERATORS {
                fm.set_ssgeg(ch, op, 0);
            }
        }

        fm
    }

    fn write_reg_bank(&mut self, second: bool, addr: u8, value: u8) {
        let reg_offset = if second { 2 } else { 0 };
        unsafe {
            let base = FM_BASE.offset(reg_offset);

            // Busy spin until idle.
            while (read_volatile(base) & 0x80) != 0 {}

            write_volatile(base, addr);
            write_volatile(base.offset(1), value);
        }
    }

    fn write_reg(&mut self, addr: u8, value: u8) {
        self.write_reg_bank(false, addr, value);
    }

    fn write_op_reg(&mut self, base: u8, channel: u8, op: u8, value: u8) {
        let (channel, second) = if channel > 3 {
            (channel - 3, true)
        } else {
            (channel, false)
        };
        let addr = base | (channel & 3) | ((op & 3) << 2);
        self.write_reg_bank(second, addr, value);
    }

    fn write_ch_reg(&mut self, base: u8, channel: u8, value: u8) {
        let (channel, second) = if channel > 3 {
            (channel - 3, true)
        } else {
            (channel, false)
        };
        let addr = base + (channel & 3);
        self.write_reg_bank(second, addr, value);
    }

    /// Enable or disable the LFO unit.
    ///
    /// This enables and sets the frequency of the low-frequency-oscillator.
    /// This is not enough to employ it - the LFO needs to be abled per
    /// operator.
    pub fn enable_lfo(&mut self, rate: Option<u8>) {
        let v = rate.map_or(0, |v| 8 | (v & 7));
        self.write_reg(FM_LFO, v);
    }

    /// Set whether the key is down for a channel.
    ///
    /// This version allows setting key down individually per-operator, however
    /// generally `set_key` alone is sufficient.
    pub fn set_key_raw(&mut self, channel: u8, operator_mask: u8) {
        let v = (ALL_CHANNELS[channel as usize] & 7) | (operator_mask << 4);
        self.write_reg(FM_KEY_ON, v);
    }

    /// Set whether the key for a particular channel is down.
    pub fn set_key(&mut self, channel: u8, down: bool) {
        let mask = if down { 0xf } else { 0 };
        self.set_key_raw(channel, mask);
    }

    /// Set the operator multiplier & detuning.
    ///
    /// The multiply value is as written, except that 0 means 0.5.
    pub fn set_multiplier(&mut self, channel: u8, operator: u8, multiply: u8, detune: u8) {
        let v = ((detune & 7) << 4) | (multiply & 0xf);
        self.write_op_reg(FM_MULTIPLY, channel, operator, v);
    }

    /// Set the total level (volume) of a single operator.
    ///
    /// The maximum value is 127.
    pub fn set_total_level(&mut self, channel: u8, operator: u8, level: u8) {
        self.write_op_reg(FM_TOTAL_LEVEL, channel, operator, level & 0x7f);
    }

    /// Set the attack rate and attack rate scaling for an operator.
    ///
    /// The attack rate defines how quickly the operator reaches maximum amplitude
    /// (with 63 being the maximum).
    ///
    /// The attack rate scale is used to increase the attack rate at higher frequencies.
    /// 0 indicates no increase, increasing the value increases the attack rate.
    pub fn set_attack_rate(&mut self, channel: u8, operator: u8, attack_rate: u8, rate_scale: u8) {
        let v = (attack_rate & 0x1f) | (rate_scale << 6);
        self.write_op_reg(FM_ATTACK_RATE, channel, operator, v);
    }

    /// Set the decay rate and enable amplitude modulation.
    ///
    /// The higher the decay rate, the steeper the decay, with 31 being the maximum.
    ///
    /// If amon is true, the global amplitude modulation will be applied
    pub fn set_decay_rate(&mut self, channel: u8, operator: u8, decay_rate: u8, amon: bool) {
        let v = ((amon as u8) << 7) | (decay_rate & 0x1f);
        self.write_op_reg(FM_DECAY_RATE, channel, operator, v);
    }

    /// Set the sustain rate of the operator (also known as the second decay rate).
    ///
    /// The maximum value is 31. Higher values mean steeper decay.
    pub fn set_sustain_rate(&mut self, channel: u8, operator: u8, sustain_rate: u8) {
        let v = sustain_rate & 0x1f;
        self.write_op_reg(FM_SUSTAIN_RATE, channel, operator, v);
    }

    /// Set the release rate and sustain level of an operator.
    pub fn set_release_rate(&mut self, channel: u8, operator: u8, release_rate: u8, sustain_level: u8) {
        let v = (release_rate & 0xf) | ((sustain_level & 0xf) << 4);
        self.write_op_reg(FM_RELEASE_RATE, channel, operator, v);
    }

    /// Set the proprietary field.
    ///
    /// According to the documentation, this should always be set to zero.
    fn set_ssgeg(&mut self, channel: u8, operator: u8, value: u8) {
        self.write_op_reg(FM_SSGEG, channel, operator, value);
    }

    /// Set the frequency for a bank.
    ///
    /// The frequency is specified in chip-specific units. Use the `Note` enum for
    /// pre-calculated values.
    ///
    /// Octave can be used to increase the octave (the maximum value is 7).
    pub fn set_frequency(&mut self, channel: u8, frequency: impl Into<u16>, octave: u8) {
        let frequency = frequency.into();
        let lo = frequency as u8;
        let hi = (((frequency >> 8) as u8) & 7) | ((octave & 7) << 3);

        self.write_ch_reg(FM_FREQUENCY_HI, channel, hi);
        self.write_ch_reg(FM_FREQUENCY_LO, channel, lo);
    }

    /// Set the algorithm and feedback for a bank.
    pub fn set_algorithm(&mut self, channel: u8, algorithm: u8, feedback: u8) {
        let v = (algorithm & 7) | ((feedback & 7) << 3);
        self.write_ch_reg(FM_ALGORITHM, channel, v);
    }

    /// Set the panning, frequency modulation and amplituate modulation for a bank.
    pub fn set_panning(&mut self, channel: u8, panning: Panning, ams: u8, fms: u8) {
        let v = ((panning as u8) << 6) | ((ams & 3) << 4) | (fms & 7);
        self.write_ch_reg(FM_PANNING, channel, v);
    }
}
