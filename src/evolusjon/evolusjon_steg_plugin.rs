use crate::environments::felles_miljø_traits::EnvironmentSpesificIndividStuff;
use crate::environments::gammelt_2d::individuals_behavior_for_2d_environments::ToDimensjonelleMijøSpesifikkeIndividOppførsler;
use crate::environments::gammelt_2d::lunar_lander_environment2d::LANDING_SITE;
use crate::environments::gammelt_2d::moving_plank_2d::{PLANK_HIGHT, PLANK_LENGTH};
use crate::environments::tre_d::lunar_lander_individual_behavior::LunarLanderIndividBehaviors;
use crate::evolusjon::hjerne_fenotype::nullstill_nettverk_verdier_til_0;
use crate::evolusjon::phenotype_plugin::{
    IndividFitnessLabelTextTag, PhentypeAndGenome, PlankPhenotype, add_observers_to_individuals, update_phenotype_network_for_changed_genomes,
};
use crate::genome::genom_muteringer::{MutasjonerErAktive, lock_mutation_stability, mutate_genomes};
use crate::genome::genome_stuff::{Genome, InnovationNumberGlobalCounter, new_random_genome};
use crate::monitoring::camera_stuff::{AllIndividerCameraTag, AllIndividerWindowTag};
use crate::monitoring::simulation_teller::SimulationGenerationTimer;
use crate::{
    ACTIVE_ENVIROMENT, ANT_INDIVIDER_SOM_OVERLEVER_HVER_GENERASJON, ANT_PARENTS_HVER_GENERASJON, EnvValg, Kjøretilstand as OtherKjøretilstand, START_POPULATION_SIZE,
};
use avian2d::math::{AdjustPrecision, Vector};
use avian2d::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::basic::PURPLE;
use bevy::color::palettes::tailwind::CYAN_300;
use bevy::prelude::KeyCode::{KeyR, KeyT};
use bevy::prelude::*;
use lazy_static::lazy_static;
use rand::prelude::IndexedRandom;
use rand::thread_rng;
use std::cmp::{Ordering, max, min};
use std::collections::HashMap;
use std::ops::Mul;

// pub enum PossibleBehaviorSets{
//     TOd( ToDimensjonelleMijøSpesifikkeIndividOppførsler),
//     LUNAR_LANDER_3D(LunarLanderIndividBehaviors)
// }

pub enum PossibleBehaviorSets {
    TOd { oppførsel: ToDimensjonelleMijøSpesifikkeIndividOppførsler },
    LUNAR_LANDER_3D { oppførsel: LunarLanderIndividBehaviors },
}

pub struct EvolusjonStegPlugin {
    pub environmentSpesificIndividStuff: PossibleBehaviorSets,
}

#[derive(Message)]
pub struct PopulationIsSpawnedMessage;

impl Plugin for EvolusjonStegPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ResetToStartPositionsEvent>()
            .insert_state(Kjøretilstand::Kjørende)
            .insert_state(MutasjonerErAktive::Ja) // todo la en knapp skru av og på mutasjon, slik at jeg kan se om identiske chilren gjør nøyaktig det som parents gjør
            .add_message::<PopulationIsSpawnedMessage>()
            .add_message::<SpawnNewIndividualMessage>()
            .add_message::<ResetToStartPositionsEvent>()
            .add_message::<GenerationIsDone>()
            .add_message::<CheckIfDoneRequest>()
            .add_systems(Startup, ((spawn_start_population,).chain(),))
            .add_systems(
                PostStartup,
                (
                    add_observers_to_individuals.run_if(on_message::<PopulationIsSpawnedMessage>),
                    reset_to_star_pos.run_if(on_message::<PopulationIsSpawnedMessage>),
                ),
            )
            .add_systems(
                Update,
                (
                    extinction_on_t,
                    reset_event_ved_input,
                    // (
                    // ToDimensjonelleMijøSpesifikkeIndividOppførsler::check_if_done,
                    // self.environmentSpesificIndividStuff::check_if_done,
                    // match &self.environmentSpesificIndividStuff {
                    //     // PossibleBehaviorSets::LUNAR_LANDER_3D { oppførsel } => oppførsel::check_if_done,
                    //     PossibleBehaviorSets::LUNAR_LANDER_3D { oppførsel } => oppførsel::check_if_done,
                    //     PossibleBehaviorSets::TOd { oppførsel } => oppførsel::check_if_done,
                    // },
                    // check_if_done.run_if(every_time_if_stop_on_right_window()), // ELDSTE simulasjoner hadde mål å bevege seg til høyre
                    // ToDimensjonelleMijøSpesifikkeIndividOppførsler::agent_action_and_fitness_evaluation,
                    // )
                    //     .run_if(in_state(Kjøretilstand::Kjørende)),
                    (
                        kill_worst_individuals,
                        reset_to_star_pos,
                        mutate_genomes.run_if(in_state(MutasjonerErAktive::Ja)),
                        // updatePhenotypeNetwork for entities with mutated genomes .run_if(in_state(MutasjonerErAktive::Ja)),
                        update_phenotype_network_for_changed_genomes.run_if(in_state(MutasjonerErAktive::Ja)),
                        nullstill_nettverk_verdier_til_0,
                        create_new_children,
                        // ToDimensjonelleMijøSpesifikkeIndividOppførsler::spawn_a_random_new_individual2,
                        lock_mutation_stability,
                        add_observers_to_individuals.after(create_new_children),
                        set_to_kjørende_state,
                    )
                        .chain()
                        .run_if(in_state(Kjøretilstand::EvolutionOverhead)),
                ),
            );

        // todo ha denne pluginen sende messages som plukkes opp av implementasjon spesifikke plugins
        
        // }
        //     let a = match &self.environmentSpesificIndividStuff {
        //     // PossibleBehaviorSets::LUNAR_LANDER_3D { oppførsel } => oppførsel::check_if_done,
        //     PossibleBehaviorSets::LUNAR_LANDER_3D { oppførsel } => {
        // println!("LUNAR_LANDER_3D!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        // LunarLanderIndividBehaviors::spawn_a_random_new_individual;
        // },
        // PossibleBehaviorSets::TOd { oppførsel } => {
        // println!("LUNAR_LANDER_2D!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        // ToDimensjonelleMijøSpesifikkeIndividOppførsler::spawn_a_random_new_individual;
        // },
        // };

        // if let  PossibleBehaviorSets::LUNAR_LANDER_3D{oppførsel} = &self.environmentSpesificIndividStuff {
        //         app.add_systems(
        //              Update,(
        //                 LunarLanderIndividBehaviors::spawn_a_random_new_individual,
        //                 LunarLanderIndividBehaviors::agent_action_and_fitness_evaluation,
        //                 LunarLanderIndividBehaviors::check_if_done,
        //             )
        //         );
        // };
    }
}

