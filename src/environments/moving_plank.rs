use crate::environments::simulation_teller::SimulationTotalRuntimeRunningTeller;
use crate::{create_phenotype_layers, EttHakkState, Kjøretilstand, PhenotypeNeuralNetwork, PlankPhenotype};
use avian2d::prelude::*;
use avian2d::PhysicsPlugins;
use bevy::prelude::KeyCode::{KeyA, KeyD, KeyX, KeyZ};
use bevy::prelude::*;
use std::vec;
use crate::environments::GenomeStuff::Genome;
// use bevy_rapier2d::na::ComplexField;
// use bevy_rapier2d::prelude::{Collider, CollisionGroups, Group, NoUserData, PhysicsSet, RapierDebugRenderPlugin, RapierPhysicsPlugin, RigidBody, Velocity};

pub struct MovingPlankPlugin;

impl MovingPlankPlugin {}

pub const PIXELS_PER_METER: f32 = 10.0;
pub const PHYSICS_RELATIVE_SPEED: f32 = 20.0;

impl Plugin for MovingPlankPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER)
            //     // To spesify my won runing conditions, so that i can pause the sim, or run it one timestep at a time
            //     .with_default_system_setup(false))
            // .add_plugins(RapierDebugRenderPlugin::default())
            // .add_systems(Update, (
            //     RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend).in_set(PhysicsSet::SyncBackend),
            //     RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::StepSimulation).in_set(PhysicsSet::StepSimulation),
            //     RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback).in_set(PhysicsSet::Writeback),
            // ).chain() // overasknede viktig. uten den så lagger ting
            //     .run_if(in_state(Kjøretilstand::Kjørende)),
            // )
            .add_plugins((PhysicsPlugins::default().with_length_unit(PIXELS_PER_METER),))
            .insert_resource(Time::<Physics>::default().with_relative_speed(PHYSICS_RELATIVE_SPEED)) // NOTE: Denne og SimulationGenerationTimer henger ikke sammen. Kan endres til å henge sammen, men er ikke gjort akkurat nå

            // Important note: gravity is default on, but only if ExternalForces is used https://github.com/Jondolf/avian/issues/526
            // .insert_resource(Gravity(Vector::NEG_Y * 9.81 * 100.0))
            .insert_resource(Gravity::ZERO)

            // .add_systems(Startup, spawn_plank)
            .add_systems(Update, (
                (set_physics_time_to_paused_or_unpaused).run_if(state_changed::<Kjøretilstand>),
                (
                    move_plank,
                    impulse_plank,
                    // print_done_status,
                    // print_score,
                    // print_environment_observations
                ).run_if(
                    in_state(Kjøretilstand::Kjørende)
                ),
                (set_ett_hakk_til_kjør_ett_hakk_if_input).run_if(in_state(EttHakkState::VENTER_PÅ_INPUT)),
                (set_ett_hakk_til_vent_på_input).run_if(in_state(EttHakkState::KJØRER_ETT_HAKK)),
            ).chain());
    }
}


/// Defines the state found in the cart pole environment.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct MovingPlankObservation {
    x: f32,
    y: f32,
}


const PLANK_STARTING_POSITION: Vec3 = Vec3 { x: 0.0, y: -150.0, z: 0.0 };
pub const PLANK_LENGTH: f32 = 9.0 * PIXELS_PER_METER;  // in meters
pub const PLANK_HIGHT: f32 = 3.0 * PIXELS_PER_METER; // in meters
const PLANK_POSITION_CHANGE_MOVEMENT_SPEED: f32 = 1133.0;
const PLANK_POSITION_VELOCITY_MOVEMENT_SPEED: f32 = 1133.0;

const PLANK_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

fn set_physics_time_to_paused_or_unpaused(
    kjøretistand_state: Res<State<Kjøretilstand>>,
    mut physics_time: ResMut<Time<Physics>>, ) {
    match kjøretistand_state.get() {
        Kjøretilstand::Pause => physics_time.pause(),
        Kjøretilstand::Kjørende => physics_time.unpause(),
        Kjøretilstand::EvolutionOverhead => physics_time.unpause(),
    }
}

pub fn create_plank_env_moving_right(material_handle: Handle<ColorMaterial>, mesh2d_handle: Handle<Mesh>, start_position: Vec3, genome: Genome) -> (Mesh2d, Transform, MeshMaterial2d<ColorMaterial>, PlankPhenotype, Genome, Collider, MovingPlankObservation, LinearVelocity) {
    (
        Mesh2d(mesh2d_handle),
        Transform::from_translation(start_position),
        // .with_scale(Vec2 { x: PLANK_LENGTH, y: PLANK_HIGHT }.extend(1.)),
        MeshMaterial2d(material_handle),
        PlankPhenotype {
            score: 0.0,
            obseravations: vec!(0.0, 0.0),
            // phenotype_layers: create_phenotype_layers(genome.clone()),
            phenotype_layers: PhenotypeNeuralNetwork::new(&genome),
            // genotype: genome_entity,
        }, // alt 1
        genome,
        // Collider::cuboid(0.5, 0.5),
        Collider::rectangle(0.5, 0.5),
        MovingPlankObservation { x: 0.0, y: 0.0 }, // alt 2,
        // RigidBody::Dynamic,
        // individ, // taged so we can use queryies to make evolutionary choises about the individual based on preformance of the phenotype
        // Velocity {
        //     // linvel: Vec2::new(100.0, 2.0),
        //     linvel: Vec2::new(0.0, 0.0),
        //     angvel: 0.0,
        // },
        LinearVelocity {
            0: Vec2::new(0.0, 0.0),
        },
    )
}

