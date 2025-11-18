use crate::environments::gammelt_2d::individ_watching_2d_camera::IndividWatching2dCameraPlugin;
use crate::{EnvValg, ACTIVE_ENVIROMENT};
use avian2d::prelude::{Collider, CollisionLayers, Friction, LayerMask, Restitution, RigidBody};
use bevy::camera::visibility::RenderLayers;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::*;
use lazy_static::lazy_static;
use std::collections::HashMap;
use crate::environments::gammelt_2d::spawn_2d_individ_plugin::Spawn2dIndividPlugin;

const GROUND_LENGTH: f32 = 5495.;
const GROUND_HEIGHT: f32 = 10.;
const GROUND_COLOR: Color = Color::srgb(0.30, 0.75, 0.5);
const GROUND_STARTING_POSITION: Vec3 = Vec3 { x: 0.0, y: -300.0, z: 1.0 };

const ROOF_STARTING_POSITION: Vec3 = Vec3 { x: 0.0, y: 300.0, z: 1.0 };
// const GROUND_STARTING_POSITION: Vec3 = Vec3 { x: 0.0, y: -300.0, z: 1.0 };

pub struct LunarLanderEnvironment2d;

impl Plugin for LunarLanderEnvironment2d {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(IndividWatching2dCameraPlugin)
            .add_plugins(Spawn2dIndividPlugin)
            .add_systems(Startup, (spawn_ground, spawn_roof, spawn_landing_target));
    }
}

lazy_static! {
    static ref LANDING_SITE_PER_ENVIRONMENT: HashMap<EnvValg, Vec2> = {
        HashMap::from([
            (EnvValg::Homing, Vec2 { x: 100.0, y: -100.0 }),
            (
                EnvValg::HomingGroud,
                Vec2 {
                    x: 00.0,
                    y: GROUND_STARTING_POSITION.y + GROUND_HEIGHT,
                },
            ),
            (
                EnvValg::HomingGroudY,
                Vec2 {
                    x: 00.0,
                    y: GROUND_STARTING_POSITION.y + GROUND_HEIGHT,
                },
            ),
        ])
    };
    pub static ref LANDING_SITE: Vec2 = LANDING_SITE_PER_ENVIRONMENT[&ACTIVE_ENVIROMENT];
}

fn spawn_ground(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn((
        RigidBody::Static,
        Mesh2d(meshes.add(Rectangle::default()).into()),
        MeshMaterial2d(materials.add(GROUND_COLOR)),
        Transform::from_translation(GROUND_STARTING_POSITION).with_scale(
            Vec2 {
                x: GROUND_LENGTH,
                y: GROUND_HEIGHT,
            }
            .extend(1.),
        ),
        // Sleeping::disabled(),
        Collider::rectangle(1.0, 1.0),
        Restitution::new(0.0),
        Friction::new(0.5),
        CollisionLayers::new(0b0010, LayerMask::ALL),
        RenderLayers::layer(1),
    ));
}

fn spawn_landing_target(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn((
        RigidBody::Static,
        Mesh2d(meshes.add(Circle::default()).into()),
        MeshMaterial2d(materials.add(Color::linear_rgb(1.0, 0.0, 0.0))),
        Transform::from_translation(LANDING_SITE.extend(0.0)).with_scale(Vec2 { x: 10.0, y: 10.0 }.extend(1.)),
        RenderLayers::layer(1),
        // Sleeping::disabled(),
    ));
}

fn spawn_roof(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn((
        RigidBody::Static,
        Mesh2d(meshes.add(Rectangle::default()).into()),
        MeshMaterial2d(materials.add(GROUND_COLOR)),
        Transform::from_translation(ROOF_STARTING_POSITION).with_scale(Vec2 { x: GROUND_LENGTH, y: 10.0 }.extend(1.)),
        // Sleeping::disabled(),
        Collider::rectangle(1.0, 1.0),
        Restitution::new(0.0),
        Friction::new(0.5),
        CollisionLayers::new(0b0010, LayerMask::ALL),
        RenderLayers::layer(1),
    ));
}

fn every_time_if_stop_on_right_window() -> impl SystemCondition<()> {
    IntoSystem::into_system(|mut flag: Local<bool>| {
        *flag = match ACTIVE_ENVIROMENT {
            EnvValg::Høyre | EnvValg::Fall | EnvValg::FallVelocityHøyre | EnvValg::FallExternalForcesHøyre => true,
            _ => false,
        };
        *flag
    })
}
