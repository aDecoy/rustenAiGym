use crate::genome::genome_stuff::Genome;
use crate::{Kjøretilstand, PhentypeAndGenome, PlankPhenotype};
use bevy::asset::{Assets, Handle};
use bevy::color::Color;
use bevy::color::palettes::basic::RED;
use bevy::ecs::query::QueryIter;
use bevy::prelude::*;
use std::cmp::Ordering;

#[derive(Debug, Component)]
pub struct EliteTag;

struct ElitePlugin;

impl Plugin for ElitePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            ((color_elite_red, add_elite_component_tag_to_best_individ)),
        )
        .add_systems(
            Update,
            ((add_elite_component_tag_to_best_individ, color_elite_red)
                .chain()
                .run_if(in_state(Kjøretilstand::EvolutionOverhead)),),
        );
    }
}

pub fn get_best_elite<'a>(
    iteratior: QueryIter<'a, '_, (Entity, &PlankPhenotype, &'_ Genome), With<PlankPhenotype>>,
) -> PhentypeAndGenome<'a> {
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
    if let Ok(old_elite_entity) = old_elite_query.get_single() {
        commands.entity(old_elite_entity).remove::<EliteTag>();
    }
    // find new elite
    let elite = get_best_elite(query.iter());
    commands.entity(elite.entity).insert((EliteTag));
}

fn color_elite_red(
    mut commands: Commands,
    mut elite_query: Query<Entity, With<EliteTag>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if let Ok(elite_entity) = elite_query.get_single() {
        let elite_material_handle: Handle<ColorMaterial> = materials.add(Color::from(RED));
        commands
            .entity(elite_entity)
            .insert(MeshMaterial2d(elite_material_handle));
    }
}

fn sort_best_to_worst<'a>(
    iteratior: QueryIter<'a, '_, (Entity, &PlankPhenotype, &'_ Genome), With<PlankPhenotype>>,
) -> Vec<PhentypeAndGenome<'a>> {
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