pub fn create_plank_env_falling(material_handle: Handle<ColorMaterial>, mesh2d_handle: Handle<Mesh>, start_position: Vec3, genome: Genome) -> (Mesh2d, Transform, MeshMaterial2d<ColorMaterial>, PlankPhenotype, Genome, Collider, RigidBody, CollisionLayers, LinearVelocity) {
    (
        Mesh2d(mesh2d_handle),
        Transform::from_translation(start_position)
            .with_scale(Vec2 {
                x: PLANK_LENGTH,
                y: PLANK_HIGHT,
            }.extend(1.)),
        MeshMaterial2d(material_handle),
        PlankPhenotype {
            score: 0.0,
            obseravations: vec!(0.0, 0.0),
            // phenotype_layers:  create_phenotype_layers(genome.clone()),
            phenotype_layers: PhenotypeNeuralNetwork::new(&genome),
            // genotype: genome_entity,
        }, // alt 1
        genome,
        Collider::rectangle(1.0, 1.0),
        // Collider::cuboid(0.5, 0.5),
        RigidBody::Dynamic,
        // MovingPlankObservation { x: 0.0, y: 0.0 }, // alt 2,
        // CollisionGroups::new(
        //     // almost looked like it runs slower with less collisions?
        //     // Kan være at det bare er mer ground kontakt, siden alle ikke hvilker på en blokk som er eneste som rører bakken
        //     Group::GROUP_1,
        //     if INDIVIDUALS_COLLIDE_IN_SIMULATION { Group::GROUP_1 } else {
        //         Group::GROUP_2
        //     },
        // ),
        CollisionLayers::new(0b0001, 0b0010),
        // Velocity {
        //     // linvel: Vec2::new(100.0, 2.0),
        //     linvel: Vec2::new(0.0, 0.0),
        //     angvel: 0.0,
        // },
        LinearVelocity {
            0: Vec2::new(0.0, 0.0),
        },
    )
}
pub fn create_plank_ext_force_env_falling(material_handle: Handle<ColorMaterial>, mesh2d_handle: Handle<Mesh>, start_position: Vec3, genome: Genome) -> (Mesh2d, MeshMaterial2d<ColorMaterial>, Transform, PlankPhenotype, Genome, Collider, RigidBody, CollisionLayers, LinearVelocity, ExternalForce, TextLayout) {
    // let text_style = TextStyle {
    //     font_size: 30.0,
    //     color: Color::WHITE,
    //     ..default()
    // };
    (
        Mesh2d(mesh2d_handle),
        MeshMaterial2d(material_handle),
        Transform::from_translation(start_position).with_scale(Vec2 { x: 1.0, y: 1.0 }.extend(1.)),
        PlankPhenotype {
            score: 0.0,
            obseravations: vec!(0.0, 0.0),
            // phenotype_layers: create_phenotype_layers(genome.clone()),
            phenotype_layers: PhenotypeNeuralNetwork::new(&genome),
            // genotype: genome_entity,
        }, // alt 1
        genome,
        // Collider::rectangle(1.0, 1.0),
        Collider::rectangle(PLANK_LENGTH, PLANK_HIGHT),
        RigidBody::Dynamic,
        CollisionLayers::new(0b0001, 0b0010),
        LinearVelocity {
            0: Vec2::new(0.0, 0.0),
        },
        // ExternalForce { force: Vec2::new(0.0, 0.0), persistent: false , ..default()} ,
        ExternalForce::new(Vec2::X).with_persistence(false),
        TextLayout::new_with_justify(JustifyText::Center),
    )
}

static INDIVIDUALS_COLLIDE_IN_SIMULATION: bool = false;

fn move_plank(mut query: Query<&mut Transform, With<PlankPhenotype>>,
              keyboard_input: Res<ButtonInput<KeyCode>>,
              time: Res<Time>,
) {
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

fn impulse_plank(
    mut query: Query<
        &mut LinearVelocity, With<PlankPhenotype>>,
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

fn set_ett_hakk_til_vent_på_input(mut next_state: ResMut<NextState<EttHakkState>>,
                                  mut next_kjøretistand_state: ResMut<NextState<Kjøretilstand>>,
) {
    next_state.set(EttHakkState::VENTER_PÅ_INPUT);
    next_kjøretistand_state.set(Kjøretilstand::Pause);
}

fn set_ett_hakk_til_kjør_ett_hakk_if_input(keyboard_input: Res<ButtonInput<KeyCode>>,
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