// todo kanksje inatrodusere configure_sets og SystemSets for de ulike stegene i evolusjon?

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum Kjøretilstand {
    #[default]
    Pause,
    Kjørende,
    EvolutionOverhead,
    // FitnessEvaluation,
    // Mutation,
    // ParentSelection,
    // SurvivorSelection,
}

pub(crate) fn spawn_start_population(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut innovation_number_global_counter: ResMut<InnovationNumberGlobalCounter>,
    mut spawn_new_individual_writer: MessageWriter<SpawnNewIndividualMessage>,
) {
    for n in 0i32..START_POPULATION_SIZE {
        // for n in 0i32..1 {
        ToDimensjonelleMijøSpesifikkeIndividOppførsler::spawn_a_random_new_individual(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut innovation_number_global_counter,
            n,
            &mut spawn_new_individual_writer,
        );
    }
}

fn kill_worst_individuals(mut commands: Commands, query: Query<(Entity, &PlankPhenotype), With<PlankPhenotype>>) {
    let mut population = Vec::new();

    //sort_individuals
    for (entity, plank) in query.iter() {
        population.push((entity, plank))
    }
    // println!("population before sort: {:?}", population);
    // sort asc
    //     population.sort_by(| a, b| if a.1.score > b.1.score { Ordering::Greater } else if a.1.score < b.1.score { Ordering::Less } else { Ordering::Equal });
    population.sort_by(|(_, a), (_, b)| {
        if a.score > b.score {
            Ordering::Greater
        } else if a.score < b.score {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    });
    // println!("population after sort:  {:?}", population);
    // let number_of_individuals_to_kill = min(4, population.len() - 1);

    // println!("pop size {}, want to kill pop size - 3 = {}. Max killing 0", population.len(), population.len()  as i32- number_of_individuals_to_leave_alive);
    // println!("pop size {}, want to kill pop size - 3 = {}. Max killing 0, ressulting in {}", population.len(), population.len() - number_of_individuals_to_leave_alive, max(0, population.len() - number_of_individuals_to_leave_alive));

    let number_of_individuals_to_kill: usize = max(0, population.len() as i32 - ANT_INDIVIDER_SOM_OVERLEVER_HVER_GENERASJON) as usize;
    println!("killing of {} entities", number_of_individuals_to_kill);
    for (entity, _) in &population[0..number_of_individuals_to_kill] {
        // println!("despawning entity {} ", entity.index());
        commands.entity(*entity).despawn_children();
        commands.entity(*entity).despawn();
    }
}

fn set_to_kjørende_state(mut next_state: ResMut<NextState<Kjøretilstand>>) {
    next_state.set(Kjøretilstand::Kjørende);
}

#[derive(Message)]
pub(crate) struct GenerationIsDone;
#[derive(Message)]
pub(crate) struct CheckIfDoneRequest;

fn request_env_to_check_if_done_with_generation(
    mut message_writer: MessageWriter<CheckIfDoneRequest>,
){
    message_writer.write(CheckIfDoneRequest);
}

fn adjust_kjøretilstand_if_generation_is_done(
    mut message_reader: MessageReader<GenerationIsDone>,
    mut next_state: ResMut<NextState<Kjøretilstand>>,
){
    if !message_reader.is_empty(){
        next_state.set(Kjøretilstand::EvolutionOverhead);
    }
    message_reader.read();
}

fn extinction_on_t(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity), With<PlankPhenotype>>,
    key_input: Res<ButtonInput<KeyCode>>,
    innovation_number_global_counter: ResMut<InnovationNumberGlobalCounter>,
    mut spawn_new_individual_writer: MessageWriter<SpawnNewIndividualMessage>,
) {
    if key_input.just_pressed(KeyT) {
        for (entity) in query.iter() {
            commands.entity(entity).despawn();
        }
        spawn_start_population(commands, meshes, materials, innovation_number_global_counter, spawn_new_individual_writer)
    }
}

