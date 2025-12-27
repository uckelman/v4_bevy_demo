use serde::Deserialize;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Angle(pub f32);

impl Angle {
    pub fn new(angle: f32) -> Option<Self> {
        if angle == 0.0 || angle <= -360.0 || angle >= 360.0 || angle.is_nan() {
            None
        }
        else {
            Some(Self(angle))
        }
    }
}

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Angle {}

impl Hash for Angle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&f32::to_ne_bytes(self.0));
    }
}
