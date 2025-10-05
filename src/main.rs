use crate::environments::lunar_lander_environment::{LANDING_SITE, LunarLanderEnvironment};
use crate::environments::moving_plank::{
    MovingPlankPlugin, PIXELS_PER_METER, PLANK_HIGHT, PLANK_LENGTH, create_plank_env_falling, create_plank_env_moving_right, create_plank_ext_force_env_falling,
};
use crate::evolusjon::evolusjon_steg_plugin::EvolusjonStegPlugin;
use crate::evolusjon::phenotype_plugin::{IndividFitnessLabelTextTag, PlankPhenotype, add_observers_to_individuals, FenotypePlugin};
use crate::genome::genom_muteringer::mutate_genomes;
use crate::genome::genom_muteringer::{MutasjonerErAktive, lock_mutation_stability};
use crate::genome::genome_stuff::{Genome, InnovationNumberGlobalCounter, new_random_genome};
use crate::genome::genome_stuff::{NodeGene, WeightGene};
use crate::monitoring::camera_stuff::{AllIndividerCameraTag, AllIndividerWindowTag, PopulasjonMenyCameraTag, RENDER_LAYER_POPULASJON_MENY};
use crate::monitoring::camera_stuff::{KnapperMenyCameraTag, MinCameraPlugin, resize_alle_individer_camera};
use crate::monitoring::draw_network::{
    TegnNevraltNettverkPlugin, oppdater_node_tegninger, place_in_focus, remove_drawing_of_network, remove_drawing_of_network_for_previous_individ_in_focus,
};
use crate::monitoring::hyllerepresentasjon::HyllerepresentasjonPlugin;
use crate::monitoring::in_focus_stuff::{InFocusPlugin, IndividInFocus, IndividInFocusСhangedEvent};
use crate::monitoring::knapp_meny::KnappMenyPlugin;
use crate::monitoring::simulation_teller::{SimulationGenerationTimer, SimulationRunningTellerPlugin};
use crate::populasjon_handlinger::population_sammenligninger::{ElitePlugin, EliteTag};
use avian2d::math::{AdjustPrecision, Vector};
use avian2d::prelude::*;
use bevy::asset::AsyncWriteExt;
use bevy::color::palettes::basic::{PURPLE, RED};
use bevy::color::palettes::css::GREEN;
use bevy::color::palettes::tailwind::{CYAN_300, RED_300, RED_800};
use bevy::ecs::observer::TriggerTargets;
use bevy::ecs::query::QueryIter;
use bevy::prelude::KeyCode::{KeyE, KeyK, KeyM, KeyN, KeyP, KeyR, KeyT};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::view::RenderLayers;
use bevy_egui::UiRenderOrder;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::egui::emath::Numeric;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use lazy_static::lazy_static;
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use rand::{Rng, thread_rng};
use std::cmp::{Ordering, PartialEq, max, min};
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::sync::Arc;
use std::vec::Vec;

mod environments;
mod evolusjon;
mod genome;
mod monitoring;
mod populasjon_handlinger;

struct Environment {
    app: App,
    value: usize,
}

fn main() {
    println!("Starting up AI GYM");

    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        // .add_plugins(MinimalPlugins)
    
        .add_plugins((
            EguiPlugin {
                enable_multipass_for_primary_context: true,
                ui_render_order: UiRenderOrder::EguiAboveBevyUi,
            },
            WorldInspectorPlugin::new(),
        ))
        .add_plugins(MeshPickingPlugin)
        .insert_state(EttHakkState::DISABLED)
        .insert_resource(InnovationNumberGlobalCounter { count: 0 })
        .add_systems(
            Update,
            (
                endre_kjøretilstand_ved_input,
            )
        )
        // Environment spesific : Later changed
        .add_plugins(MovingPlankPlugin)
        .add_plugins(SimulationRunningTellerPlugin)
        .add_plugins(MinCameraPlugin { debug: false })
        .add_plugins(InFocusPlugin)
        .add_plugins(ElitePlugin)
        .add_plugins(HyllerepresentasjonPlugin)
        .add_plugins(KnappMenyPlugin)
        .add_plugins(TegnNevraltNettverkPlugin)
        .add_plugins(EvolusjonStegPlugin)
        .add_plugins(LunarLanderEnvironment)
        .add_plugins(FenotypePlugin);

    app.run();
}

// fn every_time() -> impl Condition<()> {
//     IntoSystem::into_system(|mut flag: Local<bool>| {
//         *flag = true;
//         *flag
//     })
// }

fn every_time_if_stop_on_right_window() -> impl Condition<()> {
    IntoSystem::into_system(|mut flag: Local<bool>| {
        *flag = match ACTIVE_ENVIROMENT {
            EnvValg::Høyre | EnvValg::Fall | EnvValg::FallVelocityHøyre | EnvValg::FallExternalForcesHøyre => true,
            _ => false,
        };
        *flag
    })
}

//////////////

#[derive(Resource, Default, Debug)]
struct SimulationTimer {
    count: i32,
}

/////////////////// create/kill/develop  new individuals

