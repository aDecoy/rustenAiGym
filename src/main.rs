use crate::environments::GenomeStuff::{new_random_genome, Genome, InnovationNumberGlobalCounter};
use crate::environments::moving_plank::{create_plank_env_falling, create_plank_env_moving_right, create_plank_ext_force_env_falling, MovingPlankPlugin, PIXELS_PER_METER, PLANK_HIGHT, PLANK_LENGTH};
use crate::environments::simulation_teller::{SimulationGenerationTimer, SimulationRunningTellerPlugin};
use avian2d::math::{AdjustPrecision, Vector};
// use bevy_rapier2d::na::DimAdd;
// use bevy_rapier2d::prelude::*;
use avian2d::prelude::*;
use bevy::asset::AsyncWriteExt;
// use bevy::asset::io::memory::Value::Vec;
use bevy::color::palettes::basic::PURPLE;
use bevy::ecs::query::QueryIter;
use bevy::prelude::KeyCode::{KeyE, KeyK, KeyP, KeyR, KeyT};
use bevy::prelude::*;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy_inspector_egui::egui::emath::Numeric;
use lazy_static::lazy_static;
use rand::distributions::Uniform;
use rand::seq::SliceRandom;
use rand::{random, thread_rng, Rng};
use std::cmp::{max, min, Ordering, PartialEq};
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::sync::{Arc, RwLock};
use std::vec::Vec;
use crate::environments::drawNetwork::{draw_network_in_genome, draw_network_in_genome2, oppdater_node_tegninger, remove_drawing_of_network_for_best_individ};
use crate::environments::GenomeStuff::{NodeGene, WeightGene};
use crate::environments::genomMuteringer::lock_mutation_stability;
use crate::environments::LunarLanderEnvironment::{spawn_ground, spawn_landing_target, spawn_roof, LANDING_SITE};

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
        .insert_state(Kjøretilstand::Pause)
        // .add_plugins(WorldInspectorPlugin::new())
        .insert_state(EttHakkState::DISABLED)
        .init_resource::<GenerationCounter>()
        .insert_resource(InnovationNumberGlobalCounter { count: 0 })
        // .init_resource(     EnvValg { Homing} )
        .add_event::<ResetToStartPositionsEvent>()
        .add_systems(Startup, (
            setup_camera,
            (spawn_start_population,
             spawn_drawing_of_network_for_best_individ).chain(),
            spawn_ground,
            spawn_roof,
            spawn_landing_target,
        ))
        .add_systems(Update, (
            endre_kjøretilstand_ved_input,
            reset_event_ved_input,
            reset_to_star_pos_on_event,
            extinction_on_t,
            (
                // print_pois_velocity_and_force,
                agent_action_and_fitness_evaluation.run_if(in_state(Kjøretilstand::Kjørende)),
                label_plank_with_current_score,
                // oppdater_node_tegninger,
                remove_drawing_of_network_for_best_individ,
                spawn_drawing_of_network_for_best_individ,
            ).chain().run_if(in_state(Kjøretilstand::Kjørende)),
            check_if_done,
            // check_if_done.run_if(every_time_if_stop_on_right_window()),
            (
                print_pop_conditions,
                increase_generation_counter,
                lock_mutation_stability,
                save_best_to_history,
                kill_worst_individuals,
                remove_drawing_of_network_for_best_individ,
                spawn_drawing_of_network_for_best_individ,
                create_new_children,
                // spawn_a_random_new_individual2,
                // mutate_planks,
                // mutate_genomes,
                reset_to_star_pos,
                set_to_kjørende_state).chain().run_if(in_state(Kjøretilstand::EvolutionOverhead)),
        ),
        )
        // Environment spesific : Later changed
        .add_plugins(MovingPlankPlugin)
        .add_plugins(SimulationRunningTellerPlugin);

    app.run();
}


fn every_time() -> impl Condition<()> {
    IntoSystem::into_system(|mut flag: Local<bool>| {
        *flag = true;
        *flag
    })
}
fn every_time_if_stop_on_right_window() -> impl Condition<()> {
    IntoSystem::into_system(|mut flag: Local<bool>| {
        *flag = match ACTIVE_ENVIROMENT {
            EnvValg::Høyre | EnvValg::Fall | EnvValg::FallVelocityHøyre | EnvValg::FallExternalForcesHøyre => { true }
            _ => { false }
        };
        *flag
    })
}


//////////////


#[derive(Resource, Default, Debug)]
struct SimulationTimer {
    count: i32,
}


/////////////////// genration counter

#[derive(Resource, Default, Debug)]
struct GenerationCounter {
    count: i32,
}


static SIMULATION_GENERATION_MAX_TIME: f64 = 4.0; // seconds

fn increase_generation_counter(mut generation_counter: ResMut<GenerationCounter>) {
    generation_counter.count += 1;
}

/////////////////// Metadata obvservation

fn save_best_to_history(query: Query<&PlankPhenotype>,
                        generation_counter: Res<GenerationCounter>) {
    // let mut file = File::create("history.txt").expect("kunne ikke finne filen");
    let mut f = File::options().append(true).open("history.txt").expect("kunne ikke åpne filen");

    let population = get_population_sorted_from_best_to_worst(query.iter());

    let best = population[0];
    let best_score = best.score;
    // let best_id = best;
    let generation = generation_counter.count;
    // let row = format!("generation {generation}, Best individual: {best}, HIGHEST SCORE: {best_score},  ");
    let row = format!("generation {generation}, HIGHEST SCORE: {best_score},  ");
    writeln!(&mut f, "{}", row).expect("TODO: panic message");
}

