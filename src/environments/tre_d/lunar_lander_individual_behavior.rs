use crate::evolusjon::evolusjon_steg_plugin::Kjøretilstand;
use crate::evolusjon::hjerne_fenotype::PhenotypeNeuralNetwork;
use crate::evolusjon::phenotype_plugin::PlankPhenotype;
use crate::genome::genome_stuff::{new_random_genome, InnovationNumberGlobalCounter};
use crate::monitoring::camera_stuff::{AllIndividerWindowTag, RENDER_LAYER_ALLE_INDIVIDER};
use crate::monitoring::simulation_teller::SimulationGenerationTimer;
use avian2d::prelude::{Collider, CollisionLayers, Forces, RigidBody};
use bevy::asset::{Assets, Handle};
use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::basic::PURPLE;
use bevy::color::Color;
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;

pub struct LunarLanderIndividBehaviors;

// impl EnvironmentSpesificIndividStuff for LunarLanderIndividBehaviors {
impl LunarLanderIndividBehaviors {
    pub(crate) fn spawn_a_random_new_individual(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>, innovation_number_global_counter: &mut ResMut<InnovationNumberGlobalCounter>, n: i32) {

        let genome = new_random_genome(2, 2, innovation_number_global_counter);

        let start_position: Vec<f32> = vec!(0.0, 0.5, 0.0);
        let individ_size : Vec<f32> = vec!(10.0, 10.5, 10.0);
        let mesh_handle: Handle<Mesh> = meshes.add(Cuboid::new(individ_size[0], individ_size[1], individ_size[2]));
        let material_handle: Handle<StandardMaterial> = materials.add(StandardMaterial{ base_color: Color::from(PURPLE) , ..default()});
        
        
        commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            Transform::from_xyz(start_position[0], start_position[1], start_position[2]),

            PlankPhenotype {
                score: 0.0,
                obseravations: vec![0.0, 0.0],
                // phenotype_layers: create_phenotype_layers(genome.clone()),
                phenotype_layers: PhenotypeNeuralNetwork::new(&genome),
                // genotype: genome_entity,
            }, // alt 1
            genome,
            // Collider::rectangle(1.0, 1.0),
            Collider::capsule(individ_size[0], individ_size[1]),
            RigidBody::Dynamic,
            CollisionLayers::new(0b0001, 0b0010),
            // LinearVelocity { 0: Vec2::new(0.0, 0.0) },
            // Forces { force: Vec2::new(0.0, 0.0), persistent: false , ..default()} ,
            // Forces::new(Vec2::X).with_persistence(false),
            TextLayout::new_with_justify(Justify::Center),
            RenderLayers::layer(RENDER_LAYER_ALLE_INDIVIDER),
            // Individ {},
        ));
    }

        pub fn agent_action_and_fitness_evaluation(query: Query<(&mut Transform, &mut PlankPhenotype, Forces, Entity), (With<PlankPhenotype>)>, time: Res<Time>) {
        todo!()
    }

    pub fn spawn_a_random_new_individual2(commands: Commands, meshes: ResMut<Assets<Mesh>>, materials: ResMut<Assets<ColorMaterial>>, innovation_number_global_counter: ResMut<InnovationNumberGlobalCounter>) {
        todo!()
    }

   pub  fn check_if_done(query: Query<(&mut Transform, &mut PlankPhenotype), (With<PlankPhenotype>)>, next_state: ResMut<NextState<Kjøretilstand>>, simulation_timer: Res<SimulationGenerationTimer>, window: Query<&Window, With<AllIndividerWindowTag>>) {
        todo!()
    }
}

