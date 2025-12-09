use bevy::{
    asset::{
        AssetEvent, AssetPath, AssetServer, Handle,
        io::AssetSourceId
    },
    ecs::{
        change_detection::{Res, ResMut},
        error::Result,
        message::MessageReader
    },
    image::Image,
    prelude::{debug, AssetId, Commands, Resource}
};
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    path::Path
};

use crate::gamebox::GameBox;

#[derive(Resource)]
pub struct LoadingHandles(pub HashSet<AssetId<Image>>);

#[derive(Resource)]
pub struct SpriteHandles(pub HashMap<String, Handle<Image>>);

pub fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) -> Result
{
    let args = std::env::args().collect::<Vec<_>>();

    let game: GameBox = toml::from_str(&std::fs::read_to_string(&args[1])?)?;

   // FIXME: unwrap
    let base = Path::new(&args[1])
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let src = AssetSourceId::from(base);

    let sh = game.images.iter()
        .map(|(k, v)| (
            k.clone(),
            asset_server.load(
                AssetPath::from_path(Path::new(v)).with_source(&src)
            )
        ))
        .collect::<HashMap<_, _>>();

    let lh = sh.iter()
        .map(|h| h.1.id())
        .collect::<HashSet<_>>();

// TODO: check that actions exist, etc
// TODO: check faces against images
    assert!(
        game.piece.iter()
            .flat_map(|p| &p.faces)
            .unique()
            .all(|f| sh.contains_key(f))
    );

    commands.insert_resource(game);
    commands.insert_resource(LoadingHandles(lh));
    commands.insert_resource(SpriteHandles(sh));

    Ok(())
}

pub fn mark_images_loaded(
    mut asset_events: MessageReader<AssetEvent<Image>>,
    mut loading_handles: ResMut<LoadingHandles>
)
{
    for e in asset_events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = e {
            loading_handles.0.remove(id);
            eprint!(".");
        }
    }
}
