use std::collections::HashMap;
use std::sync::Arc;


struct FenotypePlugin;

impl Plugin for FenotypePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(    Update,
                             (

                )

            )
    }
}


// Denne kan potensielt heller gjørs med event
fn update_phenotype_network_for_changed_genomes(mut query: Query<(&Genome, &mut PlankPhenotype)>) {
    // genom verdier endrer seg ved tenking også, kan derfor ikke bruke  Changed<Genome>>, siden det tar alle
    for (genome, mut plank_phenotype) in query.iter_mut() {
        if genome.allowed_to_change {
            plank_phenotype.phenotype_layers = PhenotypeNeuralNetwork::new(genome)
        }
    }
}

/// An observer to rotate an entity when it is dragged
// fn rotate_on_drag(drag: Trigger<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
fn rotate_on_drag(drag: Trigger<Pointer<Drag>>, mut angular_velocities: Query<&mut AngularVelocity>) {
    println!("dragging");
    let mut angular_velocitiy = angular_velocities.get_mut(drag.target.entity()).unwrap();
    angular_velocitiy.0 += 0.1;
}

fn add_observers_to_individuals(mut commands: Commands, individ_query: Query<Entity, With<PlankPhenotype>>) {
    for individ_entity in individ_query.iter() {
        commands
            .get_entity(individ_entity)
            .unwrap()
            // .observe(pointer_out_of_individ)
            .observe(rotate_on_drag);
        // .observe(place_in_focus);
    }
}

#[derive(Clone, Debug)]
struct PhentypeAndGenome<'lifetime_a> {
    phenotype: &'lifetime_a PlankPhenotype,
    genome: &'lifetime_a Genome,
    entity_index: u32,
    entity_bevy_generation: u32,
    entity: Entity,
}


#[derive(Component)]
struct IndividFitnessLabelTextTag;

#[derive(Component)]
struct IndividFitnessLabelText {
    entity: Entity,
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

