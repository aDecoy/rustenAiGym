use crate::evolusjon::evolusjon_steg_plugin::{Kjøretilstand, SpawnNewIndividualMessage};
use crate::evolusjon::phenotype_plugin::PlankPhenotype;
use crate::genome::genome_stuff::{InnovationNumberGlobalCounter, new_random_genome};
use crate::monitoring::camera_stuff::AllIndividerWindowTag;
use crate::monitoring::simulation_teller::SimulationGenerationTimer;
use avian2d::prelude::Forces;
use bevy::asset::Assets;
use bevy::mesh::Mesh;
use bevy::prelude::*;

pub struct LunarLanderIndividBehaviors;

// impl EnvironmentSpesificIndividStuff for LunarLanderIndividBehaviors {
impl LunarLanderIndividBehaviors {
    pub(crate) fn spawn_a_random_new_individual(
        mut spawn_new_individual_message: MessageWriter<SpawnNewIndividualMessage>,
        innovation_number_global_counter: &mut ResMut<InnovationNumberGlobalCounter>,
        n: i32,
    ) {
        let genome = new_random_genome(2, 2, innovation_number_global_counter);
        spawn_new_individual_message.write(SpawnNewIndividualMessage { new_genome: genome, n: n });
    }

    pub fn agent_action_and_fitness_evaluation(query: Query<(&mut Transform, &mut PlankPhenotype, Forces, Entity), (With<PlankPhenotype>)>, time: Res<Time>) {
        todo!()
    }

    pub fn spawn_a_random_new_individual2(
        commands: Commands,
        meshes: ResMut<Assets<Mesh>>,
        materials: ResMut<Assets<ColorMaterial>>,
        innovation_number_global_counter: ResMut<InnovationNumberGlobalCounter>,
    ) {
        todo!()
    }

    pub fn check_if_done(
        query: Query<(&mut Transform, &mut PlankPhenotype), (With<PlankPhenotype>)>,
        next_state: ResMut<NextState<Kjøretilstand>>,
        simulation_timer: Res<SimulationGenerationTimer>,
        window: Query<&Window, With<AllIndividerWindowTag>>,
    ) {
        todo!()
    }
}
