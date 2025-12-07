use crate::environments::tre_d::lunar_lander_individual_behavior::START_POSITION;
// use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use crate::evolusjon::evolusjon_steg_plugin::{PopulationIsSpawnedMessage, SpawnNewIndividualMessage, SpawnNewIndividualWithGenomeMessage};
use crate::evolusjon::hjerne_fenotype::PhenotypeNeuralNetwork;
use crate::evolusjon::phenotype_plugin::PlankPhenotype;
use crate::genome::genome_stuff::{InnovationNumberGlobalCounter, new_random_genome};
use crate::monitoring::camera_stuff::RENDER_LAYER_ALLE_INDIVIDER;
use avian3d::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::basic::PURPLE;
use bevy::prelude::*;
// todo lag spwawn individ on event plugin oig legg den til i main. starupt after create population

pub struct SpawnLunarLanderPlugin;

pub static INDIVID_DEFAULT_COLOR: Srgba = PURPLE;

impl Plugin for SpawnLunarLanderPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnNewIndividualMessage>()
            .add_message::<PopulationIsSpawnedMessage>()
            .add_systems(
                Startup,
                (spawn_new_3d_individ_med_nytt_genome_meldingsspiser.after(crate::evolusjon::evolusjon_steg_plugin::spawn_start_population)),
            )
            .add_systems(
                Update,
                (
                    create_genome_and_send_spawn_message,
                    spawn_new_3d_individ_med_nytt_genome_meldingsspiser.after(create_genome_and_send_spawn_message),
                ),
            );
    }
}

fn create_genome_and_send_spawn_message(
    mut innovation_number_global_counter: ResMut<InnovationNumberGlobalCounter>,
    mut spawn_new_individual_message_reader: MessageReader<SpawnNewIndividualMessage>,
    mut spawn_new_individual_message_writer: MessageWriter<SpawnNewIndividualWithGenomeMessage>,
) {
    for _ in spawn_new_individual_message_reader.read() {
        let genome = new_random_genome(3, 3, &mut innovation_number_global_counter);
        spawn_new_individual_message_writer.write(SpawnNewIndividualWithGenomeMessage { new_genome: genome });
    }
}

fn spawn_new_3d_individ_med_nytt_genome_meldingsspiser(
    mut spawn_new_individual_message: MessageReader<SpawnNewIndividualWithGenomeMessage>,
    mut population_done_spawning_in: MessageWriter<PopulationIsSpawnedMessage>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if spawn_new_individual_message.is_empty() {
       return; 
    }
    for message in spawn_new_individual_message.read() {
        // let genome = new_random_genome(3, 3, &mut innovation_number_global_counter);
        let genome = message.new_genome.clone();

        let length = 1.0;
        // let start_position: Vec<f32> = vec![1.0, 1.5, 1.0];
        // let start_position: Vec<f32> = vec![1.0, 1.5, 1.0];
        let individ_size: Vec<f32> = vec![0.5, 0.5, 0.5];
        // cube
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(individ_size[0], individ_size[1], individ_size[2]))),
            // MeshMaterial3d(materials.add(Color::srgb(0.5, 0.4, 0.3))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::from(INDIVID_DEFAULT_COLOR),
                ..default()
            })),
            Transform::from_translation(START_POSITION.clone()), // from_xyz(START_POSITION.x, start_position[1], start_position[2]),
            PlankPhenotype {
                score: 0.0,
                obseravations: vec![0.0, 0.0],
                // phenotype_layers: create_phenotype_layers(genome.clone()),
                phenotype_layers: PhenotypeNeuralNetwork::new(&genome),
                // genotype: genome_entity,
            }, // alt 1
            genome,
            RigidBody::Dynamic,
            CollisionLayers::new(0b0001, 0b0010),
            Collider::cuboid(length, length, length),
            TextLayout::new_with_justify(Justify::Center),
            RenderLayers::layer(RENDER_LAYER_ALLE_INDIVIDER),
        ));
    }

    population_done_spawning_in.write(PopulationIsSpawnedMessage);
}
