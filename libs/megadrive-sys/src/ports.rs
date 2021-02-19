use core::ptr::{write_volatile, read_volatile};

/// NOTE: All three of the ports on the Mega Drive can support serial operation
/// as well as GPIO operation. However, that is not commonly used and is not
/// implemented in this library (yet).

/// A representation of one of the 3 IO ports on the Mega Drive.
pub struct IOPort(*mut u8);

impl IOPort {
    /// Set the directions of the pins on this IO port.
    ///
    /// A one indicates the pin is used as output.
    pub fn set_pin_directions_raw(&self, directions: u8) {
        unsafe {
            write_volatile(self.0.offset(6), directions & 0x3f);
        }
    }

    /// Set the values of output pins.
    ///
    /// Any other specified pin values are ignored.
    pub fn set_pins(&self, values: u8) {
        unsafe {
            write_volatile(self.0, values)
        }
    }

    /// Get the value of all of the pins.
    ///
    /// Output pins show the value they were last set to.
    pub fn get_pins(&self) -> u8 {
        unsafe {
            read_volatile(self.0)
        }
    }
}

/// The first player's controller port.
pub fn controller_1() -> IOPort { IOPort(0xa10003 as _) }

/// The second player's controller port.
pub fn controller_2() -> IOPort { IOPort(0xa10005 as _) }

/// The optional extension port on the back of the console.
pub fn ext() -> IOPort { IOPort(0xa10007 as _) }