// fn get_population_sorted_from_best_to_worst<'lifetime_a>(query: QueryIter<'lifetime_a, '_, &PlankPhenotype, ()>) -> Vec<&'lifetime_a PlankPhenotype> {
fn get_population_sorted_from_best_to_worst<'lifetime_a>(query: QueryIter<'lifetime_a, '_, &PlankPhenotype, ()>) -> Vec<&'lifetime_a PlankPhenotype> {
    // fn get_population_sorted_from_best_to_worst<'a>(query: Query<&'a PlankPhenotype>) -> Vec<&'a PlankPhenotype> {
    let mut population = Vec::new();
    //sort_individuals
    for (plank) in query {
        population.push(plank)
    }
    // sort desc
    population.sort_by(|a, b| if a.score > b.score { Ordering::Less } else if a.score < b.score { Ordering::Greater } else { Ordering::Equal });
    return population;
}

// all the lifteimes bassicly just means to keep return value alive as long as the input value
fn get_population_sorted_from_best_to_worst_v2<'lifetime_a>(query: QueryIter<'lifetime_a, '_, (Entity, &PlankPhenotype, &Genome), ()>) -> Vec<PhentypeAndGenome<'lifetime_a>> {
    // fn get_population_sorted_from_best_to_worst<'a>(query: Query<&'a PlankPhenotype>) -> Vec<&'a PlankPhenotype> {
    let mut population = Vec::new();
    //sort_individuals
    for (entity, plank, genome) in query {
        // for (plank) in query {
        // population.push(plank)
        population.push(PhentypeAndGenome {
            phenotype: plank,
            genome: genome,
            entity_index: entity.index(),
            entity_bevy_generation: entity.generation(),
        });
    }

    // sort desc
    population.sort_by(|a, b| if a.phenotype.score > b.phenotype.score { Ordering::Less } else if a.phenotype.score < b.phenotype.score { Ordering::Greater } else { Ordering::Equal });
    return population;
}

fn print_pop_conditions(query: Query<(Entity, &PlankPhenotype, &Genome)>,
                        generation_counter: Res<GenerationCounter>) {
    let population = get_population_sorted_from_best_to_worst_v2(query.iter());
    let best = population[0].clone();

    // let best_id = best.genotype.index();
    let all_fitnesses = population.iter().map(|individ| individ.phenotype.score);
    // println!("generation {} just ended, has population size {} Best individual: {} has fitness {} ", generation_counter.count, population.len(), best_id, best.score);
    println!("generation {} just ended, has population size {} Best individual has fitness {} ", generation_counter.count, population.len(), best.phenotype.score);
    // println!("all fintesses for generation: ");
    // all_fitnesses.for_each(|score| print!("{} ", score));
    println!();
    println!("all fintesses for generation: ");
    population.into_iter().for_each(|individ| print!("Entity {} from bevy-generation {} har score {},", individ.entity_index, individ.entity_bevy_generation, individ.phenotype.score));
    println!();
}

/////////////////// create/kill/develop  new individuals

static START_POPULATION_SIZE: i32 = 1;

fn spawn_start_population(mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<ColorMaterial>>,
                          mut innovationNumberGlobalCounter: ResMut<InnovationNumberGlobalCounter>,
) {
    for n in 0i32..START_POPULATION_SIZE {
        // for n in 0i32..1 {

        spawn_a_random_new_individual(&mut commands, &mut meshes, &mut materials, &mut innovationNumberGlobalCounter, n);
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


// Turns out Rust dont have any good default parameter solutions. At least none that i like. Ok kanskje det er noen ok løsninger. https://www.thecodedmessage.com/posts/default-params/
fn spawn_a_random_new_individual2(mut commands: Commands,
                                  mut meshes: ResMut<Assets<Mesh>>,
                                  mut materials: ResMut<Assets<ColorMaterial>>,
                                  mut innovationNumberGlobalCounter: ResMut<InnovationNumberGlobalCounter>,
) {
    let n: i32 = 1;
    spawn_a_random_new_individual(&mut commands, &mut meshes, &mut materials, &mut innovationNumberGlobalCounter, n)
}

fn spawn_a_random_new_individual(commands: &mut Commands,
                                 meshes: &mut ResMut<Assets<Mesh>>,
                                 materials: &mut ResMut<Assets<ColorMaterial>>,
                                 innovationNumberGlobalCounter: &mut ResMut<InnovationNumberGlobalCounter>,
                                 n: i32,
) {
    let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(PLANK_LENGTH, PLANK_HIGHT));
    let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE));
    // println!("Har jeg klart å lage en genome fra entity = : {}", genome2.allowed_to_change);
    // let text_style = TextStyle {
    //     font_size: 20.0,
    //     color: Color::WHITE,
    //     ..default()
    // };
    let text_justification = JustifyText::Center;

    let genome = new_random_genome(2, 2, innovationNumberGlobalCounter);

    match ACTIVE_ENVIROMENT {
        EnvValg::Høyre => commands.spawn(create_plank_env_moving_right(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + n as f32 * 50.0, z: 1.0 }, genome)),
        EnvValg::Fall | EnvValg::FallVelocityHøyre => commands.spawn(create_plank_env_falling(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + (n as f32 * 15.0), z: 1.0 }, genome)),
        EnvValg::FallExternalForcesHøyre | EnvValg::Homing | EnvValg::HomingGroud => { commands.spawn(create_plank_ext_force_env_falling(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + 3.3 * 50.0, z: 1.0 }, genome)) }
    }
        .with_children(|builder| {
            builder.spawn((
                Text2d::new("translation"),
                TextLayout::new_with_justify(JustifyText::Center),
                Transform::from_xyz(0.0,
                                    0.0,
                                    2.0),
                IndividLabelText,
            )
            );
        });
}

fn extinction_on_t(mut commands: Commands,
                   meshes: ResMut<Assets<Mesh>>,
                   materials: ResMut<Assets<ColorMaterial>>,
                   query: Query<(Entity), With<PlankPhenotype>>,
                   key_input: Res<ButtonInput<KeyCode>>,
                   innovationNumberGlobalCounter: ResMut<InnovationNumberGlobalCounter>,
) {
    if key_input.just_pressed(KeyT) {
        for (entity) in query.iter() {
            commands.entity(entity).despawn();
        }
        spawn_start_population(commands, meshes, materials, innovationNumberGlobalCounter)
    }
}

