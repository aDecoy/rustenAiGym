use crate::EttHakkState;
use crate::environments::tre_d::spawn_lunar_lander_individ_plugin::INDIVID_DEFAULT_COLOR;
use crate::evolusjon::evolusjon_steg_plugin::{Kjøretilstand, PopulationIsSpawnedMessage};
use crate::evolusjon::hjerne_fenotype::PhenotypeNeuralNetwork;
use crate::evolusjon::phenotype_plugin::{Individ, PlankPhenotype};
use crate::genome::genome_stuff::Genome;
use crate::monitoring::camera_stuff::RENDER_LAYER_ALLE_INDIVIDER;
use crate::monitoring::simulation_teller::SimulationTotalRuntimeRunningTeller;
use avian3d::PhysicsPlugins;
use avian3d::math::Vector;
use avian3d::prelude::*;
use avian3d::prelude::{AngularVelocity as AngularVelocity3d, LinearVelocity as LinearVelocity3d};
use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind::{CYAN_300, PINK_100, RED_500};
use bevy::picking::pointer::PointerInteraction;
use bevy::prelude::KeyCode::{KeyA, KeyD, KeyX, KeyZ};
use bevy::prelude::*;
use std::vec;
// use bevy_rapier2d::na::ComplexField;
// use bevy_rapier2d::prelude::{Collider, CollisionGroups, Group, NoUserData, PhysicsSet, RapierDebugRenderPlugin, RapierPhysicsPlugin, RigidBody, Velocity};

pub struct UserInput3dPlugin;

impl Plugin for UserInput3dPlugin {
    fn build(&self, app: &mut App) {
        app
            // NOTE: Denne og SimulationGenerationTimer henger ikke sammen. Kan endres til å henge sammen, men er ikke gjort akkurat nå
            // Important note: gravity is default on, but only if ExternalForces is used https://github.com/Jondolf/avian/issues/526
            // .insert_resource(Gravity(Vector::NEG_Y * 9.81 * 100.0))
            .add_plugins(MeshPickingPlugin)
            .add_systems(Startup, update_gizmo_config)
            .add_systems(
                PostStartup,
                (add_picking_observers_to_new_individuals.run_if(on_message::<PopulationIsSpawnedMessage>),),
            )
            // .add_systems(Startup, spawn_plank)
            .add_systems(
                Update,
                (
                    draw_mesh_intersections,
                    add_picking_observers_to_new_individuals.run_if(on_message::<PopulationIsSpawnedMessage>),
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

fn update_gizmo_config(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.render_layers = RenderLayers::layer(RENDER_LAYER_ALLE_INDIVIDER);
    println!("oppdatert gizmo render layer")
}

/// A system that draws hit indicators for every pointer.
fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        println!("draw mesh med point {}", &point);

        gizmos.sphere(point, 0.05, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
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

// fn rotate_on_drag(drag: On<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
fn rotate_on_drag3d(drag: On<Pointer<Drag>>, mut angular_velocities: Query<&mut AngularVelocity3d>) {
    // println!("dragging3d rotate");
    let mut angular_velocitiy = angular_velocities.get_mut(drag.event().entity.entity()).unwrap();
    angular_velocitiy.0 += Vector::new(0.1, 0.1, 0.1);
}
fn get_velocity_on_drag3d(drag: On<Pointer<Drag>>, mut velocities: Query<&mut LinearVelocity3d>) {
    // println!("dragging3d velocity");
    let mut velocitiy = velocities.get_mut(drag.event().entity.entity()).unwrap();
    velocitiy.0 += Vector::new(drag.delta.x * 0.02, -drag.delta.y * 0.02, 0.); // todo https://bevy.org/examples/picking/mesh-picking/
}

pub fn add_picking_observers_to_new_individuals(
    mut commands: Commands,
    individ_query: Query<Entity, Added<PlankPhenotype>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let hover_matl = materials.add(Color::from(CYAN_300));
    let default_matl = materials.add(Color::from(INDIVID_DEFAULT_COLOR));
    for individ_entity in individ_query.iter() {
        commands
            .get_entity(individ_entity)
            .unwrap()
            // .observe(rotate_on_drag2d)
            .observe(get_velocity_on_drag3d)
            .observe(rotate_on_drag3d)
            .observe(update_material_on::<Pointer<Over>>(hover_matl.clone()))
            .observe(update_material_on::<Pointer<Out>>(default_matl.clone()));
    }
}

/// Returns an observer that updates the entity's material to the one specified.
fn update_material_on<E: EntityEvent>(new_material: Handle<StandardMaterial>) -> impl Fn(On<E>, Query<&mut MeshMaterial3d<StandardMaterial>>) {
    // An observer closure that captures `new_material`. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // material. Instead, the event type is a generic, and the material is passed in.
    move |event, mut query| {
        if let Ok(mut material) = query.get_mut(event.event_target()) {
            material.0 = new_material.clone();
        }
    }
}
