use bevy::{
    app::{App, Startup, Update},
    asset::AssetServer,
    color::Color,
    input::ButtonInput,
    math::Vec3,
    prelude::{
        Camera2dBundle, Commands, Component, Deref, DerefMut, Entity, KeyCode, Query, Res,
        Resource, With,
    },
    sprite::{Sprite, SpriteBundle},
    transform::components::Transform,
    utils::{
        hashbrown::{hash_map::Entry, HashSet},
        HashMap,
    },
    DefaultPlugins,
};
use bevy_indices::EntityIndex;

fn main() {
    let app = App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, add_stuff)
        .add_systems(Update, select_entities)
        .run();
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct TestIndex(usize);

impl EntityIndex for TestIndex {
    fn on_insert(
        mut world: bevy::ecs::world::DeferredWorld<'_>,
        pairs: [(Self, bevy::prelude::Entity)],
    ) {
        let store = world.get_index_store_mut::<TestIndexMap>();

        for (key, entity) in pairs {
            match store.entry(key) {
                Entry::Occupied(mut o) => {
                    o.get_mut().insert(entity);
                }
                Entry::Vacant(v) => {
                    let mut set = HashSet::default();
                    set.insert(entity);
                    v.insert(set);
                }
            };
        }
    }

    fn on_remove(
        mut world: bevy::ecs::world::DeferredWorld<'_>,
        pairs: [(Self, bevy::prelude::Entity)],
    ) {
        let store = world.get_index_store_mut::<TestIndexMap>();

        for (key, entity) in pairs {
            match store.entry(key) {
                Entry::Occupied(mut o) => {
                    let set = o.get_mut();
                    set.remove(&entity);
                    if set.len() == 0 {
                        o.remove();
                    }
                }
                Entry::Vacant(v) => {
                    panic!("HOW DID WE GET HERE???")
                }
            };
        }
    }
}

#[derive(Deref, DerefMut, Resource, Default)]
pub struct TestIndexMap(HashMap<TestIndex, HashSet<Entity>>);

#[derive(Component)]
struct Selected;

fn add_stuff(mut commands: Commands, asset_server: Res<AssetServer>) {
    for i in 1..=5 {
        commands.spawn(Camera2dBundle::default());
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("bevy_bird_dark.png"),
                transform: Transform::from_translation(Vec3::new(
                    -768.0 + i as f32 * 256.0,
                    0.0,
                    0.0,
                )),
                ..Default::default()
            },
            // Add an index to this!
            Index(TestIndex(i)),
            Selected,
        ));
    }
}

fn select_entities(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut ents: Query<(&mut Sprite, Option<&Selected>)>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        add_selection_to(2, &mut commands, &mut ents);
        add_selection_to(5, &mut commands, &mut ents);
    }
    if keys.just_pressed(KeyCode::Digit2) {
        add_selection_to(1, &mut commands, &mut ents);
        add_selection_to(3, &mut commands, &mut ents);
    }
    if keys.just_pressed(KeyCode::Digit3) {
        add_selection_to(2, &mut commands, &mut ents);
        add_selection_to(4, &mut commands, &mut ents);
    }
    if keys.just_pressed(KeyCode::Digit4) {
        add_selection_to(3, &mut commands, &mut ents);
        add_selection_to(5, &mut commands, &mut ents);
    }
    if keys.just_pressed(KeyCode::Digit5) {
        add_selection_to(4, &mut commands, &mut ents);
        add_selection_to(1, &mut commands, &mut ents);
    }
}

fn add_selection_to(
    index: usize,
    commands: &mut Commands,
    ents: &mut Query<(&mut Sprite, Option<&Selected>)>,
) {
    if let Some(ent) = ents
        .get_indexed(&TestIndex(index))
        .and_then(|ents| ents.iter().next())
    {
        if let Ok((mut sprite, selected)) = ents.get_mut(*ent) {
            if selected.is_some() {
                commands.entity(*ent).remove::<Selected>();
                sprite.color = Color::BLACK;
            } else {
                commands.entity(*ent).insert(Selected);
                sprite.color = Color::WHITE;
            }
        }
    }
}
