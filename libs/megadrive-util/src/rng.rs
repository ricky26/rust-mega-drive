use core::ptr::read_volatile;

const GFX_HVCOUNTER_PORT: *const u16 = 0xC00008 as _;

pub struct PseudoRng {
    current_rand:  u16,
}

impl PseudoRng {
    // Thank you Stephane Dallongeville!
    pub fn from_seed(seed: u16) -> PseudoRng {
        PseudoRng {
            current_rand: seed ^ 0xD94B // XOR with some val to avoid 0
        }
    }

    pub fn random(&mut self) -> u16 {
        // SAFETY
        // The read_volatile call is guaranteed ONLY on the Sega Mega Drive. The horizontal/vertical
        // video sync counter is a "port" that is mapped directly into the system address space. It
        // is to my knowledge always initialized, so GFX_HVCOUNTER_PORT can never be a null
        // reference.
        unsafe {
            // https://github.com/Stephane-D/SGDK/blob/908926201af8b48227be4dbc8fbb0d5a18ac971b/src/tools.c#L36
            let hv_counter = read_volatile(GFX_HVCOUNTER_PORT) as u16;
            self.current_rand ^= (self.current_rand >> 1) ^ hv_counter;
            self.current_rand ^= self.current_rand << 1;
            self.current_rand
        }
    }
}