fn spawn_drawing_of_network_for_best_individ<'a>(
    // query: Query<(Entity, &PlankPhenotype), With<PlankPhenotype>>){
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,

    query: Query<(Entity, &PlankPhenotype, &Genome), With<PlankPhenotype>>) {
    // query: Query<'a, 'a, (Entity, &'a PlankPhenotype, &'a Genome), With<PlankPhenotype>>) {
    // let population = get_population_sorted_from_best_to_worst(query.iter());

    let elite = get_best_elite(query.iter());
    // todo kanskje heller lage en elite Resource som kan hentes inn direkte, istedenfor å sortere her og så kalle tegne funksjonen
    draw_network_in_genome2(commands, meshes, materials, elite.genome);
}
//
// #[derive(Debug,Resource)]
// struct EliteGenome {
//     Genome
// }

fn sort_best_to_worst<'a>(iteratior: QueryIter<'a, '_, (Entity, &PlankPhenotype, &'_ Genome), With<PlankPhenotype>>) -> Vec<PhentypeAndGenome<'a>> {
    let mut population = Vec::new();
    //sort_individuals
    for (entity, plank, genome) in iteratior {
        population.push(PhentypeAndGenome { phenotype: plank, genome: genome, entity_index: entity.index(), entity_bevy_generation: entity.generation() })
    }
    // sort desc
    population.sort_by(|a, b| if a.phenotype.score > b.phenotype.score { Ordering::Less } else if a.phenotype.score < b.phenotype.score { Ordering::Greater } else { Ordering::Equal });
    // println!("parents after sort:  {:?}", population);
    population
}
fn get_best_elite<'a>(iteratior: QueryIter<'a, '_, (Entity, &PlankPhenotype, &'_ Genome), With<PlankPhenotype>>) -> PhentypeAndGenome<'a> {
    let mut population = Vec::new();
    //sort_individuals
    for (entity, plank, genome) in iteratior {
        population.push(PhentypeAndGenome { phenotype: plank, genome: genome, entity_index: entity.index(), entity_bevy_generation: entity.generation() })
    }
    // sort desc
    population.sort_by(|a, b| if a.phenotype.score > b.phenotype.score { Ordering::Less } else if a.phenotype.score < b.phenotype.score { Ordering::Greater } else { Ordering::Equal });
    // println!("parents after sort:  {:?}", population);
    let elite = population.get(0).unwrap();
    return PhentypeAndGenome{ genome : elite.genome , entity_index: elite.entity_index, phenotype : elite.phenotype, entity_bevy_generation: elite.entity_bevy_generation };
    return elite.clone(); // KAN DET VÆRE AT DETTE LAGER EN NY GENOME`?
}
// fn get_population_sorted_from_best_to_worst<'lifetime_a>(query: QueryIter<'lifetime_a, '_, &crate::PlankPhenotype, ()>) -> Vec<&'lifetime_a crate::PlankPhenotype> {

fn kill_worst_individuals(
    mut commands: Commands,
    query: Query<(Entity, &PlankPhenotype), With<PlankPhenotype>>) {
    let mut population = Vec::new();

    //sort_individuals
    for (entity, plank) in query.iter() {
        population.push((entity, plank))
    }
    // println!("population before sort: {:?}", population);
    // sort asc
    //     population.sort_by(| a, b| if a.1.score > b.1.score { Ordering::Greater } else if a.1.score < b.1.score { Ordering::Less } else { Ordering::Equal });
    population.sort_by(|(_, a), (_, b)| if a.score > b.score { Ordering::Greater } else if a.score < b.score { Ordering::Less } else { Ordering::Equal });
    // println!("population after sort:  {:?}", population);
    // let number_of_individuals_to_kill = min(4, population.len() - 1);
    let number_of_individuals_to_leave_alive = 3;

    // println!("pop size {}, want to kill pop size - 3 = {}. Max killing 0", population.len(), population.len()  as i32- number_of_individuals_to_leave_alive);
    // println!("pop size {}, want to kill pop size - 3 = {}. Max killing 0, ressulting in {}", population.len(), population.len() - number_of_individuals_to_leave_alive, max(0, population.len() - number_of_individuals_to_leave_alive));

    let number_of_individuals_to_kill: usize = max(0, population.len() as i32 - number_of_individuals_to_leave_alive) as usize;
    println!("killing of {} entities", number_of_individuals_to_kill);
    for (entity, _) in &population[0..number_of_individuals_to_kill] {
        println!("despawning entity {} ", entity.index());
        commands.entity(*entity).despawn_recursive();
    }
}

// #[derive(Clone)]
// struct PhentypeGenome<'lifetime_a> {
//     phenotype: &'lifetime_a PlankPhenotype<'lifetime_a>,
//     genome: &'lifetime_a Genome<'lifetime_a>,
//     entity_index: u32,
//     entity_bevy_generation: u32,
// }

#[derive(Clone)]
struct PhentypeAndGenome<'lifetime_a> {
    phenotype: &'lifetime_a PlankPhenotype,
    genome: &'lifetime_a Genome,
    entity_index: u32,
    entity_bevy_generation: u32,
}


