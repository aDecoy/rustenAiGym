use crate::evolusjon::evolusjon_steg_plugin::Kjøretilstand;
use crate::evolusjon::phenotype_plugin::PlankPhenotype;
use crate::genome::genome_stuff::InnovationNumberGlobalCounter;
use crate::monitoring::camera_stuff::AllIndividerWindowTag;
use crate::monitoring::simulation_teller::SimulationGenerationTimer;
use avian2d::prelude::Forces;
use bevy::asset::Assets;
use bevy::mesh::Mesh;
use bevy::prelude::{ColorMaterial, Commands, Entity, NextState, Query, Res, ResMut, Time, Transform, Window, With};

pub trait EnvironmentSpesificIndividStuff {
    fn spawn_a_random_new_individual(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        innovation_number_global_counter: &mut ResMut<InnovationNumberGlobalCounter>,
        n: i32,
    ); // fn agent_action(query: Query<Transform, With<Individual>>) {
    fn agent_action_and_fitness_evaluation(
        // mut query: Query<(&mut Transform, &mut PlankPhenotype, &mut LinearVelocity, Option<Forces>, Entity), (With<PlankPhenotype>)>,
        query: Query<
            (
                &mut Transform,
                &mut PlankPhenotype,
                // &mut LinearVelocity,
                Forces,
                Entity,
            ),
            (With<PlankPhenotype>),
        >,
        time: Res<Time>,
    ); // Turns out Rust dont have any good default parameter solutions. At least none that i like. Ok kanskje det er noen ok løsninger. https://www.thecodedmessage.com/posts/default-params/
    fn spawn_a_random_new_individual2(
        commands: Commands,
        meshes: ResMut<Assets<Mesh>>,
        materials: ResMut<Assets<ColorMaterial>>,
        innovation_number_global_counter: ResMut<InnovationNumberGlobalCounter>,
    );
    fn check_if_done(
        query: Query<(&mut Transform, &mut PlankPhenotype), (With<PlankPhenotype>)>,
        next_state: ResMut<NextState<Kjøretilstand>>,
        simulation_timer: Res<SimulationGenerationTimer>,
        window: Query<&Window, With<AllIndividerWindowTag>>,
    );
}
