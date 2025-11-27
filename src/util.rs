use bevy::{
    camera::{Projection, OrthographicProjection},
    ecs::error::BevyError
};

pub trait AsOrthographicProjection {
    fn as_ortho(&self) -> Result<&OrthographicProjection, BevyError>;

    fn as_ortho_mut(&mut self) -> Result<&mut OrthographicProjection, BevyError>;
}

impl AsOrthographicProjection for Projection {
    fn as_ortho(&self) -> Result<&OrthographicProjection, BevyError> {
        match *self {
            Projection::Orthographic(ref p) => Ok(p),
            _ => Err("Projection is not orthographic!".into())
        }
    }

    fn as_ortho_mut(&mut self) -> Result<&mut OrthographicProjection, BevyError> {
        match *self {
            Projection::Orthographic(ref mut p) => Ok(p),
            _ => Err("Projection is not orthographic!".into())
        }
    }
}