fn create_new_children(mut commands: Commands,
                       mut meshes: ResMut<Assets<Mesh>>,
                       mut materials: ResMut<Assets<ColorMaterial>>,
                       query: Query<(Entity, &PlankPhenotype, &Genome), With<PlankPhenotype>>) {
    let mut population = Vec::new();
    //sort_individuals
    for (entity, plank, genome) in query.iter() {
        population.push(PhentypeAndGenome { phenotype: plank, genome: genome, entity_index: entity.index(), entity_bevy_generation: entity.generation() })
    }
    // println!("population size when making new individuals: {}", population.len() );
    // println!("parents before sort: {:?}", population);
    // todo legge inn generation_rank som en komponent, og sortere i ett system ??
    // todo alt. ha sorterte Plank også ta inn genom eller entity/ eller (phenotype,genom) tuples eller ny struct som bare brukes til dette. ..
    // sadfasdf
    // sort desc
    population.sort_by(|a, b| if a.phenotype.score > b.phenotype.score { Ordering::Less } else if a.phenotype.score < b.phenotype.score { Ordering::Greater } else { Ordering::Equal });
    // println!("parents after sort:  {:?}", population);

    // create 3 children for each top 3
    let mut parents = Vec::new();

    // Parent selection is set to top 3
    for n in 0..min(1, population.len()) {
        let parent = population[n].clone();
        println!("the lucky winner was parent with entity index {}, with entity generation {} that had score: {} ", parent.entity_index, parent.entity_bevy_generation, parent.phenotype.score);
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

        let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(PLANK_LENGTH, PLANK_HIGHT));
        let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE).with_alpha(0.5));

        // let text_style = TextStyle {
        //     font_size: 20.0,
        //     color: Color::WHITE,
        //     ..default()
        // };
        match ACTIVE_ENVIROMENT {
            EnvValg::Fall | EnvValg::FallVelocityHøyre => commands.spawn(create_plank_env_falling(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + 3.3 * 50.0, z: 1.0 }, new_genome)),
            EnvValg::Høyre => commands.spawn(create_plank_env_moving_right(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + 3.3 * 50.0, z: 1.0 }, new_genome)),
            EnvValg::FallExternalForcesHøyre | EnvValg::Homing | EnvValg::HomingGroud => {
                commands.spawn(create_plank_ext_force_env_falling(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + 3.3 * 50.0, z: 0.0 }, new_genome))
            }
        }
            .with_children(|builder| {
                builder.spawn((
                    Text2d::new("Fitness label"),
                    TextLayout::new_with_justify(JustifyText::Center),
                    Transform::from_xyz(0.0, 0.0, 2.0),
                    IndividLabelText,
                )
                );
            });
    }
}

#[derive(Component)]
struct IndividLabelText;

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
}

static ACTIVE_ENVIROMENT: EnvValg = EnvValg::Homing;


// static LANDING_SITE: Vec2 = Vec2 { x: 100.0, y: -100.0 };

// state control

