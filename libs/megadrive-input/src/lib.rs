#![no_std]

use megadrive_sys::ports;

/// ControllerState represents the last-read state of the controller.
#[derive(Clone, Debug)]
pub struct ControllerState {
    buttons: u16,
    last_buttons: u16,
    is_6button: bool,
}

impl ControllerState {
    /// Returns true if this is a 6-button controller.
    pub fn is_6button(&self) -> bool {
        self.is_6button
    }

    /// Return the mask of buttons which are currently down.
    pub fn get_down_raw(&self) -> u16 { self.buttons }

    /// Return the bitmask of buttons which have been pressed down since last
    /// frame.
    pub fn get_pressed_raw(&self) -> u16 {
        self.buttons &! self.last_buttons
    }
}

/// A high level controller manager which can interact with controllers connected to the
/// IO ports.
///
/// The controller only uses the two controller ports and not the EXT port.
pub struct Controllers {
    controllers: [Option<ControllerState>; 2],
}

impl Controllers {
    /// Create a controller manager and initialise it.
    ///
    /// Whilst this is not unsafe, as it would not cause any memory risk,
    /// creating two of these managers will create interference.
    pub fn new() -> Controllers {
        // Configure the controllers for input except for the 'clock' pin.
        let c1 = ports::controller_1();
        c1.set_pin_directions_raw(0x40);
        c1.set_pins(0x40);

        let c2 = ports::controller_2();
        c2.set_pin_directions_raw(0x40);
        c2.set_pins(0x40);

        let controllers = unsafe {
            let mut c: [Option<ControllerState>; 2] = core::mem::MaybeUninit::uninit().assume_init();
            c[0] = None;
            c[1] = None;
            c
        };

        Controllers {
            controllers,
        }
    }

    /// Retrieve the controller states of all controllers.
    pub fn controller_states(&self) -> &[Option<ControllerState>] {
        &self.controllers
    }

    /// Fetch the controller state for a single controller.
    pub fn controller_state(&self, index: usize) -> Option<&ControllerState> {
        self.controllers[index].as_ref()
    }

    fn read_pins_half(v: u8) -> (u8, u8) {
        let c1 = ports::controller_1();
        let c2 = ports::controller_2();

        c1.set_pins(v);
        c2.set_pins(v);

        // HACK: these could be NOPs but asm!() is not stable.
        c1.set_pins(v);
        c1.set_pins(v);
        c1.set_pins(v);

        let c1pins = c1.get_pins();
        let c2pins = c2.get_pins();

        (c1pins, c2pins)
    }

    fn read_pins() -> (u16, u16) {
        let (c1lo, c2lo) = Controllers::read_pins_half(0x40);
        let (c1hi, c2hi) = Controllers::read_pins_half(0x00);

        let c1pins = ((c1hi as u16) << 8) | (c1lo as u16);
        let c2pins = ((c2hi as u16) << 8) | (c2lo as u16);

        (c1pins, c2pins)
    }

    fn update_state(state: &mut Option<ControllerState>, connected: bool, is_6button: bool, buttons: u16) {
        if state.is_some() && connected {
            // Just update.
            let ptr = state.as_mut().unwrap();
            ptr.last_buttons = ptr.buttons;
            ptr.buttons = buttons;
            ptr.is_6button = is_6button;
            return;
        }

        *state = if connected {
            Some(ControllerState{
                buttons,
                last_buttons: 0,
                is_6button,
            })
        } else {
            None
        };
    }

    /// Update the state of the controllers.
    ///
    /// This should only be called once per VBlank. Calling it too frequently
    /// can result in incorrect results.
    pub fn update(&mut self) {
        // We have to read the controllers 3 times in order to read extended
        // buttons.

        let (c1_pins, c2_pins) = Controllers::read_pins();
        let mut c1_buttons = (!c1_pins & 0x3f) | ((!c1_pins >> 6) & 0xc0);
        let mut c2_buttons = (!c2_pins & 0x3f) | ((!c2_pins >> 6) & 0xc0);

        let c1_connected = (c1_pins & 0xc00) == 0;
        let c2_connected = (c2_pins & 0xc00) == 0;

        Controllers::read_pins();
        let (c1_test, c2_test) = Controllers::read_pins();

        let c1_is6 = (c1_test & 0xf) == 0;
        let c2_is6 = (c2_test & 0xf) == 0;
        let (c1_ext, c2_ext) = Controllers::read_pins_half(0x40);

        if c1_is6 {
            c1_buttons |= ((c1_ext as u16) & 0xf) << 8;
        }

        if c2_is6 {
            c2_buttons |= ((c2_ext as u16) & 0xf) << 8;
        }

        Controllers::update_state(&mut self.controllers[0], c1_connected, c1_is6, c1_buttons);
        Controllers::update_state(&mut self.controllers[1], c2_connected, c2_is6, c2_buttons);
    }
}
