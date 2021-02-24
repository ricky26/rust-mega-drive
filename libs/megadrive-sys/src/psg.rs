use core::ptr::write_volatile;

const PSG_BASE: *mut u8 = 0xc00011 as _;
const NUM_CHANNELS: u8 = 4;

/// A frequency for use with the noise generator.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum NoiseFrequency {
    High = 0,
    Mid = 1,
    Low = 2,
    Channel2 = 3,
}

/// A selection of note frequencies to use with the tone generators.
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum Note {
    C3 = 851,
    CSharp3 = 803,
    D3 = 758,
    DSharp3 = 715,
    E3 = 675,
    F3 = 637,
    FSharp3 = 601,
    G3 = 568,
    GSharp3 = 536,
    A3 = 506,
    ASharp3 = 477,
    B3 = 450,
}

impl Note {
    /// Return a frequency value for the given note increased by the given
    /// number of octaves.
    pub fn increase_octave(self, o: usize) -> u16 {
        (self as u16) >> o
    }
}

impl From<Note> for u16 {
    fn from(n: Note) -> Self {
        n as u16
    }
}

/// The programmable sound generator.
///
/// This chip can generate noise and some square waves. Mostly this is used for
/// sound effects.
pub struct PSG;

impl PSG {
    /// Initialise and return the PSG.
    ///
    /// This is not marked unsafe as it cannot lead to memory safety issues, however
    /// creating two of these can cause conflicts in the generated sounds.
    pub fn new() -> PSG {
        let psg = PSG;

        for c in 0..NUM_CHANNELS {
            psg.set_volume(c, 0);
        }

        psg
    }

    fn write(&self, v: u8) {
        unsafe { write_volatile(PSG_BASE, v) };
    }

    /// Set the volume of a channel.
    pub fn set_volume(&self, channel: u8, volume: u8) {
        self.write(0x90 | ((channel & 3) << 5) | (0x1f - (volume & 0x1f)));
    }

    /// Set the pitch of one of the channels.
    ///
    /// This is not valid for the noise generator on channel 3.
    pub fn set_pitch(&self, channel: u8, frequency: impl Into<u16>) {
        let frequency = frequency.into();
        self.write(0x80 | ((channel & 3) << 5) | ((frequency as u8) & 0xf));
        self.write((frequency >> 4) as u8);
    }

    /// Configure the noise channel.
    pub fn set_noise(&self, white: bool, frequency: NoiseFrequency) {
        let white = if white { 4 } else { 0 };
        self.write(0xe0 | white | (frequency as u8));
    }
}
