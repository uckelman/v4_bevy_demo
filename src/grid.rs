use bevy::{
    camera::Camera,
    ecs::{
        error::Result,
        observer::On,
        prelude::{Query, Single, With}
    },
    picking::{
        events::{DragDrop, Pointer, Move}
    },
    prelude::{debug, EntityEvent, GlobalTransform, trace, Transform, Vec3Swizzles}
};
use tracing::instrument;

use crate::{RectGrid, RectGridParams};

#[instrument(skip_all)]
pub fn handle_over_grid(
    mut over: On<Pointer<Move>>,
    grid_query: Query<&RectGridParams, With<RectGrid>>,
    query: Single<(&Camera, &GlobalTransform)>
) -> Result
{
    trace!("");

    if let Some(pos) = over.hit.position {
        let entity = over.event().event_target();
        let grid = grid_query.get(entity)?;

        let (camera, global_transform) = *query;
        let pos = camera.viewport_to_world_2d(global_transform, over.pointer_location.position)?;

        trace!("{},{}", (pos.x / grid.cw).floor(), (pos.y / grid.rh).floor());
    }

    Ok(())
}

#[instrument(skip_all)]
pub fn on_piece_drop(
    mut drop: On<Pointer<DragDrop>>,
    mut piece_query: Query<&mut Transform>,
    grid_query: Query<&RectGridParams, With<RectGrid>>,
    query: Single<(&Camera, &GlobalTransform)>
) -> Result
{
    debug!("");

    if let Some(pos) = drop.hit.position {
        let dst = drop.event().event_target();
        let src = drop.event().dropped;

        debug!("{} {}", src, dst);

        let grid = grid_query.get(dst)?;
        let mut src_transform = piece_query.get_mut(src)?;

        let (camera, global_transform) = *query;
        let pos = camera.viewport_to_world_2d(global_transform, drop.pointer_location.position)?;

        debug!("{},{} {},{}", pos.x, pos.y, (pos.x / grid.cw).floor(), (pos.y / grid.rh).floor());

        // snap piece to center
        let cx = ((pos.x / grid.cw).floor() + 0.5) * grid.cw;
        let cy = ((pos.y / grid.rh).floor() + 0.5) * grid.rh;

        src_transform.translation.x = cx;
        src_transform.translation.y = cy;
    }

    Ok(())
}

/*
fn world_to_grid(v: Vec2) -> Vec2 {


}
*/
