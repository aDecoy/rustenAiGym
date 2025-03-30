use crate::environments::genome_stuff::{Genome, NodeGene, WeightGene};
use crate::PlankPhenotype;
use bevy::prelude::{Changed, Query};
use rand::random;
use std::sync::Arc;

// lock and unlock mutation to lock parents/Elites. Still not decided if i want a 100% lock or allow some small genetic drift also in elites
pub(crate) fn lock_mutation_stability(mut genome_query: Query<&mut Genome>) {
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
            mutate_existing_nodes_arc(&mut gene.node_genes);
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
    for node_gene in node_genes.iter_mut() {
        if random::<f32>() > node_gene.mutation_stability {
            let change = (random::<f32>() * 2.0 - 1.0) * mutation_strength;
            {
                let mut bias = node_gene.bias.write().unwrap();
                *bias = *bias + change;
            }
            // node_gene.mutation_stability += random::<f32>() * 2.0 - 1.0;
            // enabling
        }
    }
}
pub fn mutate_existing_nodes_arc(mut node_genes: &mut Vec<Arc<NodeGene>>) {
    // println!("mutating {} nodes ", node_genes.iter().count());
    let mutation_strength = 2.0;
    for mut node_gene in node_genes.iter_mut() {
        if random::<f32>() > node_gene.mutation_stability {
            let change = (random::<f32>() * 2.0 - 1.0) * mutation_strength;
            {
                let mut bias = node_gene.bias.write().unwrap();
                *bias = *bias + change;
            }
            // node_gene.bias += (random::<f32>() * 2.0 - 1.0) * mutation_strength;
            // node_gene.mutation_stability += random::<f32>() * 2.0 - 1.0;
            // enabling
        }
        if random::<f32>() > node_gene.enabled_mutation_stability && !node_gene.outputnode {
            let mut enabled = node_gene.enabled.write().unwrap();
            *enabled = !*enabled;
        }
    }
}

pub fn mutate_existing_weights(mut weight_genes: &mut Vec<WeightGene>) {
    // println!("mutating {} weights ", weight_genes.iter().count());
    let mutation_strength = 1.0;

    for mut weight_gene in weight_genes.iter_mut() {
        // println!("weight gene mutation_stability : {}", weight_gene.mutation_stability);
        if random::<f32>() > weight_gene.mutation_stability {
            // println!("weight gene value before mutation: {}", weight_gene.value);
            weight_gene.value += (random::<f32>() * 2.0 - 1.0) * mutation_strength;
            // println!("weight gene value after mutation: {}", weight_gene.value);
            // weight_gene.mutation_stability += random::<f32>() * 2.0 - 1.0;
        }
        if random::<f32>() > weight_gene.enabled_mutation_stability {
            weight_gene.enabled = !weight_gene.enabled;
        }
    }
}
