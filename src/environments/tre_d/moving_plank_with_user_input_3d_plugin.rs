use crate::EttHakkState;
use crate::evolusjon::evolusjon_steg_plugin::Kjøretilstand;
use crate::evolusjon::hjerne_fenotype::PhenotypeNeuralNetwork;
use crate::evolusjon::phenotype_plugin::{Individ, PlankPhenotype};
use crate::genome::genome_stuff::Genome;
use crate::monitoring::camera_stuff::RENDER_LAYER_ALLE_INDIVIDER;
use crate::monitoring::simulation_teller::SimulationTotalRuntimeRunningTeller;
use avian3d::PhysicsPlugins;
use avian3d::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::KeyCode::{KeyA, KeyD, KeyX, KeyZ};
use bevy::prelude::*;
use std::vec;
// use bevy_rapier2d::na::ComplexField;
// use bevy_rapier2d::prelude::{Collider, CollisionGroups, Group, NoUserData, PhysicsSet, RapierDebugRenderPlugin, RapierPhysicsPlugin, RigidBody, Velocity};

pub struct MovingPlankWithUserInput3dPlugin;

impl Plugin for MovingPlankWithUserInput3dPlugin {
    fn build(&self, app: &mut App) {
        app
            // NOTE: Denne og SimulationGenerationTimer henger ikke sammen. Kan endres til å henge sammen, men er ikke gjort akkurat nå
            // Important note: gravity is default on, but only if ExternalForces is used https://github.com/Jondolf/avian/issues/526
            // .insert_resource(Gravity(Vector::NEG_Y * 9.81 * 100.0))
            .insert_resource(Gravity::ZERO)
            // .add_systems(Startup, spawn_plank)
            .add_systems(
                Update,
                (
                    (set_physics_time_to_paused_or_unpaused).run_if(state_changed::<Kjøretilstand>),
                    (
                        move_plank_with_keyboard_inputs,
                        impulse_plank_with_keyboard_inputs,
                        // print_done_status,
                        // print_score,
                        // print_environment_observations
                        // print_pois_velocity_and_force,
                    )
                        .run_if(in_state(Kjøretilstand::Kjørende)),
                    (set_ett_hakk_til_kjør_ett_hakk_if_input).run_if(in_state(EttHakkState::VENTER_PÅ_INPUT)),
                    (set_ett_hakk_til_vent_på_input).run_if(in_state(EttHakkState::KJØRER_ETT_HAKK)),
                )
                    .chain(),
            );
    }
}

/// Defines the state found in the cart pole environment.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct MovingPlankObservation {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

// const PLANK_STARTING_POSITION: Vec3 = Vec3 { x: 0.0, y: -150.0, z: 0.0 };
const PLANK_POSITION_CHANGE_MOVEMENT_SPEED: f32 = 0.1;
const PLANK_POSITION_VELOCITY_MOVEMENT_SPEED: f32 = 0.1;

const PLANK_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);

fn set_physics_time_to_paused_or_unpaused(kjøretistand_state: Res<State<Kjøretilstand>>, mut physics_time: ResMut<Time<Physics>>) {
    match kjøretistand_state.get() {
        Kjøretilstand::Pause => physics_time.pause(),
        Kjøretilstand::Kjørende => physics_time.unpause(),
        Kjøretilstand::EvolutionOverhead => physics_time.unpause(),
    }
}

// Docs for avian froce rework  https://github.com/Jondolf/avian/pull/770
// todo tyngdekraft kan endres til
// // Apply a constant force of 10 N in the positive Y direction.
// ConstantForce::new(0.0, 10.0, 0.0),
// The forces are only constant in the sense that they persist across time steps. They can still be modified in systems like normal.

fn print_pois_velocity_and_force(mut query: Query<(&Transform, &PlankPhenotype, &LinearVelocity, Forces), (With<PlankPhenotype>)>) {
    for (translation, plank, linvel, external_force) in query.iter_mut() {
        println!("translation {:#?}", translation);
        println!("linvel {:#?}", linvel);
        // println!("external_force {:#?}", external_force);
        println!("----------------------------")
    }
}

static INDIVIDUALS_COLLIDE_IN_SIMULATION: bool = false;

