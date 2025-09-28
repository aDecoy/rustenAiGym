use crate::genome::genome_stuff::Genome;
use crate::{Kjøretilstand, PhentypeAndGenome, PlankPhenotype};
use bevy::asset::{Assets, Handle};
use bevy::color::Color;
use bevy::color::palettes::basic::RED;
use bevy::ecs::query::QueryIter;
use bevy::prelude::*;
use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Component)]
pub struct EliteTag;

pub struct ElitePlugin;

impl Plugin for ElitePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GenerationCounter>()
            .add_systems(PostStartup, ((color_elite_red, add_elite_component_tag_to_best_individ))) // Litt random første runde :)
            .add_systems(
                Update,
                (
                    (add_elite_component_tag_to_best_individ, color_elite_red)
                        .chain()
                        .run_if(in_state(Kjøretilstand::EvolutionOverhead)),
                    (
                        increase_generation_counter,
                        // print_pop_conditions,
                    )
                        .chain()
                        .run_if(in_state(Kjøretilstand::EvolutionOverhead)),
                ),
            );
    }
}

pub fn get_best_elite<'a>(iteratior: QueryIter<'a, '_, (Entity, &PlankPhenotype, &'_ Genome), With<PlankPhenotype>>) -> PhentypeAndGenome<'a> {
    let mut population = Vec::new();
    //sort_individuals
    for (entity, plank, genome) in iteratior {
        population.push(PhentypeAndGenome {
            phenotype: plank,
            genome: genome,
            entity_index: entity.index(),
            entity: entity,
            entity_bevy_generation: entity.generation(),
        })
    }
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
    let elite = population.get(0).unwrap();
    return elite.clone();
    return PhentypeAndGenome {
        genome: elite.genome,
        entity_index: elite.entity_index,
        entity: elite.entity,
        phenotype: elite.phenotype,
        entity_bevy_generation: elite.entity_bevy_generation,
    };
    return elite.clone(); // KAN DET VÆRE AT DETTE LAGER EN NY GENOME`?
}
// fn get_population_sorted_from_best_to_worst<'lifetime_a>(query: QueryIter<'lifetime_a, '_, &crate::PlankPhenotype, ()>) -> Vec<&'lifetime_a crate::PlankPhenotype> {

pub(crate) fn add_elite_component_tag_to_best_individ(
    mut commands: Commands,
    query: Query<(Entity, &PlankPhenotype, &Genome), With<PlankPhenotype>>,
    old_elite_query: Query<(Entity), With<EliteTag>>,
) {
    // remove old one
    if let Ok(old_elite_entity) = old_elite_query.single() {
        commands.entity(old_elite_entity).remove::<EliteTag>();
    }
    // find new elite
    if (query.iter().len() != 0) {
        println!("Added elite tag to best individual amongst population of size {}", query.iter().len());
        let elite = get_best_elite(query.iter());
        commands.entity(elite.entity).insert((EliteTag));
    } else {
        println!("Fant ingen elite, av population av størrelse {}", query.iter().len());
    }
}

fn color_elite_red(mut commands: Commands, mut elite_query: Query<Entity, With<EliteTag>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    if let Ok(elite_entity) = elite_query.get_single() {
        let elite_material_handle: Handle<ColorMaterial> = materials.add(Color::from(RED));
        commands.entity(elite_entity).insert(MeshMaterial2d(elite_material_handle));
    }
}

fn sort_best_to_worst<'a>(iteratior: QueryIter<'a, '_, (Entity, &PlankPhenotype, &'_ Genome), With<PlankPhenotype>>) -> Vec<PhentypeAndGenome<'a>> {
    let mut population = Vec::new();
    //sort_individuals
    for (entity, plank, genome) in iteratior {
        population.push(PhentypeAndGenome {
            phenotype: plank,
            genome: genome,
            entity_index: entity.index(),
            entity: entity,
            entity_bevy_generation: entity.generation(),
        })
    }
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
    population
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

fn save_best_to_history(query: Query<&PlankPhenotype, With<EliteTag>>, generation_counter: Res<GenerationCounter>) {
    // let mut file = File::create("history.txt").expect("kunne ikke finne filen");
    let mut f = File::options().append(true).open("history.txt").expect("kunne ikke åpne filen");

    // let population = get_population_sorted_from_best_to_worst(query.iter());
    // let best = population[0];
    let best = query.get_single().unwrap();

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
    population.sort_by(|a, b| {
        if a.score > b.score {
            Ordering::Less
        } else if a.score < b.score {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
    return population;
}

// all the lifteimes bassicly just means to keep return value alive as long as the input value
pub fn get_population_sorted_from_best_to_worst_v2<'lifetime_a>(
    query: QueryIter<'lifetime_a, '_, (Entity, &PlankPhenotype, &Genome), ()>,
) -> Vec<PhentypeAndGenome<'lifetime_a>> {
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
            entity: entity,
            entity_bevy_generation: entity.generation(),
        });
    }

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
    return population;
}

fn print_pop_conditions(query: Query<(Entity, &PlankPhenotype, &Genome)>, generation_counter: Res<GenerationCounter>) {
    let population = get_population_sorted_from_best_to_worst_v2(query.iter());
    let best = population[0].clone();

    // let best_id = best.genotype.index();
    let all_fitnesses = population.iter().map(|individ| individ.phenotype.score);
    // println!("generation {} just ended, has population size {} Best individual: {} has fitness {} ", generation_counter.count, population.len(), best_id, best.score);
    println!(
        "generation {} just ended, has population size {} Best individual has fitness {} ",
        generation_counter.count,
        population.len(),
        best.phenotype.score
    );
    // println!("all fintesses for generation: ");
    // all_fitnesses.for_each(|score| print!("{} ", score));
    println!();
    println!("all fintesses for generation: ");
    population.into_iter().for_each(|individ| {
        print!(
            "Entity {} from bevy-generation {} har score {},",
            individ.entity_index, individ.entity_bevy_generation, individ.phenotype.score
        )
    });
    println!();
}
