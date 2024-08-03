use bevy::color::palettes::css::PURPLE;
use bevy::prelude::*;
use bevy::prelude::KeyCode::{KeyA, KeyD};
use bevy::sprite::MaterialMesh2dBundle;
use bevy_rapier2d::na::ComplexField;
use bevy_rapier2d::prelude::{Collider, NoUserData, RapierDebugRenderPlugin, RapierPhysicsPlugin, RigidBody};

use crate::environments::simulation_teller::SimulationRunningTeller;
use crate::{EttHakkState, Kjøretilstand};

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
                    print_done_status,
                    print_score,
                    print_environment_observations
                ).run_if(
                    in_state(Kjøretilstand::Kjørende)
                ),
                (set_ett_hakk_til_kjør_ett_hakk_if_input).run_if(in_state(EttHakkState::VENTER_PÅ_INPUT)),
                (set_ett_hakk_til_vent_på_input).run_if(in_state(EttHakkState::KJØRER_ETT_HAKK)),
            ).chain())
        ;
    }
}


#[derive(Component)]
struct Plank;

/// Defines the state found in the cart pole environment.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
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
               // ) -> Entity {
) {
    let _id = commands.spawn((
                                MaterialMesh2dBundle {
                                    mesh: meshes.add(Rectangle::default()).into(),
                                    transform: Transform::from_translation(PLANK_STARTING_POSITION)
                                        .with_scale(Vec2 { x: PLANK_LENGTH, y: PLANK_HIGHT }.extend(1.)),

                                    material: materials.add(Color::from(PURPLE)),
                                    ..default()
                                },
                                Plank,
                                // RigidBody::Dynamic,
                                Collider::cuboid(0.5, 0.5),
                                MovingPlankObservation { x: 0.0, y: 0.0 }
                            ), ).id();
    // return id;
}

fn move_plank(mut query: Query<&mut Transform, With<Plank>>,
              keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut delta_x = 0.0;
    if keyboard_input.pressed(KeyA) {
        delta_x -= PLANK_MOVEMENT_SPEED;
    }
    if keyboard_input.pressed(KeyD) {
        delta_x += PLANK_MOVEMENT_SPEED;
    }
    let mut transform = query.single_mut();
    transform.translation.x += delta_x;
}

fn set_ett_hakk_til_vent_på_input(mut next_state: ResMut<NextState<EttHakkState>>,
                                  mut next_kjøretistand_state: ResMut<NextState<Kjøretilstand>>,

) {
    next_state.set(EttHakkState::VENTER_PÅ_INPUT);
    next_kjøretistand_state.set(Kjøretilstand::Pause);

}

fn set_ett_hakk_til_kjør_ett_hakk_if_input(keyboard_input: Res<ButtonInput<KeyCode>>,
                                           mut next_state: ResMut<NextState<EttHakkState>>,
                                           mut next_kjøretistand_state: ResMut<NextState<Kjøretilstand>>,
) {
    let mut input_exist = false;
    if keyboard_input.pressed(KeyA) {
        input_exist = true;
    }
    if keyboard_input.pressed(KeyD) {
        input_exist = true;
    }
    if input_exist {
        next_state.set(EttHakkState::KJØRER_ETT_HAKK);
        next_kjøretistand_state.set(Kjøretilstand::Kjørende);
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
