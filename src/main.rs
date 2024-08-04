use bevy::color::palettes::basic::PURPLE;
use bevy::prelude::*;
use bevy::prelude::KeyCode::{KeyE, KeyK, KeyP};
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy_inspector_egui::egui::emath::Numeric;
use bevy_rapier2d::parry::transformation::utils::transform;
use rand::random;
use crate::environments::moving_plank::{create_plank, MovingPlankPlugin, mutate_planks, Plank};
use crate::environments::simulation_teller::SimulationRunningTellerPlugin;

mod environments;

struct Environment {
    app: App,
    value: usize,
}

fn main() {
    println!("Starting up AI GYM");

    let mut app = App::new();
    app
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::DX12),
                ..default()
            }),
            synchronous_pipeline_compilation: false,
        }))
        // .add_plugins(DefaultPlugins)
        .insert_state(Kjøretilstand::Kjørende)
        .insert_state(EttHakkState::DISABLED)
        .add_systems(Startup, (
            setup_camera,
            spawn_x_individuals
        ))
        .add_systems(Update, (
            endre_kjøretilstand_ved_input,
            agent_action.run_if(in_state(Kjøretilstand::Kjørende)),
            mutate_planks,
        ))
        // Environment spesific : Later changed
        .add_plugins(MovingPlankPlugin)
        .add_plugins(SimulationRunningTellerPlugin);

    app.run();
}
//
// #[derive(Component)]
// pub struct Individual {
//     phenotype: f32,
//     plank: Plank,
//     score: f32,
//     obseravations: f32,
// }

fn spawn_x_individuals(mut commands: Commands,
                       mut meshes: ResMut<Assets<Mesh>>,
                       mut materials: ResMut<Assets<ColorMaterial>>, ) {
    for n in 0i32..10 {
        let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::default());
        let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE));
        commands.spawn(
            create_plank(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + n as f32 * 50.0, z: 1.0 })
        );
    }
}

// fn agent_action(query: Query<Transform, With<Individual>>) {
fn agent_action(mut query: Query<(&mut Transform, &mut Plank), ( With<Plank>)>) {
    for (mut individual, mut plank) in query.iter_mut() {
        individual.translation.x += random::<f32>()* plank.phenotype ;
        plank.score = individual.translation.x.clone();
        plank.obseravations = individual.translation.x.clone();
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum Kjøretilstand {
    #[default]
    Pause,
    Kjørende,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum EttHakkState {
    #[default]
    DISABLED,
    VENTER_PÅ_INPUT,
    KJØRER_ETT_HAKK,
}


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



