use crate::environments::genome_stuff::{Genome, InnovationNumberGlobalCounter, NodeGene, WeightGene};
use crate::PlankPhenotype;
use bevy::prelude::{Changed, Query, ResMut};
use rand::random;
use std::sync::{Arc, RwLock};

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

pub fn mutate_genomes(mut genomer: Query<&mut Genome>,
                      mut innovation_number_global_counter: ResMut<InnovationNumberGlobalCounter>,
) {
    for mut genom in genomer.iter_mut() {
        // println!("mutating genome with original ancestor {}, if allowed: {} ", gene.original_ancestor_id, gene.allowed_to_change);
        if genom.allowed_to_change {
            mutate_existing_nodes_arc(&mut genom.node_genes);
            mutate_existing_weights_value_and_på_av(&mut genom.weight_genes);
            mutate_new_node_in_middle_mutation(&mut genom, &mut innovation_number_global_counter )
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

pub fn mutate_existing_weights_value_and_på_av(mut weight_genes: &mut Vec<WeightGene>) {
    // println!("mutating {} weights ", weight_genes.iter().count());
    let mutation_strength = 1.0;

    for mut weight_gene in weight_genes.iter_mut() {
        // println!("weight gene mutation_stability : {}", weight_gene.mutation_stability);
        if random::<f32>() > weight_gene.value_mutation_stability {
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

pub fn mutate_new_node_in_middle_mutation(
    mut genome:  &mut Genome,
    innovation_number_global_counter: &mut ResMut<InnovationNumberGlobalCounter>,
) {
    let mut weight_genes: &mut Vec<WeightGene> =  &mut genome.weight_genes;
    let mut node_genes: &mut Vec<Arc<NodeGene>> =  &mut genome.node_genes;
    
    let mut weight_genes_to_add: Vec<WeightGene> =  Vec::new();
    
    for mut weight_gene in weight_genes.iter_mut() {
        if random::<f32>() > weight_gene.add_new_node_in_middle_mutation_stability {
            // O ------- O  =>  O ----O--- O
            let end_node = Arc::clone(&weight_gene.destinasjonsnode);
            let start_node =  Arc::clone(&weight_gene.kildenode);
            let innovation_number = innovation_number_global_counter.get_number();
            let new_middle_node = Arc::new( NodeGene { // todo ha noe defalult for hidden noder?
                // innovation_number: n as i32,
                innovation_number: innovation_number_global_counter.get_number(),
                bias: RwLock::new(0.0), // litt av ideen er at det ikke egentlig skal påvirke noe med denne mutasjonen
                enabled: RwLock::new(true),
                inputnode: false,
                outputnode: false,
                mutation_stability: 0.8,
                enabled_mutation_stability: 0.8,
                layer: 0, // denne brukes vell strengt tatt ikke
                value: RwLock::new(0.0),
                label: innovation_number.to_string(),
            });
            weight_gene.destinasjonsnode =  Arc::clone(&new_middle_node);

            let new_weight = WeightGene { // todo kanskje lage default verdier for hidden weightGene?
                kildenode: Arc::clone(&new_middle_node),
                destinasjonsnode: Arc::clone(&end_node),
                value: 1.0, // litt av ideen er at denne mutasjonen ikke egentlig skal påvirke nettverk output 
                enabled: true,
                value_mutation_stability: 0.8,
                enabled_mutation_stability: 0.9,
                add_new_node_in_middle_mutation_stability: 0.5
            };

            node_genes.push(new_middle_node);
            weight_genes_to_add.push(new_weight);

        }
    }
    genome.weight_genes.append(&mut weight_genes_to_add);
}
