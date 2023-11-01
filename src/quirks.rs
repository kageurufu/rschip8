use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Quirks {
    pub vf_reset: bool,
    pub memory: bool,
    pub display_wait: bool,
    pub sprite_wrapping: bool,
    pub hires_draw_flag: bool,
    pub shifting: bool,
    pub jumping: bool,
}

impl Quirks {
    pub fn new(
        vf_reset: bool,
        memory: bool,
        display_wait: bool,
        clipping: bool,
        hires_draw_flag: bool,
        shifting: bool,
        jumping: bool,
    ) -> Quirks {
        Quirks {
            vf_reset,
            memory,
            display_wait,
            sprite_wrapping: clipping,
            hires_draw_flag,
            shifting,
            jumping,
        }
    }

    pub fn chip8() -> Quirks {
        Quirks {
            vf_reset: true,
            memory: true,
            display_wait: true,
            sprite_wrapping: false,
            hires_draw_flag: false,
            shifting: false,
            jumping: false,
        }
    }

    pub fn superchip() -> Quirks {
        Quirks {
            vf_reset: false,
            memory: false,
            display_wait: false,
            sprite_wrapping: false,
            hires_draw_flag: true,
            shifting: true,
            jumping: true,
        }
    }

    pub fn xochip() -> Quirks {
        Quirks {
            vf_reset: false,
            memory: true,
            display_wait: false,
            sprite_wrapping: true,
            hires_draw_flag: false,
            shifting: false,
            jumping: false,
        }
    }
}

impl Default for Quirks {
    fn default() -> Self {
        Quirks::chip8()
    }
}
