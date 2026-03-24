use bevy::{
    ecs::{
        change_detection::Res,
        prelude::{Commands, With, Without}

    },
    prelude::{debug, Entity, Query, Result}
};
use serde::{
    Deserialize, Deserializer,
    de::{
        DeserializeSeed, MapAccess, SeqAccess, Visitor,
        value::MapAccessDeserializer
    }
};
use std::{
    fmt,
    fs::File,
    io::BufReader
};
use tracing::instrument;

use crate::{
    LogPath,
    clone::CloneEdit,
    create::CreateEdit,
    delete::DeleteEdit,
    flip::FlipEdit,
    log::{EditOf, Edits, EditsComplete, EditType},
    r#move::MoveEdit,
    rotate::RotateEdit
};

struct EditProxyInSeed<'c, 'w, 's> {
    entity: Entity,
    commands: &'c mut Commands<'w, 's>
}

impl<'de> DeserializeSeed<'de> for EditProxyInSeed<'_, '_, '_> {
    type Value = EditProxyIn;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>
    {
        deserializer.deserialize_any(EditProxyInVisitor {
            entity: self.entity,
            commands: self.commands
        })
    }
}

#[derive(Debug, Deserialize)]
enum EditProxyIn {
    Group,
    #[serde(untagged)]
    Leaf(EditProxyInEdit)
}

struct EditProxyInVisitor<'c, 'w, 's> {
    entity: Entity,
    commands: &'c mut Commands<'w, 's>
}

impl<'de> Visitor<'de> for EditProxyInVisitor<'_, '_, '_> {
    type Value = EditProxyIn;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "an edit or a sequence of edits")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>
    {
        let mut children = vec![];

        loop {
            let mut ec = self.commands.spawn_empty();
            let entity = ec.id();

            let seed = EditProxyInSeed {
                entity,
                commands: ec.commands_mut()
            };

            let Some(ep) = seq.next_element_seed::<EditProxyInSeed>(seed)? else { self.commands.entity(entity).despawn(); break; };

            match ep {
                EditProxyIn::Leaf(EditProxyInEdit::Clone(ed)) => {
                    ec.insert((EditType::Clone, ed));
                },
                EditProxyIn::Leaf(EditProxyInEdit::Create(ed)) => {
                    ec.insert((EditType::Create, ed));
                },
                EditProxyIn::Leaf(EditProxyInEdit::Delete(ed)) => {
                    ec.insert((EditType::Delete, ed));
                },
                EditProxyIn::Leaf(EditProxyInEdit::Flip(ed)) => {
                    ec.insert((EditType::Flip, ed));
                },
                EditProxyIn::Group => {
                    ec.insert(EditType::Group);
                },
                EditProxyIn::Leaf(EditProxyInEdit::Move(ed)) => {
                    ec.insert((EditType::Move, ed));
                },
                EditProxyIn::Leaf(EditProxyInEdit::Rotate(ed)) => {
                    ec.insert((EditType::Rotate, ed));
                }
            }

            children.push(entity);
        }

        // set up the group
        self.commands.entity(self.entity)
            .insert(Edits::default());

        // add all the children to it
        for e in children {
            self.commands.entity(e)
                .insert(EditOf(self.entity));
        }

        Ok(EditProxyIn::Group)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>
    {
        Deserialize::deserialize(MapAccessDeserializer::new(map))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
enum EditProxyInEdit {
    Clone(CloneEdit),
    Create(CreateEdit),
    Delete(DeleteEdit),
    Flip(FlipEdit),
    Move(MoveEdit),
    Rotate(RotateEdit)
}

#[instrument(skip_all)]
pub fn deserialize_edits(
    log_path: Res<LogPath>,
    root_query: Query<Entity, (With<Edits>, Without<EditOf>)>,
    mut commands: Commands
) -> Result {
    debug!("");

    let Some(path) = log_path.0.as_ref() else { return Ok(()); };

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let root_entity = root_query.single()?;

// TODO: ensure that root is a group

    let r = EditProxyInSeed {
        entity: root_entity,
        commands: &mut commands
    };

    let mut d = serde_json::Deserializer::from_reader(reader);
    let _ = r.deserialize(&mut d)?;

    commands.trigger(EditsComplete);
    Ok(())
}
