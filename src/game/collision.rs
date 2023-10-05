use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::resources::{Fruit, ScoreTracker};
use crate::setup::Score;

use super::{create_fruit_bundle, Alive};

#[derive(Component)]
pub struct MarkForDelete;

pub fn collision(
    mut collisions: EventReader<CollisionEvent>,
    asset_server: Res<AssetServer>,
    mut score_tracker: ResMut<ScoreTracker>,
    fruits: Query<(&Fruit, &mut Transform)>,
    mut score_query: Query<&mut Text, With<Score>>,
    mut not_alive_fruits: Query<(&Fruit, &mut AdditionalMassProperties), Without<Alive>>,
    mut commands: Commands,
) {
    for collision in collisions.iter() {
        if let CollisionEvent::Started(collider_a, collider_b, _) = collision {
            // TODO: change this event away from collision and into a time-tracked event
            if let Ok((_, mut mprops)) = not_alive_fruits.get_mut(*collider_a) {
                *mprops = AdditionalMassProperties::Mass(0.001);
                commands.entity(*collider_a).insert(Alive);
            }
            if let Ok((_, mut mprops)) = not_alive_fruits.get_mut(*collider_b) {
                *mprops = AdditionalMassProperties::Mass(0.001);
                commands.entity(*collider_a).insert(Alive);
            }

            if let Ok([(fruit_a, transform_a), (fruit_b, transform_b)]) =
                fruits.get_many([*collider_a, *collider_b])
            {
                // TODO: add a handler for if three fruits of the same size collide at the same time
                if fruit_a.size == fruit_b.size {
                    let new_x = (transform_a.translation.x + transform_b.translation.x) / 2.0;
                    let new_y = (transform_a.translation.y + transform_b.translation.y) / 2.0;
                    // Fruit.merged_size returns None if two largest fruits collide
                    // in this case, both are despawned, and no new fruits created
                    if let Some(fruit) = fruit_a.merge() {
                        let texture_handle = asset_server.load(&fruit.image_file_name);
                        score_tracker.add_score(fruit.score);
                        let mut score = score_query.single_mut();
                        score.sections[0].value = score_tracker.score.to_string();
                        commands.spawn(create_fruit_bundle(texture_handle, new_x, new_y, fruit));
                    }
                    commands
                        .entity(*collider_a)
                        .remove::<(RigidBody, SpriteBundle, Collider)>()
                        .insert(MarkForDelete);
                    commands
                        .entity(*collider_b)
                        .remove::<(RigidBody, SpriteBundle, Collider)>()
                        .insert(MarkForDelete);
                }
            }
        }
    }
}

pub fn remove_used_fruits(
    fruits_marked_for_delete: Query<Entity, With<MarkForDelete>>,
    mut commands: Commands,
) {
    for fruit in fruits_marked_for_delete.iter() {
        commands.entity(fruit).despawn();
    }
}