#[derive(Message, Debug)]
pub struct SpawnNewIndividualMessage {
    pub new_genome: Genome,
    pub n: i32,
}

fn create_new_children(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &PlankPhenotype, &Genome), With<PlankPhenotype>>,
    camera: Query<Entity, With<AllIndividerCameraTag>>,
    mut spawn_new_individual_writer: MessageWriter<SpawnNewIndividualMessage>,
) {
    let mut population = Vec::new();
    //sort_individuals
    for (entity, plank, genome) in query.iter() {
        population.push(PhentypeAndGenome {
            phenotype: plank,
            genome: genome,
            entity_index: entity.index(),
            entity: entity,
            entity_bevy_generation: entity.generation().to_bits(),
        })
    }
    // println!("population size when making new individuals: {}", population.len() );
    // println!("parents before sort: {:?}", population);
    // todo legge inn generation_rank som en komponent, og sortere i ett system ??
    // todo alt. ha sorterte Plank også ta inn genom eller entity/ eller (phenotype,genom) tuples eller ny struct som bare brukes til dette. ..
    // sadfasdf
    // sort desc
    population.sort_by(|a, b| {
        if a.phenotype.score > b.phenotype.score {
            Ordering::Less
        } else if a.phenotype.score < b.phenotype.score {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
    // println!("parents after sort:  {:?}", population);

    let mut parents = Vec::new();

    // Select potential Parent
    for n in 0..min(ANT_PARENTS_HVER_GENERASJON, population.len()) {
        let parent = population[n].clone();
        println!(
            "the lucky winner was parent with entity index {}, with entity generation {} that had score: {} ",
            parent.entity_index, parent.entity_bevy_generation, parent.phenotype.score
        );
        parents.push(parent);
    }

    // For now, simple fill up population to pop  size . Note this does ruin some evolution patters if competition between indiviuals are a thing in the environment
    let pop_to_fill = START_POPULATION_SIZE - population.len() as i32;
    let mut thread_random = thread_rng();
    for _ in 0..pop_to_fill {
        // let uniform_dist = Uniform::new(-1.0, 1.0);
        // https://stackoverflow.com/questions/34215280/how-can-i-randomly-select-one-element-from-a-vector-or-array
        // let parent: &PlankPhenotype = parents.sample(&mut thread_random);

        // let mut new_genome : Genome = commands.get_entity(parents.choose(&mut thread_random).expect("No potential parents :O !?").genotype).expect("burde eksistere").clone();
        let parent: &PhentypeAndGenome = parents.choose(&mut thread_random).expect("No potential parents :O !?");
        // println!("the lucky winner was parent with entity index {}, that had score {} ", parent.entity_index, parent.phenotype.score);
        let mut new_genome: Genome = parent.genome.clone();

        // NB: mutation is done in a seperate bevy system
        new_genome.allowed_to_change = true;

        // let text_style = TextStyle {
        //     font_size: 20.0,
        //     color: Color::WHITE,
        //     ..default()
        // };

        // todo kanskje heller bare lage en event/ message til dit det skal lages en ny enitet? Trenger å skille evolusjon steg rekkefølge fra mer mijø og individ spesifikke ting.
        //  Ikke kalle direkte, bare lage en event som mijø / individ eller phenotype lytter på. Da også egentlig lett å utived senere til flere individ-art regler per mijø

        spawn_new_individual_writer.write(SpawnNewIndividualMessage { new_genome: new_genome, n: 0 });

        // spawn_new_2d_individ(&mut commands, new_genome, rectangle_mesh_handle, material_handle);
    }
}

#[derive(Message, Debug, Default)]
pub(crate) struct ResetToStartPositionsEvent;



fn reset_event_ved_input(user_input: Res<ButtonInput<KeyCode>>, mut reset_events: MessageWriter<ResetToStartPositionsEvent>) {
    if user_input.pressed(KeyR) {
        reset_events.write_default();
    }
}
fn reset_to_star_pos(user_input: Res<ButtonInput<KeyCode>>, mut reset_events: MessageWriter<ResetToStartPositionsEvent>) {
        reset_events.write_default();
}


