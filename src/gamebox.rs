use bevy::ecs::prelude::Resource;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashMap;

use crate::actionfunc::ActionFunc;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub enum Anchor {
    BottomLeft,
    BottomCenter,
    BottomRight,
    CenterLeft,
    #[default]
    Center,
    CenterRight,
    TopLeft,
    TopCenter,
    TopRight
}

impl From<Anchor> for bevy::sprite::Anchor {
    fn from(a: Anchor) -> Self {
        match a {
            Anchor::BottomLeft => bevy::sprite::Anchor::BOTTOM_LEFT,
            Anchor::BottomCenter => bevy::sprite::Anchor::BOTTOM_CENTER,
            Anchor::BottomRight => bevy::sprite::Anchor::BOTTOM_RIGHT,
            Anchor::CenterLeft => bevy::sprite::Anchor::CENTER_LEFT,
            Anchor::Center => bevy::sprite::Anchor::CENTER,
            Anchor::CenterRight => bevy::sprite::Anchor::CENTER_RIGHT,
            Anchor::TopLeft => bevy::sprite::Anchor::TOP_LEFT,
            Anchor::TopCenter => bevy::sprite::Anchor::TOP_CENTER,
            Anchor::TopRight => bevy::sprite::Anchor::TOP_RIGHT
        }
    }
}

const fn default_scale() -> f32 {
    1.0
}

#[derive(Debug, Deserialize)]
pub struct GroupDefinition {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default = "default_scale")]
    pub s: f32,
    #[serde(default)]
    pub a: f32,
    #[serde(default)]
    pub anchor: Anchor,
    pub children: Vec<SurfaceItem>
}

#[derive(Debug, Deserialize)]
pub struct MapDefinition {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default = "default_scale")]
    pub s: f32,
    #[serde(default)]
    pub a: f32,
    #[serde(default)]
    pub anchor: Anchor,
    pub image: String
}

// first hex column: high or low?
// show/hide grid
// grid color, opacity
// grid thickness
// either hs or hw, hh

#[derive(Debug, Deserialize)]
pub struct RectGridDefinition {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default = "default_scale")]
    pub s: f32,
    #[serde(default)]
    pub a: f32,
    #[serde(default)]
    pub anchor: Anchor,
    pub cols: u32,
    pub rows: u32,
    pub cw: f32,
    pub rh: f32
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum ColumnStagger {
    Low,
    High
}

#[derive(Debug, Deserialize)]
pub struct HexGridDefinition {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default = "default_scale")]
    pub s: f32,
    #[serde(default)]
    pub a: f32,
    #[serde(default)]
    pub anchor: Anchor,
    pub cols: u32,
    pub rows: u32,
    pub hw: f32,
    pub hh: f32,
    pub hs: f32,
    pub first: ColumnStagger
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GridDefinition {
    Rect(RectGridDefinition),
    Hex(HexGridDefinition)
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SurfaceItem {
    Map(MapDefinition),
    Grid(GridDefinition),
    Group(GroupDefinition)
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ImageDefinition {
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

#[derive(Clone, Debug, Deserialize)]
pub struct Action {
    pub label: String,
    pub action: ActionFunc,
    pub key: Option<String>
}

#[derive(Debug, Deserialize)]
pub struct PieceType {
    pub name: String,
    #[serde(default)]
    pub faces: Vec<String>,
    #[serde(default)]
    pub actions: Vec<Action>
}

#[derive(Debug, Deserialize)]
struct MaybeGameBox {
    #[serde(default)]
    pub images: HashMap<String, ImageDefinition>,
    #[serde(default)]
    pub piece: Vec<PieceType>,
    pub surface: SurfaceItem
}

// TODO: rename fields? pieces is probably a nicer name?
#[derive(Debug, Deserialize, Resource)]
#[serde(try_from = "MaybeGameBox")]
pub struct GameBox {
    pub images: HashMap<String, ImageDefinition>,
    pub piece: Vec<PieceType>,
    pub surface: SurfaceItem
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
            .map(|f| f.rsplit_once('@').map(|(l, _)| l).unwrap_or(f))
            .all(|f| m.images.contains_key(f))
        {
            return Err(GameBoxError);
        }

        // TODO: follow chains to reject loops
        // check that crop source keys exist 
        if !m.images.iter()
            .all(|(k, v)| match v {
                ImageDefinition::Crop { src, .. } => m.images.contains_key(src),
                ImageDefinition::Grid { src, .. } => m.images.contains_key(src),
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
