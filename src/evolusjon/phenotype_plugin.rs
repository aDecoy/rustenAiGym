use crate::evolusjon::hjerne_fenotype::PhenotypeNeuralNetwork;
use crate::genome::genome_stuff::Genome;
use avian2d::prelude::AngularVelocity as AngularVelocity2d;
use avian3d::prelude::{AngularVelocity as AngularVelocity3d, LinearVelocity as LinearVelocity3d};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use avian3d::math::Vector;
use avian3d::picking::PhysicsPickingPlugin;

pub struct FenotypePlugin;

impl Plugin for FenotypePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(PhysicsPickingPlugin)
            .add_systems(PostStartup, ((label_plank_with_current_score,add_observers_to_individuals)))
            .add_systems(Update, ((label_plank_with_current_score,add_observers_to_individuals)));
    }
}

// Denne kan potensielt heller gjørs med event
pub fn update_phenotype_network_for_changed_genomes(mut query: Query<(&Genome, &mut PlankPhenotype)>) {
    // genom verdier endrer seg ved tenking også, kan derfor ikke bruke  Changed<Genome>>, siden det tar alle
    for (genome, mut plank_phenotype) in query.iter_mut() {
        if genome.allowed_to_change {
            plank_phenotype.phenotype_layers = PhenotypeNeuralNetwork::new(genome)
        }
    }
}

/// An observer to rotate an entity when it is dragged
// fn rotate_on_drag(drag: On<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
fn rotate_on_drag2d(drag: On<Pointer<Drag>>, mut angular_velocities: Query<&mut AngularVelocity2d>) {
    println!("dragging");
    let mut angular_velocitiy = angular_velocities.get_mut(drag.event().entity.entity()).unwrap();
    angular_velocitiy.0 += 0.1;
}
// fn rotate_on_drag(drag: On<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
fn rotate_on_drag3d(drag: On<Pointer<Drag>>, mut angular_velocities: Query<&mut AngularVelocity3d>) {
    println!("dragging3d rotate");
    let mut angular_velocitiy = angular_velocities.get_mut(drag.event().entity.entity()).unwrap();
    angular_velocitiy.0 += Vector::new(0.1,0.1,0.1);
}
fn get_velocity_on_drag3d(drag: On<Pointer<Drag>>, mut velocities: Query<&mut LinearVelocity3d>) {
    println!("dragging3d velocity");
    let mut velocitiy = velocities.get_mut(drag.event().entity.entity()).unwrap();
    velocitiy.0 += Vector::new(drag.delta.x * 0.02,-drag.delta.y * 0.02,0.);  // todo https://bevy.org/examples/picking/mesh-picking/
}

pub fn add_observers_to_individuals(mut commands: Commands, individ_query: Query<Entity, Added<PlankPhenotype>>) {
    for individ_entity in individ_query.iter() {
        commands.get_entity(individ_entity).unwrap()
            // .observe(rotate_on_drag2d)
            .observe(get_velocity_on_drag3d)
        .observe(rotate_on_drag3d);
    }
}

#[derive(Clone, Debug)]
pub struct PhentypeAndGenome<'lifetime_a> {
    pub phenotype: &'lifetime_a PlankPhenotype,
    pub genome: &'lifetime_a Genome,
    pub entity_index: u32,
    pub entity_bevy_generation: u32,
    pub entity: Entity,
}

#[derive(Component)]
pub struct IndividFitnessLabelTextTag;

#[derive(Component)]
pub struct IndividFitnessLabelText {
    pub entity: Entity,
}

#[derive(Component, Debug)]
// #[derive(Component, Eq, Ord, PartialEq, PartialOrd, PartialEq)]
pub struct Individ {} // denne er ganske det samme som PlankPhenotype, siden alle individer har PlankPhenotype

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

fn label_plank_with_current_score(mut text_query: Query<(&mut Text2d, &ChildOf), With<IndividFitnessLabelTextTag>>, parent_query: Query<&PlankPhenotype>) {
    for (mut tekst, child_of) in text_query.iter_mut() {
        if let Ok(plank_phenotype) = parent_query.get(child_of.parent()) {
            tekst.0 = plank_phenotype.score.to_string();
        }
    }
}