static START_POPULATION_SIZE: i32 = 3;
static ANT_INDIVIDER_SOM_OVERLEVER_HVER_GENERASJON: i32 = 5;
static ANT_PARENTS_HVER_GENERASJON: usize = 3;

// todo. legg på label på input og output i tegninger, slik at det er enkelt å se hva som er x og y
// todo , også legg på elite ID på tenging, slik at vi ser at den er den samme hele tiden.

fn spawn_start_population(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut innovation_number_global_counter: ResMut<InnovationNumberGlobalCounter>,
) {
    for n in 0i32..START_POPULATION_SIZE {
        // for n in 0i32..1 {
        spawn_a_random_new_individual(&mut commands, &mut meshes, &mut materials, &mut innovation_number_global_counter, n);
    }
}

// fn spawn_a_new_random_individual_2(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<ColorMaterial>>) {
//     let n: i32;
//
//     let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(PLANK_LENGTH, PLANK_HIGHT));
//
//     let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE));
//
//     let mut genome = new_random_genome(2, 2);
//     // let genome_entity = commands.spawn(genome).id(); // todo kanksje det samme om inne i en bundle eller direkte?
//     // let genome2 :Genome = genome_entity.get::<Genome>().unwrap();
//
//     // println!("Har jeg klart å lage en genome fra entity = : {}", genome2.allowed_to_change);
//
//     match ACTIVE_ENVIROMENT {
//         EnvValg::Høyre => commands.spawn(create_plank_env_moving_right(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + n as f32 * 50.0, z: 1.0 }, new_random_genome(2, 2))),
//         EnvValg::Fall | EnvValg::FallVelocityHøyre => commands.spawn(create_plank_env_falling(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + (n as f32 * 15.0), z: 1.0 }, new_random_genome(2, 2))),
//         EnvValg::FallExternalForcesHøyre | EnvValg::Homing => { commands.spawn(create_plank_ext_force_env_falling(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + 3.3 * 50.0, z: 1.0 }, new_random_genome(2, 2))) }
//     };
// }


fn spawn_a_random_new_individual(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    innovation_number_global_counter: &mut ResMut<InnovationNumberGlobalCounter>,
    n: i32,
) {
    let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(PLANK_LENGTH, PLANK_HIGHT));
    let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE));

    let hover_matl = materials.add(Color::from(CYAN_300));
    // println!("Har jeg klart å lage en genome fra entity = : {}", genome2.allowed_to_change);
    // let text_style = TextStyle {
    //     font_size: 20.0,
    //     color: Color::WHITE,
    //     ..default()
    // };
    let genome = match ACTIVE_ENVIROMENT {
        EnvValg::HomingGroudY => new_random_genome(1, 1, innovation_number_global_counter),
        _ => new_random_genome(2, 2, innovation_number_global_counter),
    };

    match ACTIVE_ENVIROMENT {
        EnvValg::Høyre => commands.spawn(create_plank_env_moving_right(
            material_handle.clone(),
            rectangle_mesh_handle.into(),
            Vec3 {
                x: 0.0,
                y: -150.0 + n as f32 * 50.0,
                z: 1.0,
            },
            genome,
        )),
        EnvValg::Fall | EnvValg::FallVelocityHøyre => commands.spawn(create_plank_env_falling(
            material_handle.clone(),
            rectangle_mesh_handle.into(),
            Vec3 {
                x: 0.0,
                y: -150.0 + (n as f32 * 15.0),
                z: 1.0,
            },
            genome,
        )),
        EnvValg::FallExternalForcesHøyre | EnvValg::Homing | EnvValg::HomingGroud | EnvValg::HomingGroudY => commands.spawn(create_plank_ext_force_env_falling(
            material_handle.clone(),
            rectangle_mesh_handle.into(),
            Vec3 { x: 30.0, y: 100.0, z: 1.0 },
            genome,
        )),
    }
    .with_children(|builder| {
        builder.spawn((
            Text2d::new("translation"),
            TextLayout::new_with_justify(JustifyText::Center),
            Transform::from_xyz(0.0, 0.0, 2.0),
            IndividFitnessLabelTextTag,
            RenderLayers::layer(1),
        ));
    });
}

/// Returns an observer that updates the entity's material to the one specified.
fn update_material_on<E>(new_material: Handle<ColorMaterial>) -> impl Fn(Trigger<E>, Query<&mut MeshMaterial2d<ColorMaterial>>) {
    // An observer closure that captures `new_material`. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // material. Instead, the event type is a generic, and the material is passed in.
    move |trigger, mut query| {
        if let Ok(mut material) = query.get_mut(trigger.target().entity()) {
            material.0 = new_material.clone();
        }
    }
}

// #[derive(Clone)]
// struct PhentypeGenome<'lifetime_a> {
//     phenotype: &'lifetime_a PlankPhenotype<'lifetime_a>,
//     genome: &'lifetime_a Genome<'lifetime_a>,
//     entity_index: u32,
//     entity_bevy_generation: u32,
// }

