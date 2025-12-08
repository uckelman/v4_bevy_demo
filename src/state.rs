use bevy::prelude::States;
use std::hash::Hash;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Splash,
    Game
}
