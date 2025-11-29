use bevy::input::keyboard::KeyCode;

// TODO: make KeyConfig derivable?

pub trait KeyConfig {
    fn code(&self) -> KeyCode;
}
