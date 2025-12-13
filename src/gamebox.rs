use bevy::ecs::prelude::Resource;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct PieceType {
    pub name: String,
    #[serde(default)]
    pub faces: Vec<String>,
    #[serde(default)]
    pub actions: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct MapType {
    pub image: String,
    pub x: f32,
    pub y: f32
}

#[derive(Debug, Deserialize)]
pub struct SurfaceType {
    #[serde(default)]
    pub map: Vec<MapType>
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ImageDescriptor {
    File(String),
    Crop {
        src: String,
        x: u32,
        y: u32,
        w: u32,
        h: u32
    },
    Grid {
        src: String,
        x: u32,
        y: u32,
        cols: u32,
        rows: u32,
        cw: u32,
        rh: u32,
        #[serde(default)]
        cgap: i32,
        #[serde(default)]
        rgap: i32
    }
}

#[derive(Debug, Deserialize)]
struct MaybeGameBox {
    #[serde(default)]
    pub images: HashMap<String, ImageDescriptor>,
    #[serde(default)]
    pub piece: Vec<PieceType>,
    pub surface: SurfaceType
}

// TODO: rename fields? pieces is probably a nicer name?
#[derive(Debug, Deserialize, Resource)]
#[serde(try_from = "MaybeGameBox")]
pub struct GameBox {
    pub images: HashMap<String, ImageDescriptor>,
    pub piece: Vec<PieceType>,
    pub surface: SurfaceType
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("Malformed gamebox data")]
pub struct GameBoxError;

// TODO: check that actions exist, etc

impl TryFrom<MaybeGameBox> for GameBox {
    type Error = GameBoxError;

    fn try_from(m: MaybeGameBox) -> Result<Self, Self::Error> {
// TODO: check that grid keys are formatted name@c,r

        // check that face keys exist 
        if !m.piece.iter()
            .flat_map(|p| &p.faces)
            .unique()
            .map(|f| f.rsplit_once('@').map(|(l, r)| l).unwrap_or(f))
            .all(|f| m.images.contains_key(f))
        {
            return Err(GameBoxError);
        }

        // TODO: follow chains to reject loops
        // check that crop source keys exist 
        if !m.images.iter()
            .all(|(k, v)| match v {
                ImageDescriptor::Crop { src, .. } => m.images.contains_key(src),
                ImageDescriptor::Grid { src, .. } => m.images.contains_key(src),
                _ => true
            })
        {
            return Err(GameBoxError);
        }

        Ok(GameBox {
            images: m.images,
            piece: m.piece,
            surface: m.surface
        })
    }
}
