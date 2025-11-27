use bevy::prelude::States;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
pub enum GameState {
    #[default]
    Splash,
    Game
}
