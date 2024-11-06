use std::cmp::{max, min, Ordering, PartialEq};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hasher;
use std::io::Write;
use std::vec::Vec;
use avian2d::math::{AdjustPrecision, Vector};
use avian2d::prelude::{CollisionLayers, LayerMask};
use bevy::asset::AsyncWriteExt;
// use bevy::asset::io::memory::Value::Vec;
use bevy::color::palettes::basic::PURPLE;
use bevy::prelude::*;
use bevy::prelude::KeyCode::{KeyE, KeyK, KeyP, KeyR, KeyT};
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::sprite::MaterialMesh2dBundle;
use bevy_inspector_egui::egui::emath::Numeric;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use bevy_rapier2d::na::DimAdd;
// use bevy_rapier2d::prelude::*;
use avian2d::prelude::*;
use bevy::ecs::query::QueryIter;
use lazy_static::lazy_static;
use rand::{random, Rng, thread_rng};
use rand::distributions::Uniform;
use rand::seq::SliceRandom;
use crate::environments::moving_plank::{create_plank_env_falling, create_plank_env_moving_right, create_plank_ext_force_env_falling, MovingPlankPlugin, PIXELS_PER_METER, PLANK_HIGHT, PLANK_LENGTH};
use crate::environments::simulation_teller::{SimulationGenerationTimer, SimulationRunningTellerPlugin};

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
        // .add_plugins(WorldInspectorPlugin::new())
        .insert_state(EttHakkState::DISABLED)
        .init_resource::<GenerationCounter>()
        // .init_resource(     EnvValg { Homing} )
        .add_event::<ResetToStartPositionsEvent>()
        .add_systems(Startup, (
            setup_camera,
            spawn_start_population,
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
                label_plank_with_current_score,
                agent_action.run_if(in_state(Kjøretilstand::Kjørende)),
            ).chain(),
            check_if_done,
            // check_if_done.run_if(every_time_if_stop_on_right_window()),
            (
                print_pop_conditions,
                increase_generation_counter,
                lock_mutation_stability,
                save_best_to_history,
                kill_worst_individuals,
                create_new_children,
                spawn_a_random_new_individual2,
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
fn get_population_sorted_from_best_to_worst_v2<'lifetime_a>(query: QueryIter<'lifetime_a, '_, (Entity, &PlankPhenotype, &Genome), ()>) -> Vec<PhentypeGenome<'lifetime_a>> {
    // fn get_population_sorted_from_best_to_worst<'a>(query: Query<&'a PlankPhenotype>) -> Vec<&'a PlankPhenotype> {
    let mut population = Vec::new();
    //sort_individuals
    for (entity, plank, genome) in query {
        // for (plank) in query {
        // population.push(plank)
        population.push(PhentypeGenome { phenotype: plank, genome: genome, entity_index: entity.index(), entity_bevy_generation: entity.generation() });
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

static START_POPULATION_SIZE: i32 = 20;

fn spawn_start_population(mut commands: Commands,
                          mut meshes: ResMut<Assets<Mesh>>,
                          mut materials: ResMut<Assets<ColorMaterial>>, ) {
    for n in 0i32..START_POPULATION_SIZE {
        // for n in 0i32..1 {

        spawn_a_random_new_individual(&mut commands, &mut meshes, &mut materials, n);
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
) {
    let n: i32 = 1;
    spawn_a_random_new_individual(&mut commands, &mut meshes, &mut materials, n)
}

fn spawn_a_random_new_individual(commands: &mut Commands,
                                 meshes: &mut ResMut<Assets<Mesh>>,
                                 materials: &mut ResMut<Assets<ColorMaterial>>,
                                 n: i32,
) {
    let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(PLANK_LENGTH, PLANK_HIGHT));
    let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE));
    // println!("Har jeg klart å lage en genome fra entity = : {}", genome2.allowed_to_change);
    let text_style = TextStyle {
        font_size: 20.0,
        color: Color::WHITE,
        ..default()
    };
    let text_justification = JustifyText::Center;

    match ACTIVE_ENVIROMENT {
        EnvValg::Høyre => commands.spawn(create_plank_env_moving_right(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + n as f32 * 50.0, z: 1.0 }, new_random_genome(2, 2))),
        EnvValg::Fall | EnvValg::FallVelocityHøyre => commands.spawn(create_plank_env_falling(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + (n as f32 * 15.0), z: 1.0 }, new_random_genome(2, 2))),
        EnvValg::FallExternalForcesHøyre | EnvValg::Homing | EnvValg::HomingGroud => { commands.spawn(create_plank_ext_force_env_falling(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + 3.3 * 50.0, z: 1.0 }, new_random_genome(2, 2))) }
    }
        .with_children(|builder| {
            builder.spawn((
                Text2dBundle {
                    text: Text::from_section("translation", text_style.clone())
                        .with_justify(text_justification),
                    transform: Transform::from_xyz(0.0, 0.0, 2.0),
                    ..default()
                },
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
) {
    if key_input.just_pressed(KeyT) {
        for (entity) in query.iter() {
            commands.entity(entity).despawn();
        }
        spawn_start_population(commands, meshes, materials)
    }
}

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
    let number_of_individuals_to_kill = max(1, population.len() - number_of_individuals_to_leave_alive);
    // println!("killing of {} entities", number_of_individuals_to_kill);
    for (entity, _) in &population[0..number_of_individuals_to_kill] {
        // println!("despawning entity {} ", entity.index());
        commands.entity(*entity).despawn_recursive();
    }
}

#[derive(Clone)]
struct PhentypeGenome<'lifetime_a> {
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
        population.push(PhentypeGenome { phenotype: plank, genome: genome, entity_index: entity.index(), entity_bevy_generation: entity.generation() })
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
        let parent: &PhentypeGenome = parents.choose(&mut thread_random).expect("No potential parents :O !?");
        // println!("the lucky winner was parent with entity index {}, that had score {} ", parent.entity_index, parent.phenotype.score);
        let mut new_genome: Genome = parent.genome.clone();

        // NB: mutation is done in a seperate bevy system
        new_genome.allowed_to_change = true;

        let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(PLANK_LENGTH, PLANK_HIGHT));
        let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE).with_alpha(0.5));

        let text_style = TextStyle {
            font_size: 20.0,
            color: Color::WHITE,
            ..default()
        };
        let text_justification = JustifyText::Center;

        match ACTIVE_ENVIROMENT {
            EnvValg::Fall | EnvValg::FallVelocityHøyre => commands.spawn(create_plank_env_falling(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + 3.3 * 50.0, z: 1.0 }, new_genome)),
            EnvValg::Høyre => commands.spawn(create_plank_env_moving_right(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + 3.3 * 50.0, z: 1.0 }, new_genome)),
            EnvValg::FallExternalForcesHøyre | EnvValg::Homing | EnvValg::HomingGroud => {
                commands.spawn(create_plank_ext_force_env_falling(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + 3.3 * 50.0, z: 0.0 }, new_genome))
            }
        }
            .with_children(|builder| {
                builder.spawn((
                    Text2dBundle {
                        text: Text::from_section("translation", text_style.clone())
                            .with_justify(text_justification),
                        transform: Transform::from_xyz(0.0, 0.0, 2.0),
                        ..default()
                    },
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

static ACTIVE_ENVIROMENT: EnvValg = EnvValg::HomingGroud;

lazy_static! {
     static ref LANDING_SITE_PER_ENVIRONMENT:HashMap<EnvValg ,Vec2 > = {
 HashMap::from([
    ( EnvValg::Homing, Vec2 { x: 100.0, y: -100.0 }),
    ( EnvValg::HomingGroud, Vec2 { x: 00.0, y: GROUND_STARTING_POSITION.y + GROUND_HEIGHT }),
    ])
    };
    static ref LANDING_SITE: Vec2 = LANDING_SITE_PER_ENVIRONMENT[&ACTIVE_ENVIROMENT];
}

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
    query: Query<(&mut Transform, &mut crate::PlankPhenotype, &mut LinearVelocity, Option<&mut ExternalForce>), ( With<crate::PlankPhenotype>)>,
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


fn reset_to_star_pos(mut query: Query<(&mut Transform, &mut PlankPhenotype, &mut LinearVelocity, Option<&mut ExternalForce>), ( With<PlankPhenotype>)>) {
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
fn agent_action(
    mut query: Query<(&mut Transform, &mut PlankPhenotype, &mut LinearVelocity, Option<&mut ExternalForce>, Entity), ( With<PlankPhenotype>)>,
    time: Res<Time>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_seconds_f64().adjust_precision();

    println!();
    println!();
    println!();

    for (mut transform, mut plank, mut velocity, option_force, entity) in query.iter_mut() {
        plank.obseravations = vec![transform.translation.x.clone(), transform.translation.y.clone()];

        // let input_values = vec![1.0, 2.0]; // 2 inputs
        // let input_values = vec![individual.translation.x.clone() * 0.002, individual.translation.y.clone()* 0.002]; // 2 inputs
        let input_values = plank.obseravations.clone();
        let action = plank.phenotype_layers.decide_on_action(input_values);            // fungerer
        // let action = plank.phenotype_layers.decide_on_action(  plank.obseravations.clone() );  // fungerer ikke ?!?!

        // individual.translation.x += random::<f32>() * action * 5.0;
        // println!("action : {action}");
        let mut a = option_force.expect("did not have forces on individ!!? :( ");
        match ACTIVE_ENVIROMENT {
            EnvValg::Høyre | EnvValg::Fall => transform.translation.x += action[0] * 2.0,
            EnvValg::FallVelocityHøyre => velocity.0.x += action[0],
            // EnvValg::FallGlideBomb => velocity.0 += action,
            // EnvValg::FallExternalForcesHøyre => option_force.expect("did not have forces on individ!!? :( ").x = action,
            EnvValg::FallExternalForcesHøyre | EnvValg::Homing  | EnvValg::HomingGroud => {
                // a.x = 100.0 * action[0] * delta_time;
                // a.y = 100.0 * action[1] * delta_time;
                a.x = 1.0 * action[0];
                a.y = 1.0 * action[1];

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
            EnvValg::Homing  | EnvValg::HomingGroud => {
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
    mut query: Query<(&mut Text, &Parent), With<IndividLabelText>>,
    parent_query: Query<&PlankPhenotype>,
) {
    for (mut tekst, parent_entity) in query.iter_mut() {
        if let Ok(plank_phenotype) = parent_query.get(**parent_entity) {
            tekst.sections[0].value = plank_phenotype.score.to_string();
        }
    }
}


#[derive(Component, Debug, )]
// #[derive(Component, Eq, Ord, PartialEq, PartialOrd, PartialEq)]
pub struct PlankPhenotype {
    pub score: f32,
    pub obseravations: Vec<f32>,
    // pub phenotype: f32,
    phenotype_layers: PhenotypeLayers, // for now we always have a neural network to make decisions for the agent
    // pub genotype: Genome,
    // Genome er flyttet til å bli en component på Entity som også holder på PlankPhenotype komponent. Mistenker det fungerer bedre med tanke på bevy
    // pub genotype: Entity, // by having genotype also be an entity, we can query for it directly, without going down through parenkt PlankPhenotype that carries the genome ( phenotype is not that relevant if we do mutation or other pure genotype operations)
}


#[derive(Component, Debug)]
// #[derive(Component, Eq, Ord, PartialEq, PartialOrd, PartialEq)]
pub struct Individ {}

#[derive(Debug, Component, Copy, Clone)] // todo spesifisker eq uten f32 verdiene
pub struct NodeGene {
    innovation_number: i32,
    bias: f32,
    enabled: bool,
    inputnode: bool,
    outputnode: bool,
    mutation_stability: f32, // 1 is compleat lock/static genome. 0 is a mutation for all genes
    layer: usize,

    value: f32, // mulig denne blir flyttet til sin egen Node struct som brukes i nettverket, for å skille fra gen.
}
impl std::cmp::PartialEq for NodeGene {
    fn eq(&self, other: &Self) -> bool {
        self.innovation_number == other.innovation_number
    }
}


impl Eq for NodeGene {}
//
impl std::hash::Hash for NodeGene {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.innovation_number.hash(state);
    }
}

#[derive(Debug)]
struct PhenotypeLayers {
    ant_layers: usize,
    hidden_layers: Vec<Vec<NodeGene>>,
    input_layer: Vec<NodeGene>,
    output_layer: Vec<NodeGene>,
    // &'a to promise compiler that it lives the same length
    // weights_per_destination_node : HashMap<&'a NodeGene, Vec<&'a WeightGene>>,
    // weights_per_destination_node: HashMap<i32, Vec<WeightGene>>,
    weights_per_destination_node: HashMap<NodeGene, Vec<WeightGene>>,
}

impl PhenotypeLayers {
    pub fn decide_on_action(&mut self, input_values: Vec<f32>) -> Vec<f32> {
        let mut clamped_input_values = Vec::new();
        clamped_input_values.reserve(input_values.len());
        for node in input_values {
            //     println!("raw input values {:?}", node);
            clamped_input_values.push(node / PIXELS_PER_METER);
            // println!("new clamped input value {:?}", node);
        }

        // todo clamp x = x / max X = x -  (window_with/2)   ...... not very scalable....

        // how to use
        for i in 0..clamped_input_values.len() {
            self.input_layer[i].value = clamped_input_values[i] + self.input_layer[i].bias;
        }

        for mut node in self.output_layer.iter_mut() {
            // let relevant_weigh_nodes : Vec<&WeightGene> =  self.genome.weight_genes.iter().filter(  | weight_gene: &&WeightGene | weight_gene.destinationsnode == node.innovation_number  ).collect::<Vec<&WeightGene>>();   // bruk nodene istedenfor en vektor, slik at jeg vet hvilke vekter jeg skal bruke. Alt 2, sett opp nettet som bare vek først. Men det virker litt værre.
            // let relevant_weigh_nodes : Vec<WeightGene> =  self.weights_per_destination_node.get(node); // todo, jeg må bruke key ref som jeg orginalt brukte. Altså node. Men om jeg borrower node inn i phenotypelayer
            let relevant_weigh_nodes = self.weights_per_destination_node.get(node).expect("burde være her");
            // let relevant_weigh_nodes = match self.weights_per_destination_node.get(node) {
            //     Some(weights) => weights,
            //     None => &Vec::new()
            // };


            let mut acc_value = 0.0;
            for weight_node in relevant_weigh_nodes.iter() {
                let mut kildenode: NodeGene;
                for x in self.input_layer.iter() {
                    if x.innovation_number == weight_node.kildenode {
                        acc_value += x.value * weight_node.value;
                        break;
                    }
                };
            }
            // let kildenode : &NodeGene =  self.input_layer.iter().filter( | node_gene: &&NodeGene | weight_node.kildenode ==  node_gene.innovation_number ).collect();
            //  acc_value += kildenode.value * weight_node.value;
            // }
            node.value = acc_value + node.bias;
        }


        // for node in self.output_layer.iter() {
        // println!("output nodes {:?}", node);
        // }

        // todo, not sure if this is good or not
        let mut expanded_output_values = Vec::new();
        clamped_input_values.reserve(self.output_layer.len());
        for node in self.output_layer.iter() {
            expanded_output_values.push(node.value * PIXELS_PER_METER);
            // println!("new expianded output value {:?}", node);
        }
        // return expanded_output_values[0];
        return expanded_output_values;
        // return self.output_layer[0].value;
        // return random::<f32>();
    }
}

#[derive(Debug, Clone)]
pub struct WeightGene {
    innovation_number: i32,
    value: f32,
    enabled: bool,
    kildenode: i32,
    destinationsnode: i32,
    mutation_stability: f32,
}

#[derive(Debug, Component, Clone)]
struct Genome {
    // nodeGene can not be queried, since genome is a compnent and not an Entity. (It can be changed, but I feel like it is acceptable to give the entire genome to the bevy system

    // kan også kanskje vurdere å bruke bevy_hirearky for å operere på agenene idividuelt, istedenfor å altid gå via Genom parent
    pub node_genes: Vec<NodeGene>,
    pub weight_genes: Vec<WeightGene>,
    pub original_ancestor_id: usize,
    pub allowed_to_change: bool, // Useful to not mutate best solution found/Elite
}


// skal layers absorbere genome, skal den returnere genome og layers, eller skal den ta inn en copy av genome?
// trenger vi genome senere etter env ? Ja.
// Prøver å returenre begge


// fn create_phenotype_layers (genome: &Genome) -> (PhenotypeLayers, &Genome) {

// alt 2 tar inn en klone
pub fn create_phenotype_layers(genome: Genome) -> (PhenotypeLayers) {

    // for now just connect input output directly, and ignore hidden

    // let mut input_layer2 : Vec<&NodeGene>= Vec::new();
    // let mut  output_layer2: Vec<&NodeGene> = Vec::new();

    let mut input_layer2: Vec<NodeGene> = Vec::new();
    let mut output_layer2: Vec<NodeGene> = Vec::new();
    // let mut weights_per_destination_node : HashMap<usize, Vec<WeightGene>>  = HashMap::new();
    let mut weights_per_destination_node: HashMap<NodeGene, Vec<WeightGene>> = HashMap::new();

    weights_per_destination_node.reserve(genome.node_genes.clone().len());
    // for node in genome.node_genes.iter(){
    for node in genome.node_genes {
        if node.outputnode {
            output_layer2.push(node);
        } else if node.inputnode { input_layer2.push(node) }
    }

    // let input_layer = genome.node_genes.iter().filter( |node_gene: &&NodeGene | node_gene.inputnode ).collect();
    // let output_layer = genome.node_genes.iter().filter( |node_gene: &&NodeGene | node_gene.outputnode ).collect();
    // let mut layers = PhenotypeLayers { ant_layers: 2 , hidden_layers : Vec::new(), input_layer, output_layer };

    // println!("output layer {:?}", output_layer2);
    let weights_genes = genome.weight_genes.clone();  //  todo jeg har ingen weight genes!!!
    // println!("weights_genes  {:?}", weights_genes.clone());
    for node in output_layer2.iter() {
        // let relevant_weigh_nodes: Vec<&WeightGene> = genome.weight_genes.iter().filter(|weight_gene: &&WeightGene| weight_gene.destinationsnode == node.innovation_number).collect::<Vec<&WeightGene>>();   // bruk nodene istedenfor en vektor, slik at jeg vet hvilke vekter jeg skal bruke. Alt 2, sett opp nettet som bare vek først. Men det virker litt værre.
        for weight_gene in weights_genes.clone() {
            // if weights_per_destination_node.contains_key(&weight_gene.destinationsnode) {
            if weights_per_destination_node.contains_key(node) {
                // weights_per_destination_node.get_mut(&weight_gene.destinationsnode.clone()).expect("REASON").push(weight_gene);
                weights_per_destination_node.get_mut(node).expect("REASON").push(weight_gene);
                // https://stackoverflow.com/questions/32300132/why-cant-i-store-a-value-and-a-reference-to-that-value-in-the-same-struct
            } else {
                // weights_per_destination_node.insert(weight_gene.destinationsnode.clone(), vec![weight_gene]);
                weights_per_destination_node.insert(*node, vec![weight_gene]);
            }
            // .iter().filter(|weight_gene: &&WeightGene| weight_gene.destinationsnode == node.innovation_number).collect::<Vec<WeightGene>>();   // bruk nodene istedenfor en vektor, slik at jeg vet hvilke vekter jeg skal bruke. Alt 2, sett opp nettet som bare vek først. Men det virker litt værre.
            // weights_per_destination_node.insert(node.innovation_number, relevant_weigh_nodes);
        }
    }


    // println!("weights_per_destination_node {:#?}", weights_per_destination_node.clone());
    let layers = PhenotypeLayers { ant_layers: 2, hidden_layers: Vec::new(), input_layer: input_layer2, output_layer: output_layer2, weights_per_destination_node: weights_per_destination_node };


    // println!("output nodes {:?}", layers.output_layer.iter().map( | node_gene: NodeGene | node_gene ));
    // return (layers , genome);
    return layers;
}

pub fn new_random_genome(ant_inputs: usize, ant_outputs: usize) -> Genome {
    let mut node_genes = Vec::new();
    let mut thread_random = thread_rng();
    let uniform_dist = Uniform::new(-1.0, 1.0);

    for n in 0..ant_inputs {
        node_genes.push(NodeGene {
            innovation_number: n as i32,
            bias: thread_random.sample(uniform_dist),
            enabled: true,
            inputnode: true,
            outputnode: false,
            mutation_stability: 0.0,
            layer: 0,
            value: 0.0,
        });
    }

    for n in 0..ant_outputs {
        node_genes.push(NodeGene {
            innovation_number: n as i32,
            bias: thread_random.sample(uniform_dist),
            enabled: true,
            inputnode: false,
            outputnode: true,
            mutation_stability: 0.0,
            layer: 0,
            value: 0.0,
        });
    }
    // start with no connections, start with fully connected, or random

    // fully connected input output
    let mut weight_genes = Vec::new();
    for n in 0..ant_inputs {
        for m in 0..ant_outputs {
            weight_genes.push(WeightGene {
                // kildenode : &node_genes[n],
                // destinationsnode: node_genes[m],
                kildenode: n as i32,
                destinationsnode: m as i32,
                innovation_number: 42,
                value: thread_random.sample(uniform_dist),
                enabled: true,
                mutation_stability: 0.0,
            })
        }
    }


    return Genome { node_genes: node_genes, weight_genes: weight_genes, original_ancestor_id: random(), allowed_to_change: true };
}

// lock and unlock mutation to lock parents/Elites. Still not decided if i want a 100% lock or allow some small genetic drift also in elites
fn lock_mutation_stability(mut genome_query: Query<&mut Genome>) {
    for mut genome in genome_query.iter_mut() {
        // for mut node_gene in genome.node_genes.iter_mut() {
        //     // node_gene.mutation_stability = 1.0
        // }
        // for mut weight_gene in genome.weight_genes.iter_mut() {
        // weight_gene.mutation_stability = 1.0
        // }
        genome.allowed_to_change = false;
    }
}

pub fn mutate_genomes(mut genes: Query<&mut Genome>) {
    for mut gene in genes.iter_mut() {
        // println!("mutating genome with original ancestor {}, if allowed: {} ", gene.original_ancestor_id, gene.allowed_to_change);
        if gene.allowed_to_change {
            mutate_existing_nodes(&mut gene.node_genes);
            mutate_existing_weights(&mut gene.weight_genes);
        }
    }
}
// Gets the Position component of all Entities whose Velocity has changed since the last run of the System
fn genome_changed_event_update_phenotype(query: Query<&PlankPhenotype, Changed<Genome>>) {
    for position in &query {}
}


pub fn mutate_existing_nodes(mut node_genes: &mut Vec<NodeGene>) {
    // println!("mutating {} nodes ", node_genes.iter().count());
    let mutation_strength = 2.0;
    for mut node_gene in node_genes.iter_mut() {
        if random::<f32>() > node_gene.mutation_stability {
            node_gene.bias += (random::<f32>() * 2.0 - 1.0) * mutation_strength;
            // node_gene.mutation_stability += random::<f32>() * 2.0 - 1.0;
            // enabling
        }
    }
}

pub fn mutate_existing_weights(mut weight_genes: &mut Vec<WeightGene>) {
    // println!("mutating {} weights ", weight_genes.iter().count());
    let mutation_strength = 2.0;

    for mut weight_gene in weight_genes.iter_mut() {
        // println!("weight gene mutation_stability : {}", weight_gene.mutation_stability);
        if random::<f32>() > weight_gene.mutation_stability {
            // println!("weight gene value before mutation: {}", weight_gene.value);
            weight_gene.value += (random::<f32>() * 2.0 - 1.0) * mutation_strength;
            // println!("weight gene value after mutation: {}", weight_gene.value);
            // weight_gene.mutation_stability += random::<f32>() * 2.0 - 1.0;
        }
        if random::<f32>() > weight_gene.mutation_stability {
            weight_gene.enabled = !weight_gene.enabled;
        }

        // evo devo eller hardkoded layer?
        if random::<f32>() > weight_gene.mutation_stability {
            weight_gene.enabled = !weight_gene.enabled;
        }
    }
}

const GROUND_LENGTH: f32 = 5495.;
const GROUND_HEIGHT: f32 = 10.;
const GROUND_COLOR: Color = Color::rgb(0.30, 0.75, 0.5);
const GROUND_STARTING_POSITION: Vec3 = Vec3 { x: 0.0, y: -300.0, z: 1.0 };

const ROOF_STARTING_POSITION: Vec3 = Vec3 { x: 0.0, y: 300.0, z: 1.0 };
// const GROUND_STARTING_POSITION: Vec3 = Vec3 { x: 0.0, y: -300.0, z: 1.0 };


fn spawn_ground(mut commands: Commands,
                mut meshes: ResMut<Assets<Mesh>>,
                mut materials: ResMut<Assets<ColorMaterial>>, ) {
    commands.spawn((
                       RigidBody::Static,
                       MaterialMesh2dBundle {
                           mesh: meshes.add(Rectangle::default()).into(),
                           material: materials.add(GROUND_COLOR),
                           transform: Transform::from_translation(GROUND_STARTING_POSITION)
                               .with_scale(Vec2 { x: GROUND_LENGTH, y: GROUND_HEIGHT }.extend(1.)),
                           ..default()
                       },
                       // Sleeping::disabled(),
                       Collider::rectangle(1.0, 1.0),
                       Restitution::new(0.0),
                       Friction::new(0.5),
                       CollisionLayers::new(0b0010, LayerMask::ALL),
                   ), );
}


fn spawn_landing_target(mut commands: Commands,
                        mut meshes: ResMut<Assets<Mesh>>,
                        mut materials: ResMut<Assets<ColorMaterial>>, ) {
    commands.spawn((
                       RigidBody::Static,
                       MaterialMesh2dBundle {
                           mesh: meshes.add(Circle::default()).into(),
                           material: materials.add(Color::linear_rgb(1.0, 0.0, 0.0)),
                           transform: Transform::from_translation(LANDING_SITE.extend(0.0))
                               .with_scale(Vec2 { x: 10.0, y: 10.0 }.extend(1.)),
                           ..default()
                       },
                       // Sleeping::disabled(),
                   ), );
}


fn spawn_roof(mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>, ) {
    commands.spawn((
                       RigidBody::Static,
                       MaterialMesh2dBundle {
                           mesh: meshes.add(Rectangle::default()).into(),
                           material: materials.add(GROUND_COLOR),
                           transform: Transform::from_translation(ROOF_STARTING_POSITION)
                               .with_scale(Vec2 { x: GROUND_LENGTH, y: 10.0 }.extend(1.)),
                           ..default()
                       },
                       // Sleeping::disabled(),
                       Collider::rectangle(1.0, 1.0),
                       Restitution::new(0.0),
                       Friction::new(0.5),
                       CollisionLayers::new(0b0010, LayerMask::ALL),
                   ), );
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