fn move_plank_with_keyboard_inputs(mut query: Query<&mut Transform, With<PlankPhenotype>>, keyboard_input: Res<ButtonInput<KeyCode>>, time: Res<Time>) {
    let mut delta_x = 0.0;
    if keyboard_input.pressed(KeyA) {
        delta_x -= PLANK_POSITION_CHANGE_MOVEMENT_SPEED;
    }
    if keyboard_input.pressed(KeyD) {
        delta_x += PLANK_POSITION_CHANGE_MOVEMENT_SPEED;
    }
    if delta_x != 0.0 {
        for mut transform in query.iter_mut() {
            transform.translation.x += delta_x * time.delta_secs();
        }
    }
}

fn impulse_plank_with_keyboard_inputs(
    mut query: Query<&mut LinearVelocity, With<PlankPhenotype>>,
    // &mut Velocity, With<PlankPhenotype>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut delta_x = 0.0;

    if keyboard_input.pressed(KeyZ) {
        delta_x -= PLANK_POSITION_VELOCITY_MOVEMENT_SPEED;
    }
    if keyboard_input.pressed(KeyX) {
        delta_x += PLANK_POSITION_VELOCITY_MOVEMENT_SPEED;
    }
    if delta_x != 0.0 {
        for mut velocity in query.iter_mut() {
            // velocity.linvel.x += delta_x * time.delta_secs();
            velocity.0.x += delta_x * time.delta_secs();
            // println!("impulse plank has delta x { }", velocity.0.x);
        }
    }
}

fn set_ett_hakk_til_vent_på_input(mut next_state: ResMut<NextState<EttHakkState>>, mut next_kjøretistand_state: ResMut<NextState<Kjøretilstand>>) {
    next_state.set(EttHakkState::VENTER_PÅ_INPUT);
    next_kjøretistand_state.set(Kjøretilstand::Pause);
}

fn set_ett_hakk_til_kjør_ett_hakk_if_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<EttHakkState>>,
    mut next_kjøretistand_state: ResMut<NextState<Kjøretilstand>>,
    mut physics_time: ResMut<Time<Physics>>,
) {
    let mut input_exist = false;
    if keyboard_input.pressed(KeyA) {
        input_exist = true;
    }
    if keyboard_input.pressed(KeyD) {
        input_exist = true;
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        input_exist = true;
    }
    if input_exist {
        physics_time.unpause();
        next_state.set(EttHakkState::KJØRER_ETT_HAKK);
        next_kjøretistand_state.set(Kjøretilstand::Kjørende);
    }
}

fn get_observations(transform: Transform) -> MovingPlankObservation {
    // let translation = query.single().unwrap().translation.clone();
    return MovingPlankObservation {
        x: transform.translation.x,
        y: transform.translation.y,
    };
}

fn get_simulation_time(query: Query<&Transform, With<PlankPhenotype>>) -> MovingPlankObservation {
    let translation = query.single().unwrap().translation.clone();
    return MovingPlankObservation {
        x: translation.x,
        y: translation.y,
    };
}

fn print_environment_observations(query: Query<&Transform, With<PlankPhenotype>>) {
    for transform in query.iter() {
        println!("Moving plank observations : {:?}", get_observations(transform.clone()));
    }
}

fn get_score(time_alive: Res<SimulationTotalRuntimeRunningTeller>) -> u32 {
    let score = time_alive.count.clone();
    return score;
}

fn print_score(time_alive: Res<SimulationTotalRuntimeRunningTeller>) {
    println!("score is time alive: {}", get_score(time_alive));
}

// fn check_if_done(query: Query<&Transform, With<Plank>>, window: Query<&Window>) -> bool {
fn check_if_done(transform: Transform, window: Window) -> bool {
    let max_width = window.width() * 0.5;
    let max_height = window.height() * 0.5;
    let translation = transform.translation.clone();
    if translation.x.abs() > max_width {
        return true;
    }
    if translation.y.abs() > max_height {
        return true;
    }
    return false;
}

fn print_done_status(query: Query<&Transform, With<PlankPhenotype>>, window: Query<&Window>) {
    println!("------All done statues -------------------");
    let window = window.single().unwrap().clone();

    for transform in query.iter() {
        println!("Er done ? : {}", check_if_done(transform.clone(), window.clone()));
    }
    println!("-------------------------");
}

fn reset_plank(mut query: Query<&mut Transform, With<PlankPhenotype>>) {
    let mut translation = query.single_mut().unwrap().translation;
    translation.x = 0.0;
    translation.y = 0.0;
    translation.z = 0.0;
}
