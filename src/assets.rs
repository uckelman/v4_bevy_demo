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
    math::{URect, UVec2},
    prelude::{debug, AssetId, Assets, Commands, Resource, TextureAtlas, TextureAtlasLayout}
};
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    path::Path
};

use crate::gamebox::{
    GameBox,
    ImageDescriptor
};

#[derive(Clone, Debug)]
pub enum ImageSource {
    Single(Handle<Image>),
    Crop {
        handle: Handle<Image>,
        atlas: TextureAtlas
    }
}

#[derive(Resource)]
pub struct SpriteHandles(pub HashMap<String, ImageSource>);

#[derive(Resource)]
pub struct LoadingHandles(pub HashSet<AssetId<Image>>);

pub fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
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

    // begin loading the file-source images
    let files = game.images.iter()
        .filter_map(|(k, v)| match v {
            ImageDescriptor::File(f) => Some(
                (
                    k.clone(),
                    asset_server.load(
                        AssetPath::from_path(Path::new(f)).with_source(&src)
                    )
                )
            ),
            _ => None
        })
        .collect::<HashMap<_, _>>();

    // track which asset handles have loaded
    let lh = files.values()
        .map(|v| v.id())
        .collect::<HashSet<_>>();

    commands.insert_resource(LoadingHandles(lh));

// TODO: handle nesting of grid and crop
    // make texture atlas layouts for the grids
    let layouts = game.images.iter()
        .filter_map(|(k, v)| match v {
            ImageDescriptor::Grid { src, x, y, cols, rows, cw, rh, cgap, rgap } => {
                let x = *x as i32;
                let y = *y as i32;
                let cols = *cols as i32;
                let rows = *rows as i32;
                let cw = *cw as i32;
                let rh = *rh as i32;

                let tal = TextureAtlasLayout {
                    size: UVec2::new(
                        ((x + cw * cols) + cgap * (cols - 1)) as u32,
                        ((y + rh * rows) + rgap * (rows - 1)) as u32
                    ),
                    textures: (0..rows)
                        .cartesian_product(0..cols)
                        .map(|(r, c)|
                            URect::new(
                                (x + c * (cw + cgap)) as u32,
                                (y + r * (rh + rgap)) as u32,
                                ((x + (c + 1) * cw) + c * cgap) as u32,
                                ((y + (r + 1) * rh) + r * rgap) as u32
                            )
                        )
                        .collect::<Vec<_>>()
                };

                Some((k, texture_atlas_layouts.add(tal)))
            },
            _ => None
        })
        .collect::<HashMap<_, _>>();

    // add the derived assets to the image sources
    let derived = game.images.iter()
        .flat_map(|(k, v)| match v {
            ImageDescriptor::Crop { src, x, y, w, h } => {
                let tal = TextureAtlasLayout {
                    size: UVec2::new(x + w, y + h),
                    textures: vec![URect::new(*x, *y, x + w, y + h)]
                };
                let layout = texture_atlas_layouts.add(tal);

                vec![
                    (
                        k.clone(),
                        ImageSource::Crop {
                            handle: files.get(src).unwrap().clone(),
                            atlas: TextureAtlas { layout, index: 0 }
                        }
                    )
                ]
            },
            ImageDescriptor::Grid { src, cols, rows, .. } => {
                (0..*rows)
                    .cartesian_product(0..*cols)
                    .map(|(r, c)|
                        (
                            format!("{k}@{c},{r}"),
                            ImageSource::Crop {
                                handle: files.get(src).unwrap().clone(),
                                atlas: TextureAtlas {
                                    layout: layouts.get(k).unwrap().clone(),
                                    index: (r * cols + c) as usize
                                }
                            }
                        )
                    )
                    .collect::<Vec<_>>()
            },
            _ => vec![]
        });

    let sh = files.iter()
        .map(|(k, v)| (k.clone(), ImageSource::Single(v.clone())))
        .chain(derived)
        .collect::<HashMap<_, _>>();

    commands.insert_resource(SpriteHandles(sh));
    commands.insert_resource(game);

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
