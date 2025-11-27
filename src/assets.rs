use bevy::{
    asset::{AssetEvent, AssetServer, Handle, LoadedFolder},
    ecs::{
        change_detection::Res,
        message::MessageReader
    },
    image::Image,
    prelude::{Commands, Resource}
};

#[derive(Resource)]
pub struct SpriteHandles(pub Handle<LoadedFolder>);

pub fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>
)
{
    commands.insert_resource(SpriteHandles(asset_server.load_folder(".")));
}

pub fn is_folder_loaded(
    mut asset_events: MessageReader<AssetEvent<LoadedFolder>>,
    sprite_handles: Res<SpriteHandles>
) -> bool
{
    asset_events.read()
        .any(|e| e.is_loaded_with_dependencies(&sprite_handles.0))
}

pub fn log_images_loaded(
    mut asset_events: MessageReader<AssetEvent<Image>>,
)
{
    for e in asset_events.read() {
        eprint!(".");
    }
}
