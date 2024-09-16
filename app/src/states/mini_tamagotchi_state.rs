use sails_rs::prelude::*;

pub struct MiniTamagotchi {
    pub name: String,
    pub is_playing: bool,
    pub is_hungry: bool,
}

impl MiniTamagotchi {
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_playing: false,
            is_hungry: true,
        }
    }
}