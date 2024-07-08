use bevy::prelude::*;
use bevy::prelude::KeyCode::{KeyA, KeyD};
use bevy::sprite::MaterialMesh2dBundle;
use bevy_rapier2d::na::ComplexField;
use bevy_rapier2d::prelude::{Collider, NoUserData, RapierDebugRenderPlugin, RapierPhysicsPlugin};

use crate::environments::simulation_teller::SimulationRunningTeller;
use crate::{Env, Kjøretilstand, RenderMode, StepFeedback};

pub struct MovingPlankPlugin;

impl MovingPlankPlugin {}

impl Plugin for MovingPlankPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(Startup, spawn_plank)
            .add_systems(Update, (
                (
                    move_plank,
                    // print_done_status,
                    // print_score,
                    // print_environment_observations
                ).run_if(in_state(Kjøretilstand::Kjørende)),
                (move_plank_and_change_state_to_meny_on_input).run_if(in_state(Kjøretilstand::EttHakk)),
            ));
    }
}


#[derive(Component)]
struct Plank;

/// Defines the state found in the cart pole environment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovingPlankObservation {
    x: f32,
    y: f32,
}


const PLANK_STARTING_POSITION: Vec3 = Vec3 { x: 0.0, y: -150.0, z: 1.0 };
const PLANK_LENGTH: f32 = 95.;
const PLANK_HIGHT: f32 = 30.;
const PLANK_MOVEMENT_SPEED: f32 = 10.0;

const PLANK_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

fn spawn_plank(mut commands: Commands,
               mut meshes: ResMut<Assets<Mesh>>,
               mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
                       MaterialMesh2dBundle {
                           mesh: meshes.add(Rectangle::default()).into(),
                           material: materials.add(PLANK_COLOR),
                           transform: Transform::from_translation(PLANK_STARTING_POSITION)
                               .with_scale(Vec2 { x: PLANK_LENGTH, y: PLANK_HIGHT }.extend(1.)),
                           ..default()
                       },
                       Plank,
                       // Collider::cuboid(PLANK_LENGTH, PLANK_HIGHT),
                       Collider::cuboid(0.5, 0.5),
                   ), );
}

fn move_plank(mut query: Query<&mut Transform, With<Plank>>,
              keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut delta_x = 0.0;
    if (keyboard_input.pressed(KeyA)) {
        delta_x -= PLANK_MOVEMENT_SPEED;
    }
    if (keyboard_input.pressed(KeyD)) {
        delta_x += PLANK_MOVEMENT_SPEED;
    }
    let mut transform = query.single_mut();
    transform.translation.x += delta_x;
}

// todo En mer "bevy" måte å gjøre ette på er nokk å ha enda en state, som styrer om EttHakk har motatt input eller venter på input
fn move_plank_and_change_state_to_meny_on_input(mut query: Query<&mut Transform, With<Plank>>,
                                                keyboard_input: Res<ButtonInput<KeyCode>>,
                                                mut next_state: ResMut<NextState<Kjøretilstand>>,
) {
    let mut delta_x = 0.0;
    let mut input_exist = false;
    if (keyboard_input.pressed(KeyA)) {
        delta_x -= PLANK_MOVEMENT_SPEED;
        input_exist = true;
    }
    if (keyboard_input.pressed(KeyD)) {
        delta_x += PLANK_MOVEMENT_SPEED;
        input_exist = true;
    }
    let mut transform = query.single_mut();
    transform.translation.x += delta_x;

    if input_exist {
        next_state.set(Kjøretilstand::Meny);
    }
}
fn get_observations(query: Query<&Transform, With<Plank>>) -> MovingPlankObservation {
    let translation = query.get_single().unwrap().translation.clone();
    return MovingPlankObservation { x: translation.x, y: translation.y };
}

fn get_simulation_time(query: Query<&Transform, With<Plank>>) -> MovingPlankObservation {
    let translation = query.get_single().unwrap().translation.clone();
    return MovingPlankObservation { x: translation.x, y: translation.y };
}

fn print_environment_observations(query: Query<&Transform, With<Plank>>) {
    println!("Moving plank observations : {:?}", get_observations(query));
}

fn get_score(time_alive: Res<SimulationRunningTeller>) -> u32 {
    let score = time_alive.count.clone();
    return score;
}

fn print_score(time_alive: Res<SimulationRunningTeller>) {
    println!("score is time alive: {}", get_score(time_alive));
}

fn check_if_done(query: Query<&Transform, With<Plank>>, window: Query<&Window>) -> bool {
    let window = window.get_single().unwrap().clone();
    let max_width = window.width() * 0.5;
    let max_height = window.height() * 0.5;
    let translation = query.get_single().unwrap().translation.clone();
    if translation.x.abs() > max_width {
        return true;
    }
    if translation.y.abs() > max_height {
        return true;
    }
    return false;
}


fn print_done_status(query: Query<&Transform, With<Plank>>, window: Query<&Window>) {
    println!("Er done ? : {}", check_if_done(query, window));
}

fn reset_plank(mut query: Query<&mut Transform, With<Plank>>) {
    let mut translation = query.single_mut().translation;
    translation.x = 0.0;
    translation.y = 0.0;
}


