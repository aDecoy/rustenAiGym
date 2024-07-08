use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::prelude::RapierDebugRenderPlugin;

use crate::environments::moving_plank::MovingPlankPlugin;
use crate::environments::simulation_teller::SimulationRunningTellerPlugin;

mod environments;

fn main() {
    println!("Starting up AI GYM");
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::DX12),
                ..default()
            }),
            synchronous_pipeline_compilation: false,
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_state(Kjøretilstand::Kjørende)
        .add_systems(Startup, (
            setup_camera,
        ))
        // Environment spesific : Later changed
        .add_plugins(MovingPlankPlugin)
        .add_plugins(SimulationRunningTellerPlugin)

        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}


#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum Kjøretilstand {
    #[default]
    Meny,
    Kjørende,
    // EttHakk { speed : f32},
    EttHakk,
}