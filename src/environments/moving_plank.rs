use std::cmp::Ordering;
use bevy::color::palettes::css::PURPLE;
use bevy::prelude::*;
use bevy::prelude::KeyCode::{KeyA, KeyD};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_rapier2d::na::ComplexField;
use bevy_rapier2d::prelude::{Collider, NoUserData, RapierDebugRenderPlugin, RapierPhysicsPlugin, RigidBody};
use rand::random;
use crate::environments::simulation_teller::SimulationRunningTeller;
use crate::{EttHakkState, Genome, Individ, Kjøretilstand, new_random_genome, PlankPhenotype};

pub struct MovingPlankPlugin;

impl MovingPlankPlugin {}

impl Plugin for MovingPlankPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugins(RapierDebugRenderPlugin::default())
            // .add_systems(Startup, spawn_plank)
            .add_systems(Update, (
                (
                    move_plank,
                    // print_done_status,
                    // print_score,
                    // print_environment_observations
                ).run_if(
                    in_state(Kjøretilstand::Kjørende)
                ),
                (set_ett_hakk_til_kjør_ett_hakk_if_input).run_if(in_state(EttHakkState::VENTER_PÅ_INPUT)),
                (set_ett_hakk_til_vent_på_input).run_if(in_state(EttHakkState::KJØRER_ETT_HAKK)),
            ).chain())
        ;
    }
}


/// Defines the state found in the cart pole environment.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct MovingPlankObservation {
    x: f32,
    y: f32,
}


const PLANK_STARTING_POSITION: Vec3 = Vec3 { x: 0.0, y: -150.0, z: 1.0 };
const PLANK_LENGTH: f32 = 95.;
const PLANK_HIGHT: f32 = 30.;
const PLANK_MOVEMENT_SPEED: f32 = 3.0;

const PLANK_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

// fn spawn_plank(mut commands: Commands,
//                mut meshes: ResMut<Assets<Mesh>>,
//                mut materials: ResMut<Assets<ColorMaterial>>,
//                // ) -> Entity {
// ) {
//     let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::default());
//     let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE));
//     let _id = commands.spawn(create_plank(material_handle, rectangle_mesh_handle.into()),
//     ).id();
//     // return id;
// }

pub fn create_plank(material_handle: Handle<ColorMaterial>, mesh2d_handle: Mesh2dHandle, start_position: Vec3, genome: Genome) -> (MaterialMesh2dBundle<ColorMaterial>, PlankPhenotype, Collider, MovingPlankObservation) {
    (
        MaterialMesh2dBundle {
            mesh: mesh2d_handle,
            transform: Transform::from_translation(start_position)
                .with_scale(Vec2 { x: PLANK_LENGTH, y: PLANK_HIGHT }.extend(1.)),

            material: material_handle,
            ..default()
        },
        PlankPhenotype {
            score: 0.0,
            obseravations: 0.0,
            // phenotype: phenotype,
            genotype: genome,
        }, // alt 1
        // RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5),
        MovingPlankObservation { x: 0.0, y: 0.0 }, // alt 2,
        // individ, // taged so we can use queryies to make evolutionary choises about the individual based on preformance of the phenotype
    )
}

// pub fn mutate_planks(mut query: Query<&mut PlankPhenotype>) {
//     for mut plank in query.iter_mut() {
//         let old_phenotype = plank.phenotype.clone();
//         plank.phenotype += random::<f32>() * 2.0 - 1.0;
//         println!("Changed phenotype from {} to {}", old_phenotype, plank.phenotype)
//     }
// }

fn move_plank(mut query: Query<&mut Transform, With<PlankPhenotype>>,
              keyboard_input: Res<ButtonInput<KeyCode>>,
              time: Res<Time>,
) {
    let mut delta_x = 0.0;

    if keyboard_input.pressed(KeyA) {
        delta_x -= PLANK_MOVEMENT_SPEED;
    }
    if keyboard_input.pressed(KeyD) {
        delta_x += PLANK_MOVEMENT_SPEED;
    }
    // let mut transform = query.single_mut();
    if delta_x != 0.0 {
        for mut transform in query.iter_mut() {
            transform.translation.x += delta_x * time.delta_seconds();
        }
    }
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

fn get_observations(transform: Transform) -> MovingPlankObservation {
    // let translation = query.get_single().unwrap().translation.clone();
    return MovingPlankObservation { x: transform.translation.x, y: transform.translation.y };
}

fn get_simulation_time(query: Query<&Transform, With<PlankPhenotype>>) -> MovingPlankObservation {
    let translation = query.get_single().unwrap().translation.clone();
    return MovingPlankObservation { x: translation.x, y: translation.y };
}

fn print_environment_observations(query: Query<&Transform, With<PlankPhenotype>>) {
    for transform in query.iter() {
        println!("Moving plank observations : {:?}", get_observations(transform.clone()));
    }
}

fn get_score(time_alive: Res<SimulationRunningTeller>) -> u32 {
    let score = time_alive.count.clone();
    return score;
}

fn print_score(time_alive: Res<SimulationRunningTeller>) {
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
    let window = window.get_single().unwrap().clone();

    for transform in query.iter() {
        println!("Er done ? : {}", check_if_done(transform.clone(), window.clone()));
    }
    println!("-------------------------");
}

fn reset_plank(mut query: Query<&mut Transform, With<PlankPhenotype>>) {
    let mut translation = query.single_mut().translation;
    translation.x = 0.0;
    translation.y = 0.0;
}