#[derive(PartialEq, Resource, Eq, Hash)]
enum EnvValg {
    Høyre,
    // has velocity
    Fall,
    // uses velocity
    FallVelocityHøyre,
    // uses impulse/force
    FallExternalForcesHøyre,
    // Aiming at a target in the air
    Homing,
    // Aiming at a target on the ground
    HomingGroud,
    // Aiming at a target on the ground, only uses 1 input and 1 output (y)
    HomingGroudY,
}

static ACTIVE_ENVIROMENT: EnvValg = EnvValg::HomingGroud;

// static LANDING_SITE: Vec2 = Vec2 { x: 100.0, y: -100.0 };

// state control

fn set_to_kjørende_state(mut next_state: ResMut<NextState<Kjøretilstand>>) {
    next_state.set(Kjøretilstand::Kjørende);
}

fn endre_kjøretilstand_ved_input(
    mut next_state: ResMut<NextState<Kjøretilstand>>,
    mut next_ett_hakk_state: ResMut<NextState<EttHakkState>>,
    user_input: Res<ButtonInput<KeyCode>>,
) {
    if user_input.pressed(KeyP) {
        next_state.set(Kjøretilstand::Pause);
        next_ett_hakk_state.set(EttHakkState::DISABLED);
    }
    if user_input.pressed(KeyE) {
        next_state.set(Kjøretilstand::Pause);
        next_ett_hakk_state.set(EttHakkState::VENTER_PÅ_INPUT);
    }
    if user_input.pressed(KeyK) {
        next_state.set(Kjøretilstand::Kjørende);
        next_ett_hakk_state.set(EttHakkState::DISABLED);
    }
}

// fn reset_to_star_pos(mut query: Query<(&mut Transform, &mut crate::PlankPhenotype, &mut Velocity), ( With<crate::PlankPhenotype>)>) {

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum Kjøretilstand {
    #[default]
    Pause,
    Kjørende,

    EvolutionOverhead,
    // FitnessEvaluation,
    // Mutation,
    // ParentSelection,
    // SurvivorSelection,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum EttHakkState {
    #[default]
    DISABLED,
    VENTER_PÅ_INPUT,
    KJØRER_ETT_HAKK,
}

fn print_pois_velocity_and_force(mut query: Query<(&Transform, &PlankPhenotype, &LinearVelocity, &ExternalForce), (With<crate::PlankPhenotype>)>) {
    for (translation, plank, linvel, external_force) in query.iter_mut() {
        println!("translation {:#?}", translation);
        println!("linvel {:#?}", linvel);
        println!("external_force {:#?}", external_force);
        println!("----------------------------")
    }
}

// not used abstraction ideas for/from ai gym

/////////////////////////////////
//
// All environments will share the env traint, and this gives us a type safe way of interacting with a variation of different environments.
// (Basically an interface of what is public for the outside)
pub trait Env {
    /// The type of action supported.
    type Action;

    /// The type of the observation produced after an action has been applied.
    type Observation;

    /// The type of the metadata object produced by acting on the environment.
    type Info;

    /// The type of the object produced when an environment is reset.
    type ResetInfo;

    /// Acts on an environment using the given action, producing a reward.
    fn step(&mut self, action: Self::Action);
    // fn step(&mut self, action: Self::Action) -> StepFeedback;

    /// Resets the environment to a initial random state.
    fn reset(
        &mut self,
        seed: Option<u64>,
        return_info: bool,
        // options: Option<BoxR<Self::Observation>>,
    ) -> (Self::Observation, Option<Self::ResetInfo>);

    /// Produces the renders, if any, associated with the given mode.
    /// Sets the render mode, and bevy will use that state to deterimine if it should render.
    fn render(&mut self, mode: RenderMode); // -> Renders;

    /// Closes any open resources associated with the internal rendering service.
    fn close(&mut self);
}

/// A collection of various formats describing the type of content produced during a render.
#[derive(Debug, Clone, Copy)]
pub enum RenderMode {
    /// Indicates that that renderer should be done through the terminal or an external display.
    Human,
    /// Indicates that renderer should be skipped.
    None,
}

/// The return type of [`Env::step()`].
pub struct StepFeedback<E: Env> {
    /// The observation of the environment after taking an action.
    pub observation: E::Observation,
    /// The reward after taking an action.
    // pub reward: E::Reward,
    pub reward: f32,
    /// Indication that the agent has reached a terminal state after taking an action, as defined by the task formulation.
    /// If true, it is expected that [`Env::reset()`] is called to restart the environment.
    pub terminated: bool,
    /// Indication that the agent has reached a truncation condition after taking an action, which is outside the scope of the task formulation.
    /// If true, [`Env::reset()`] should be called to restart the environment.
    pub truncated: bool,
    /// Additional information from the environment after taking an action.
    pub info: E::Info,
}
// todo envProperties

/// Defines the bounds for the reward value that can be observed.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct RewardRange {
    /// The smallest possible reward that can be observed.
    lower_bound: f32,
    /// The largest possible reward that can be observed.
    upper_bound: f32,
}