fn set_to_kjørende_state(
    mut next_state: ResMut<NextState<Kjøretilstand>>,
) {
    next_state.set(Kjøretilstand::Kjørende);
}
fn check_if_done(mut query: Query<(&mut Transform, &mut PlankPhenotype), ( With<PlankPhenotype>)>,
                 mut next_state: ResMut<NextState<Kjøretilstand>>,
                 simulation_timer: Res<SimulationGenerationTimer>,
                 window: Query<&Window>,
) {
    let max_width = window.single().width() * 0.5;


    match ACTIVE_ENVIROMENT {
        EnvValg::Høyre | EnvValg::Fall | EnvValg::FallVelocityHøyre | EnvValg::FallExternalForcesHøyre => {

            // done if one is all the way to the right of the screen
            for (individual, _) in query.iter_mut() {
                if individual.translation.x > max_width {
                    // println!("done");
                    ; // er det skalert etter reapier logikk eller pixler\?
                    next_state.set(Kjøretilstand::EvolutionOverhead)
                }
            }
        }
        EnvValg::Homing | EnvValg::HomingGroud => {
            if simulation_timer.main_timer.just_finished() {
                // println!("done");
                ; // er det skalert etter reapier logikk eller pixler\?
                next_state.set(Kjøretilstand::EvolutionOverhead);
            }
        }
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

// fn reset_to_star_pos(mut query: Query<(&mut Transform, &mut crate::PlankPhenotype, &mut Velocity), ( With<crate::PlankPhenotype>)>) {


#[derive(Event, Debug, Default)]
struct ResetToStartPositionsEvent;

fn reset_to_star_pos_on_event(
    mut reset_events: EventReader<ResetToStartPositionsEvent>,
    // query: Query<(&mut Transform, &mut crate::PlankPhenotype, &mut Velocity), ( With<crate::PlankPhenotype>)>,
    // query: Query<(&mut Transform, &mut crate::PlankPhenotype, &mut LinearVelocity, Option<&mut ExternalForce>), ( With<crate::PlankPhenotype>)>,
    query: Query<(&mut Transform, &mut crate::PlankPhenotype, &mut LinearVelocity, Option<&mut ExternalForce>)>,
) {
    if reset_events.read().next().is_some() {
        reset_to_star_pos(query);
    }
}

fn reset_event_ved_input(
    user_input: Res<ButtonInput<KeyCode>>,
    mut reset_events: EventWriter<ResetToStartPositionsEvent>,
) {
    if user_input.pressed(KeyR) {
        reset_events.send_default();
    }
}

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

fn print_pois_velocity_and_force(mut query: Query<(&Transform, &PlankPhenotype, &LinearVelocity, &ExternalForce), ( With<crate::PlankPhenotype>)>) {
    for (translation, plank, linvel, external_force) in query.iter_mut() {
        println!("translation {:#?}", translation);
        println!("linvel {:#?}", linvel);
        println!("external_force {:#?}", external_force);
        println!("----------------------------")
    }
}


fn reset_to_star_pos(mut query: Query<(&mut Transform, &mut PlankPhenotype, &mut LinearVelocity, Option<&mut ExternalForce>)>) {
    for (mut individual, mut plank, mut linvel, option_force) in query.iter_mut() {
        individual.translation.x = 0.0;
        if ACTIVE_ENVIROMENT != EnvValg::Høyre {
            individual.translation.y = 0.0;
        }
        plank.score = individual.translation.x.clone();
        plank.obseravations = vec!(individual.translation.x.clone(), individual.translation.y.clone());
        // velocity.angvel = 0.0;
        linvel.x = 0.0;
        linvel.y = 0.0;

        if let Some(mut force) = option_force {
            force.apply_force(Vector::ZERO);
        }
    }
}


// fn agent_action(query: Query<Transform, With<Individual>>) {
fn agent_action_and_fitness_evaluation
(
    mut query: Query<(&mut Transform, &mut PlankPhenotype, &mut LinearVelocity, Option<&mut ExternalForce>, Entity), ( With<PlankPhenotype>)>,
    time: Res<Time>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_secs_f64().adjust_precision();

    for (mut transform, mut plank, mut velocity, option_force, entity) in query.iter_mut() {
        plank.obseravations = vec![transform.translation.x.clone(), transform.translation.y.clone()];

        // let input_values = vec![1.0, 2.0]; // 2 inputs
        // let input_values = vec![individual.translation.x.clone() * 0.002, individual.translation.y.clone()* 0.002]; // 2 inputs
        let input_values = plank.obseravations.clone();
        // dbg!(&input_values);
        let action = plank.phenotype_layers.decide_on_action2(input_values);            // fungerer
        // dbg!(&action);
        // individual.translation.x += random::<f32>() * action * 5.0;
        // println!("action : {action}");
        let mut a = option_force.expect("did not have forces on individ!!? :( ");
        match ACTIVE_ENVIROMENT {
            EnvValg::Høyre | EnvValg::Fall => transform.translation.x += action[0] * 2.0,
            EnvValg::FallVelocityHøyre => velocity.0.x += action[0],
            // EnvValg::FallGlideBomb => velocity.0 += action,
            // EnvValg::FallExternalForcesHøyre => option_force.expect("did not have forces on individ!!? :( ").x = action,
            EnvValg::FallExternalForcesHøyre | EnvValg::Homing | EnvValg::HomingGroud => {
                // a.x = 100.0 * action[0] * delta_time;
                // a.y = 100.0 * action[1] * delta_time;
                a.x = 10.0 * action[0];
                a.y = 10.0 * action[1];

                // a.y = action;
                // NB: expternal force can be persitencte, or not. If not, then applyForce function must be called to do anything
                // println!("applying force {:#?}, and now velocity is {:?}", a.force(), velocity);
                // a.apply_force(Vector::ZERO);
            }
        }
        // println!("option force {:#?}", a.clone());
        // individual.translation.x += random::<f32>() * plank.phenotype * 5.0;
        match ACTIVE_ENVIROMENT {
            EnvValg::Høyre | EnvValg::Fall | EnvValg::FallVelocityHøyre | EnvValg::FallExternalForcesHøyre => {
                plank.score = transform.translation.x.clone();
            }
            EnvValg::Homing | EnvValg::HomingGroud => {
                // distance score to landingsite =  (x-x2)^2 + (y-y2)^2
                let distance = (LANDING_SITE.x - transform.translation.x).powi(2) + (LANDING_SITE.y - transform.translation.y).powi(2);
                // println!("Entity {} : Landingsite {:?}, and xy {} has x distance {}, and y distance {}", entity.index(), LANDING_SITE, transform.translation.xy(),
                //          (LANDING_SITE.x - transform.translation.x).powi(2), (LANDING_SITE.y - transform.translation.y).powi(2));
                // smaller sitance is good
                plank.score = 1000.0 / distance;
            }
        }
        // println!("individual {} chose action {} with inputs {}", plank.genotype.id.clone(), action ,plank.obseravations.clone()  );
    }
}

fn label_plank_with_current_score(
    mut query: Query<(&mut Text2d, &Parent), With<IndividLabelText>>,
    parent_query: Query<&PlankPhenotype>,
) {
    for (mut tekst, parent_entity) in query.iter_mut() {
        if let Ok(plank_phenotype) = parent_query.get(**parent_entity) {
            tekst.0 = plank_phenotype.score.to_string();
        }
    }
}


#[derive(Component, Debug)]
// #[derive(Component, Eq, Ord, PartialEq, PartialOrd, PartialEq)]
pub struct PlankPhenotype {
    pub score: f32,
    pub obseravations: Vec<f32>,
    // pub phenotype: f32,
    pub phenotype_layers: PhenotypeNeuralNetwork, // for now we always have a neural network to make decisions for the agent
    // pub genotype: Genome,
    // Genome er flyttet til å bli en component på Entity som også holder på PlankPhenotype komponent. Mistenker det fungerer bedre med tanke på bevy
    // pub genotype: Entity, // by having genotype also be an entity, we can query for it directly, without going down through parenkt PlankPhenotype that carries the genome ( phenotype is not that relevant if we do mutation or other pure genotype operations)
}


#[derive(Component, Debug)]
// #[derive(Component, Eq, Ord, PartialEq, PartialOrd, PartialEq)]
pub struct Individ {}

#[derive(Debug, )]
struct PhenotypeNeuralNetwork {
    ant_layers: usize,
    // Holder på objektene
    // alleNoder: Vec<NodeGene>,
    alleNoderArc: Vec<Arc<NodeGene>>,
    alleVekter: Vec<WeightGene>,
    // hidden_layers: Vec<Vec<&'a NodeGene>>,
    hidden_nodes: Vec<Vec<Arc<NodeGene>>>,
    input_layer: Vec<Arc<NodeGene>>,
    output_layer: Vec<Arc<NodeGene>>,
    // &'a to promise compiler that it lives the same length
    // weights_per_destination_node : HashMap<&'a NodeGene, Vec<&'a WeightGene>>,
    // weights_per_destination_node: HashMap<i32, Vec<WeightGene>>,
    weights_per_destination_node: HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>>,
    node_to_layer: HashMap<Arc<NodeGene>, i32>,
    layers_ordered_output_to_input: Vec<Vec<Arc<NodeGene>>>,
}

impl PhenotypeNeuralNetwork {
    pub fn decide_on_action(&mut self, input_values: Vec<f32>) -> Vec<f32> {
        let mut clamped_input_values = Vec::new();
        clamped_input_values.reserve(input_values.len());
        for value in input_values {
            //     println!("raw input values {:?}", node);
            clamped_input_values.push(value / PIXELS_PER_METER);
            // println!("new clamped input value {:?}", node);
        }
        // todo clamp x = x / max X = x -  (window_with/2)   ...... not very scalable....

        // how to use
        for i in 0..clamped_input_values.len() {
            let mut verdi = self.input_layer[i].value.write().unwrap();
            let bias = self.input_layer[i].bias.read().unwrap();
            *verdi = *verdi + *bias;
            // self.input_layer[i].value = clamped_input_values[i] + self.input_layer[i].bias;
        }

        // for mut destination_node in self.output_layer.iter_mut() {
        for mut destination_node in self.alleNoderArc.iter_mut() {
            // let relevant_weigh_nodes : Vec<&WeightGene> =  self.genome.weight_genes.iter().filter(  | weight_gene: &&WeightGene | weight_gene.destinationsnode == node.innovation_number  ).collect::<Vec<&WeightGene>>();   // bruk nodene istedenfor en vektor, slik at jeg vet hvilke vekter jeg skal bruke. Alt 2, sett opp nettet som bare vek først. Men det virker litt værre.
            // let relevant_weigh_nodes : Vec<WeightGene> =  self.weights_per_destination_node.get(node); // todo, jeg må bruke key ref som jeg orginalt brukte. Altså node. Men om jeg borrower node inn i phenotypelayer
            // let relevant_weigh_nodes = self.weights_per_destination_node.get(destination_node);
            let relevant_weigh_nodes = match self.weights_per_destination_node.get(destination_node) {
                Some(weights) => weights,
                None => &Vec::new()
            };


            let mut alle_inputs_til_destination_node: Vec<f32> = Vec::new();
            for weight_node in relevant_weigh_nodes.iter() {
                {
                    let kildenode = &weight_node.kildenode;
                    let kildenode_verdi = kildenode.value.read().unwrap();
                    let kildenode_påvirkning = *kildenode_verdi * weight_node.value;
                    alle_inputs_til_destination_node.push(kildenode_påvirkning);
                }
                // for x in self.input_layer.iter() {
                //     // if x.innovation_number == weight_node.kildenode {
                //     if *x == weight_node.kildenode {  // todo ikke sikekrt
                //         acc_value += x.value * weight_node.value;
                //         break;
                //     }
                // };
            }
            // let kildenode : &NodeGene =  self.input_layer.iter().filter( | node_gene: &&NodeGene | weight_node.kildenode ==  node_gene.innovation_number ).collect();
            //  acc_value += kildenode.value * weight_node.value;
            // }
            // destination_node.value = acc_value + destination_node.bias;
            let total_påvirking: f32 = alle_inputs_til_destination_node.iter().sum();
            {
                let mut verdi = destination_node.value.write().unwrap();
                // println!("verdi før halvering {}", verdi);
                *verdi = *verdi * 0.5;  // Hvis ikke resetter alt til 0 hver hver gang, men istedenfor er akkumulativ, så kreves det en demper også for å ikke gå til uendelig.
                *verdi = *verdi + total_påvirking;  // ,
            }
        }
        // for node in self.output_layer.iter() {
        // println!("output nodes {:?}", node);
        // }
        // todo, not sure if this is good or not
        let mut expanded_output_values = Vec::new();
        clamped_input_values.reserve(self.output_layer.len());
        for node in self.output_layer.iter() {
            let verdi = node.value.read().unwrap();
            expanded_output_values.push(verdi.clone());
            // expanded_output_values.push(node.value * PIXELS_PER_METER);
            // println!("new expianded output value {:?}", node);
        }
        // return expanded_output_values[0];
        // println!("expanded_output_values {:?} " ,expanded_output_values);
        return expanded_output_values;
        // return self.output_layer[0].value;
        // return random::<f32>();
    }

    pub(crate) fn decide_on_action2(&self, input_values: Vec<f32>) -> Vec<f32> {

        // Normalisering
        // ikke helt sikker på hvordan jeg skal normalisere input verdier enda.
        let mut clamped_input_values = Vec::new();
        clamped_input_values.reserve(input_values.len());
        for value in input_values {
            //     println!("raw input values {:?}", node);
            // clamped_input_values.push(value / PIXELS_PER_METER);
            clamped_input_values.push(value / PIXELS_PER_METER);
            // println!("new clamped input value {:?}", node);
        }
        // dbg!(&clamped_input_values);

        // Feed/load in input values

        for i in 0..clamped_input_values.len() {
            let node = &self.input_layer[i];
            let mut verdi = node.value.write().unwrap();
            let bias = node.bias.read().unwrap();
            *verdi = *verdi + *bias;
            dbg!(&verdi);
            println!("&self.input_layer[i] addr {:p}",*node);
            dbg!(&node);
            // self.input_layer[i].value = clamped_input_values[i] + self.input_layer[i].bias;
        }
        // update all values
        // for i in (self.ant_layers..0) {
        // dbg!( &self.layers_ordered_output_to_input);

        for i in (0..self.ant_layers).rev() {
            // dbg!(i);
            for mut destination_node in &self.layers_ordered_output_to_input[i] {
                // dbg!(destination_node);
                self.absorber_inkommende_verdier_og_set_ny_verdi(destination_node);
                // dbg!(destination_node);
            }
        }

        // Read of output neurons  (always last layer)

        let mut output_values = Vec::new();
        for node in self.output_layer.iter() {
            let verdi = node.value.read().unwrap();
            output_values.push(verdi.clone());
            // dbg!(verdi.clone());
        }
        // burde kanksje normalisere output også....
        // dbg!(&output_values);
        return output_values;
    }

    fn absorber_inkommende_verdier_og_set_ny_verdi(&self, mut destination_node: &Arc<NodeGene>) {
        let relevant_weights = match self.weights_per_destination_node.get(destination_node) {
            Some(weights) => weights,
            None => &Vec::new()
        };
        // dbg!(&destination_node);
        // dbg!(&relevant_weights);
        // dbg!(&self.weights_per_destination_node);
        let mut alle_inputs_til_destination_node: Vec<f32> = Vec::new();
        for weight in relevant_weights.iter() {
            {
                let kildenode = &weight.kildenode;
                let kildenode_verdi = kildenode.value.read().unwrap();
                let kildenode_påvirkning = *kildenode_verdi * weight.value;
                alle_inputs_til_destination_node.push(kildenode_påvirkning);
            }
        }
        let total_påvirking: f32 = alle_inputs_til_destination_node.iter().sum();
        // dbg!(&total_påvirking);
        {
            let mut verdi = destination_node.value.write().unwrap();
            // println!("verdi før halvering {}", verdi);
            *verdi = *verdi * 0.5;  // Hvis ikke resetter alt til 0 hver hver gang, men istedenfor er akkumulativ, så kreves det en demper også for å ikke gå til uendelig.
            *verdi = *verdi + total_påvirking;
        }
    }

    pub(crate) fn new(genome: &Genome) -> Self {
        // Kunne vært refferanse, men det ville introdusert mange lifetime annontasjoner.
        // Kan være vært å endre netverk til å være reffs senere, men ikke nå. Med et evo-devo arkitektur-netverk så er hovednettverket direkte koblet til genene uansett.
        // Med evo-devo påvirking fra mijøet i løpet av levetiden til individet, så kan det være at jeg kommer tilbake til dette

        let mut alleNoderArc: Vec<Arc<NodeGene>> = Vec::new();
        let mut alleVekter: Vec<WeightGene> = genome.weight_genes.clone();

        let weights_per_desination_node = genome.få_vekter_per_destinasjonskode();


        let (node_to_layer, layers_ordered_output_to_input) = PhenotypeNeuralNetwork::lag_lag_av_nevroner_sortert_fra_output(genome, &weights_per_desination_node);

        // WE DONT flip it around to be sorted so that output nodes are last . No need. just fill in input node values and iterate         for i in (ant_layers..0){

        let mut input_layer: Vec<Arc<NodeGene>> = Vec::new();
        let mut output_layer: Vec<Arc<NodeGene>> = Vec::new();

        for nodeArc in genome.node_genes.iter() {
            // let  nodeArc = Arc::new(node);
            if nodeArc.inputnode {
                input_layer.push(Arc::clone(nodeArc))
            } else if nodeArc.outputnode { output_layer.push(Arc::clone(nodeArc)); }
        }

        /* `PhenotypeNeuralNetwork` value */
        PhenotypeNeuralNetwork {
            ant_layers: layers_ordered_output_to_input.len(),
            // alleNoder: alleNoder,
            alleNoderArc: alleNoderArc,
            alleVekter: alleVekter,
            hidden_nodes: vec![],
            input_layer: input_layer,
            output_layer: output_layer,
            // weights_per_destination_node: weights_per_desination_node,
            weights_per_destination_node: weights_per_desination_node,
            node_to_layer,
            layers_ordered_output_to_input,
        }
    }

    pub(crate) fn lag_lag_av_nevroner_sortert_fra_output(
        genome: &Genome,
        // weights_per_desination_node: &HashMap<Arc<NodeGene>, Vec<&WeightGene>>)
        weights_per_desination_node: &HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>>)
        -> (HashMap<Arc<NodeGene>, i32>, Vec<Vec<Arc<NodeGene>>>) {
        let output_nodes: Vec<Arc<NodeGene>> = genome.node_genes.clone().iter().filter(|node| node.outputnode).map(|node| Arc::clone(node)).collect();

        // Start on input, and look at what connects.  STARTER PÅ OUTPUT OG BEVEGEWR OSS MOT INPUT
        // Starter på output for å bare inkludere noder og vekter som faktisk påvirker utfallet
        let mut node_to_layer = HashMap::new();
        output_nodes.iter().for_each(|node| { node_to_layer.insert(node.clone(), 0); });
        let mut layers_ordered_output_to_input: Vec<Vec<Arc<NodeGene>>> = vec![output_nodes];

        // dbg!(&node_to_layer);

        // I tilfeller vi har sykler, så vil vi hindre å evig flytte ting bakover i nettet. På et punkt så må vi bare godta en node kan få input som ikke er fra "venstre side". Bygger opp fra høyre side med outputs og jobber oss mot venstre.
        // Dette er løst ved å kun flytte en node en gang per vekt. (dette vil gjøre at sykluser kan gi hidden noder som er til venstre for input noder).
        // Merk at syklus noder vil gjøre litt ekstra forsterkning av sine verdier i forhold til andre vanlige hidden noder om de er til venstre for input noder.  Disse vil "ta inn nåtid data + sin fortid data og gi ut begge"
        let mut node_to_vekt_som_flyttet_på_noden: HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>> = HashMap::new();
        // let mut node_to_vekt_som_flyttet_på_noden: HashMap<Arc<NodeGene>, Vec<&WeightGene>> = HashMap::new();
        // let next_layer = få_neste_lag(&weights_per_desination_node, &mut node_to_layer, &mut layers_ordered_output_to_input, &mut node_to_vekt_som_flyttet_på_noden, 1);
        // layers_ordered_output_to_input.push(next_layer);

        let mut layer_index = 1;
        loop {
            // dbg!(&layer_index);
            let next_layer = PhenotypeNeuralNetwork::få_neste_lag(&weights_per_desination_node, &mut node_to_layer, &mut layers_ordered_output_to_input, &mut node_to_vekt_som_flyttet_på_noden, layer_index);
            layer_index += 1;
            // dbg!(&next_layer);
            // dbg!(&next_layer.len());
            if next_layer.len() == 0 { break; }
            layers_ordered_output_to_input.push(next_layer);
            // break;
        }
        // dbg!(&layers_ordered_output_to_input);

        (node_to_layer, layers_ordered_output_to_input)
    }

    fn få_neste_lag<'a>(
        // weights_per_desination_node: &HashMap<Arc<NodeGene>, Vec<&'a WeightGene>>,
        weights_per_desination_node: &HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>>,
        layer_per_node: &mut HashMap<Arc<NodeGene>, i32>,
        layers_output_to_input: &mut Vec<Vec<Arc<NodeGene>>>,
        // node_to_vekt_som_flyttet_på_noden: &mut HashMap<Arc<NodeGene>, Vec<&'a WeightGene>>,
        node_to_vekt_som_flyttet_på_noden: &mut HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>>,
        lag_index: i32,
    ) -> Vec<Arc<NodeGene>> {
        let mut next_layer = vec![];
        // for node in input_layer.iter() {
        // for node in layers_output_to_input.iter().last().unwrap() {
        // let last_layer = layers_output_to_input.iter().last();
        // for node in layers_output_to_input.last().iter() {
        // dbg!(&lag_index);
        // dbg!(&weights_per_desination_node);
        // dbg!(&layers_output_to_input);

        for node in layers_output_to_input.last().unwrap() {
            // for node in layers_output_to_input.iter().last().iter() {
            //     let node2 : &Arc<NodeGene> = *node;
            //     for weight in weights_per_desination_node.get(&Arc::clone(node)).expect("burde eksistere") {
            let mut vekter_allerede_brukt = match node_to_vekt_som_flyttet_på_noden.get_mut(&Arc::clone(node)) {
                None => { Vec::new() }
                Some(liste) => { liste.clone() }
            };
            // dbg!(&node);
            match weights_per_desination_node.get(node) {
                Some(weights) => {
                    for weight in weights {
                        if !vekter_allerede_brukt.contains(weight) {
                            // It can be multiple weights that would have added the same node to the layer, but we only want one arc refference to the node. But we want vekter allerede brukt to be updated
                            if !next_layer.contains(&weight.kildenode) {
                                next_layer.push(Arc::clone(&weight.kildenode));
                            }
                            layer_per_node.insert(Arc::clone(&weight.kildenode), lag_index);
                            vekter_allerede_brukt.push(Arc::clone(weight) );
                        }
                    }
                }
                _ => {}
            };
            node_to_vekt_som_flyttet_på_noden.insert(Arc::clone(node), vekter_allerede_brukt);
        }
        next_layer
    }
}

// todo bytte ut med PhenotypeNeuralNetwork

pub fn create_phenotype_layers(genome: Genome) -> (PhenotypeNeuralNetwork) { // todo kanksje bare bytt ut med det jeg gjør for tengning

    // PhenotypeLayers holder på en kopi av nodene og vekter i genomet, og

    // Holder på objektene
    let mut alleNoder: Vec<NodeGene> = Vec::new();
    let mut alleNoderArc: Vec<Arc<NodeGene>> = Vec::new();
    let mut alleVekter: Vec<WeightGene> = genome.weight_genes.clone();

    // let mut input_layer2: Vec<&NodeGene> = Vec::new();
    // let mut output_layer2: Vec<&NodeGene> = Vec::new();
    let mut input_layer2: Vec<Arc<NodeGene>> = Vec::new();
    let mut output_layer2: Vec<Arc<NodeGene>> = Vec::new();

    for nodeArc in genome.node_genes {
        // let  nodeArc = Arc::new(node);
        if nodeArc.inputnode { input_layer2.push(Arc::clone(&nodeArc)) } else if nodeArc.outputnode { output_layer2.push(Arc::clone(&nodeArc)); }
        alleNoderArc.push(nodeArc);
    }

    // let mut weights_per_desination_node: HashMap<&NodeGene, Vec<&WeightGene>> = HashMap::new();
    // // weights_per_desination_node.reserve(alleNoder.clone().len());
    // for weight in alleVekter.iter() {
    //     let list = weights_per_desination_node.entry(weight.destinationsnode).or_insert_with(|| Vec::new());
    //     list.push(weight);
    // }
    let mut weights_per_desination_node: HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>> = HashMap::new();
    let weightRcs = genome.weight_genes.into_iter().map(|weight| Arc::new(weight));

    // for weight in genome.weight_genes.iter() {
    for weight in weightRcs {
        // let list = weights_per_desination_node.entry(&*weight.destination_node).or_insert_with(|| Vec::new());
        let list = weights_per_desination_node.entry(Arc::clone(&weight.destinasjonsnode)).or_insert_with(|| Vec::new());
        // list.push(Rc::clone(weight));
        list.push(Arc::clone(&weight));
        // list.push(Rc::new(*weight));
        // list.push(&weight);
    }
    // println!("vekter_gruppert {:?}", weights_per_desination_node);
    // for node in uplassertHidden.iter() {

    // println!("weights_per_destination_node {:#?}", weights_per_desination_node.clone());
    let layers = PhenotypeNeuralNetwork {
        ant_layers: 2,
        // alleNoder: alleNoder,
        alleNoderArc: alleNoderArc,
        alleVekter: alleVekter,
        input_layer: input_layer2,
        output_layer: output_layer2,
        weights_per_destination_node: weights_per_desination_node,
        node_to_layer: Default::default(),
        hidden_nodes: vec![],

        layers_ordered_output_to_input: vec![],
    };
    // println!("output nodes {:?}", layers.output_layer.iter().map( | node_gene: NodeGene | node_gene ));
    // return (layers , genome);
    return layers;
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