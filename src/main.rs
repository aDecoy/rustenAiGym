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
        // .add_plugins(DefaultPlugins.set(RenderPlugin {
        //     render_creation: RenderCreation::Automatic(WgpuSettings {
        //         backends: Some(Backends::DX12),
        //         ..default()
        //     }),
        //     synchronous_pipeline_compilation: false,
        // }))
        .add_plugins(DefaultPlugins)
        // .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
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

/////////////////////////////////
//
// // All environments will share the env traint, and this gives us a type safe way of interacting with a variation of different environments.
// // (Basically an interface of what is public for the outside)
// pub trait Env{
//     /// The type of action supported.
//     type Action;
//
//     /// The type of the observation produced after an action has been applied.
//     type Observation;
//
//     /// The type of the metadata object produced by acting on the environment.
//     type Info;
//
//     /// The type of the object produced when an environment is reset.
//     type ResetInfo;
//
//     /// Acts on an environment using the given action, producing a reward.
//     fn step(&mut self, action: Self::Action) -> StepFeedback<&Self>;
//
//     /// Resets the environment to a initial random state.
//     fn reset(
//         &mut self,
//         seed: Option<u64>,
//         return_info: bool,
//         // options: Option<BoxR<Self::Observation>>,
//     ) -> (Self::Observation, Option<Self::ResetInfo>);
//
//     /// Produces the renders, if any, associated with the given mode.
//     /// Sets the render mode, and bevy will use that state to deterimine if it should render.
//     fn render(&mut self, mode: RenderMode);// -> Renders;
//
//     /// Closes any open resources associated with the internal rendering service.
//     fn close(&mut self);
// }
//
// /// A collection of various formats describing the type of content produced during a render.
// #[derive(Debug, Clone, Copy)]
// pub enum RenderMode {
//     /// Indicates that that renderer should be done through the terminal or an external display.
//     Human,
//     /// Indicates that renderer should be skipped.
//     None,
// }
//
//
// /// The return type of [`Env::step()`].
// pub struct StepFeedback<E: Env> {
//     /// The observation of the environment after taking an action.
//     pub observation: E::Observation,
//     /// The reward after taking an action.
//     pub reward: E::Reward,
//     /// Indication that the agent has reached a terminal state after taking an action, as defined by the task formulation.
//     /// If true, it is expected that [`Env::reset()`] is called to restart the environment.
//     pub terminated: bool,
//     /// Indication that the agent has reached a truncation condition after taking an action, which is outside the scope of the task formulation.
//     /// If true, [`Env::reset()`] should be called to restart the environment.
//     pub truncated: bool,
//     /// Additional information from the environment after taking an action.
//     pub info: E::Info,
// }
// // todo envProperties
//
// /// Defines the bounds for the reward value that can be observed.
// #[derive(Clone, Debug, PartialEq, PartialOrd)]
// pub struct RewardRange {
//     /// The smallest possible reward that can be observed.
//     lower_bound: f32,
//     /// The largest possible reward that can be observed.
//     upper_bound: f32,
// }



