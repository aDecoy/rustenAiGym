use crate::environments::gammelt_2d::lunar_lander_environment2d::{LANDING_SITE, LunarLanderEnvironment2d};
use crate::environments::gammelt_2d::moving_plank_2d::{MovingPlankPlugin2d, PIXELS_PER_METER, PLANK_HIGHT, PLANK_LENGTH};
use crate::environments::tre_d::lunar_lander_environment_3d::LunarLanderEnvironment3d;
use crate::environments::tre_d::lunar_lander_individual_behavior::LunarLanderIndividBehaviors;
use crate::evolusjon::evolusjon_steg_plugin::{EvolusjonStegPlugin, Kjøretilstand};
use crate::evolusjon::phenotype_plugin::{FenotypePlugin, IndividFitnessLabelTextTag, PlankPhenotype, add_observers_to_individuals};
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
use bevy::ecs::query::QueryIter;
use bevy::prelude::KeyCode::{KeyE, KeyK, KeyM, KeyN, KeyP, KeyR, KeyT};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
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
                bindless_mode_array_size: None,
            },
            WorldInspectorPlugin::new(),
        ))
        .add_plugins(MeshPickingPlugin)
        .insert_state(EttHakkState::DISABLED)
        .insert_resource(InnovationNumberGlobalCounter { count: 0 })
        .add_systems(Update, (endre_kjøretilstand_ved_input,))
        // Environment spesific : Later changed
        .add_plugins(MovingPlankPlugin2d)
        .add_plugins(SimulationRunningTellerPlugin)
        .add_plugins(MinCameraPlugin { debug: false })
        .add_plugins(InFocusPlugin)
        .add_plugins(ElitePlugin)
        .add_plugins(HyllerepresentasjonPlugin)
        .add_plugins(KnappMenyPlugin)
        .add_plugins(TegnNevraltNettverkPlugin)
        // .add_plugins(EvolusjonStegPlugin{ environmentSpesificIndividStuff :  evolusjon::evolusjon_steg_plugin::PossibleBehaviorSets::LUNAR_LANDER_3D(LunarLanderIndividBehaviors) })
        .add_plugins(EvolusjonStegPlugin {
            environmentSpesificIndividStuff: evolusjon::evolusjon_steg_plugin::PossibleBehaviorSets::LUNAR_LANDER_3D {
                oppførsel: LunarLanderIndividBehaviors,
            },
        })
        // .add_plugins(LunarLanderEnvironment2d)
        .add_plugins(LunarLanderEnvironment3d)
        .add_plugins(FenotypePlugin);

    app.run();
}

/////////////////// create/kill/develop  new individuals

static START_POPULATION_SIZE: i32 = 3;
static ANT_INDIVIDER_SOM_OVERLEVER_HVER_GENERASJON: i32 = 5;
static ANT_PARENTS_HVER_GENERASJON: usize = 3;

// todo. legg på label på input og output i tegninger, slik at det er enkelt å se hva som er x og y
// todo , også legg på elite ID på tenging, slik at vi ser at den er den samme hele tiden.

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
enum EttHakkState {
    #[default]
    DISABLED,
    VENTER_PÅ_INPUT,
    KJØRER_ETT_HAKK,
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
