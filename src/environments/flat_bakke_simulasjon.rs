use crate::environments::lunar_lander_environment::LunarLanderEnvironment;
use avian2d::prelude::{Collider, CollisionLayers, Friction, LayerMask, Restitution, RigidBody};
use bevy::asset::Assets;
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;

pub struct FlatBakkeSimulasjonPlugin;

impl Plugin for FlatBakkeSimulasjonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_ground));
    }
}

const GROUND_LENGTH: f32 = 5495.;
const GROUND_HEIGHT: f32 = 10.;
const GROUND_COLOR: Color = Color::srgb(0.30, 0.75, 0.5);
const GROUND_STARTING_POSITION: Vec3 = Vec3 { x: 0.0, y: -300.0, z: 1.0 };

fn spawn_ground(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn((
        RigidBody::Static,
        Mesh3d(meshes.add(Rectangle::default()).into()),
        MeshMaterial2d(materials.add(GROUND_COLOR)),
        Transform::from_translation(GROUND_STARTING_POSITION).with_scale(
            Vec3 {
                x: GROUND_LENGTH,
                y: GROUND_LENGTH,
                z: GROUND_HEIGHT
            }
        ),
        // Sleeping::disabled(),
        // Collider::c(1.0, 1.0),
        Restitution::new(0.0),
        Friction::new(0.5),
        CollisionLayers::new(0b0010, LayerMask::ALL),
        RenderLayers::layer(1),
    ));
}