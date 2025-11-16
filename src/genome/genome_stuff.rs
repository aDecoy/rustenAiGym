use bevy::prelude::{Component, ResMut, Resource};
use rand::distr::Uniform;
use rand::prelude::ThreadRng;
use rand::{random, rng, thread_rng, Rng};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

// #[derive(Debug, Component, Copy, Clone)] // todo spesifisker eq uten f32 verdiene
#[derive(Debug)] // todo spesifisker eq uten f32 verdiene
pub struct NodeGene {
    pub(crate) innovation_number: i32,
    // bias: f32,
    pub(crate) bias: RwLock<f32>, // fordi at andre ting (vekter) har refferanser til NodeGene, så er NodeGene pakket inn i en Arc (bevy kan ha flere systemer kjøre på entity med komponent samtidig, så asyncaronus).
    // For å få lov til å mutere bias, så må den være inne i en rwlock, siden hele klassen er inne i en Arc, som ikke gir skrivetilgang til vanlig
    pub(crate) enabled: RwLock<bool>,

    pub(crate) inputnode: bool,
    pub(crate) outputnode: bool,
    pub(crate) mutation_stability: f32,         // 1 is compleat lock/static genome. 0 is a mutation for all genes
    pub(crate) enabled_mutation_stability: f32, // drastic mutations such as disabling the entire neuron has a different parameter than just small changes
    pub(crate) layer: usize,
    // value: f32, // mulig denne blir flyttet til sin egen Node struct som brukes i nettverket, for å skille fra gen.

    //  Rwlock, slik at denne kan endres selv om den er inne i en Arc, som den må være om flere WeightGene skal kunne ha en peker på den,
    // og samtidig at jeg kan oppdatere verdien
    pub(crate) value: RwLock<f32>,

    pub(crate) label: String, // Only relevant for I/O nodes. To describe what input is what node and what node is what output
}

impl PartialEq for NodeGene {
    fn eq(&self, other: &Self) -> bool {
        self.innovation_number == other.innovation_number
    }
}
impl Eq for NodeGene {}

impl Hash for NodeGene {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.innovation_number.hash(state);
        // self.enabled.hash(state);
        self.inputnode.hash(state);
        self.outputnode.hash(state);
        self.layer.hash(state);
    }
}

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Debug, Clone)]
pub struct WeightGene {
    // innovation_number: i32, // kanskje ikke egentlig i bruk... . Skal samme forbindelse som oppstår i ulike muteringer utveksle informasjon? Kan brukes til å skille på det.
    pub(crate) value: f32,
    pub(crate) enabled: bool,
    // kildenode: i32,
    pub(crate) kildenode: Arc<NodeGene>,
    // destinationsnode: i32,
    pub(crate) destinasjonsnode: Arc<NodeGene>,
    pub(crate) value_mutation_stability: f32,
    pub(crate) enabled_mutation_stability: f32, // drastic mutations such as disabling the entire neuron has a different parameter than just small changes
    pub(crate) add_new_node_in_middle_mutation_stability: f32,
}
impl PartialEq for WeightGene {
    fn eq(&self, other: &Self) -> bool {
        self.kildenode == other.kildenode && self.destinasjonsnode == other.destinasjonsnode && self.value == other.value
    }
}
impl Eq for WeightGene {}

impl WeightGene {
    pub(crate) fn default_weight_with_random_value(thread_random: &mut ThreadRng, uniform_dist: Uniform<f32>, n: &Arc<NodeGene>, m: &Arc<NodeGene>) -> Self {
        WeightGene {
            kildenode: Arc::clone(n),
            destinasjonsnode: Arc::clone(m),
            value: thread_random.sample(uniform_dist),
            enabled: true,
            value_mutation_stability: 0.0,
            enabled_mutation_stability: 0.9,
            add_new_node_in_middle_mutation_stability: 0.5,
        }
    }

    pub(crate) fn neutral_default_for_hidden(end_node: &Arc<NodeGene>, new_middle_node: &Arc<NodeGene>) -> Self {
        let new_weight = WeightGene {
            kildenode: Arc::clone(&new_middle_node),
            destinasjonsnode: Arc::clone(&end_node),
            value: 1.0, // litt av ideen er at denne mutasjonen ikke egentlig skal påvirke nettverk output
            enabled: true,
            value_mutation_stability: 0.8,
            enabled_mutation_stability: 0.9,
            add_new_node_in_middle_mutation_stability: 0.5,
        };
        new_weight
    }
}

impl NodeGene {
    pub(crate) fn default_for_hidden(innovation_number_global_counter: &mut ResMut<InnovationNumberGlobalCounter>, innovation_number: i32) -> Self {
        NodeGene {
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
        }
    }
}

#[derive(Debug, Clone, Resource)]
pub struct InnovationNumberGlobalCounter {
    pub(crate) count: i32,
}
impl InnovationNumberGlobalCounter {
    pub(crate) fn get_number(&mut self) -> i32 {
        self.count += 1;
        return self.count;
    }
}

// #[derive(Debug, Component, Clone)]
#[derive(Debug, Component)]
pub(crate) struct Genome {
    // nodeGene can not be queried, since genome is a compnent and not an Entity. (It can be changed, but I feel like it is acceptable to give the entire genome to the bevy system

    // kan også kanskje vurdere å bruke bevy_hirearky for å operere på agenene idividuelt, istedenfor å altid gå via Genom parent
    // pub node_genes: Vec<NodeGene>,
    pub node_genes: Vec<Arc<NodeGene>>,
    pub weight_genes: Vec<WeightGene>,
    pub original_ancestor_id: i32,
    pub allowed_to_change: bool, // Useful to not mutate best solution found/Elite
}

impl Clone for Genome {
    fn clone(&self) -> Self {
        // Vi kan ikke gjøre en naiv copy, siden vi ikke vil at vektene til nye kopiert objekt skal reffere til noder i det orginale genomet
        let mut nye_noder = Vec::new();

        for node in self.node_genes.iter() {
            let ny_node: Arc<NodeGene>;
            {
                let bias = node.bias.read().unwrap();
                ny_node = Arc::new(NodeGene {
                    innovation_number: node.innovation_number.clone(),
                    bias: RwLock::new(bias.clone()),
                    enabled: RwLock::new(node.enabled.read().unwrap().clone()),
                    inputnode: node.inputnode,
                    outputnode: node.outputnode,
                    mutation_stability: node.mutation_stability,
                    enabled_mutation_stability: node.enabled_mutation_stability,
                    layer: 0,
                    // value: RwLock::new(0.0),
                    value: RwLock::new(node.value.read().unwrap().clone()),
                    label: node.label.clone(),
                });
            }
            nye_noder.push(ny_node);
        }
        let mut nye_vekter = Vec::new();
        for vekt in self.weight_genes.iter() {
            assert_eq!(nye_noder.iter().filter(|node| node.innovation_number == vekt.kildenode.innovation_number).count(), 1);
            assert_eq!(
                nye_noder
                    .iter()
                    .filter(|node| node.innovation_number == vekt.destinasjonsnode.innovation_number)
                    .count(),
                1
            );

            let ny_vekt = WeightGene {
                enabled: vekt.enabled,
                value_mutation_stability: vekt.value_mutation_stability,
                add_new_node_in_middle_mutation_stability: vekt.add_new_node_in_middle_mutation_stability,
                enabled_mutation_stability: vekt.enabled_mutation_stability,
                value: vekt.value,
                kildenode: Arc::clone(
                    nye_noder
                        .iter()
                        .filter(|node| node.innovation_number == vekt.kildenode.innovation_number)
                        .next()
                        .expect("fant ikke kildenode til vekten"),
                ),
                destinasjonsnode: Arc::clone(
                    nye_noder
                        .iter()
                        .filter(|node| node.innovation_number == vekt.destinasjonsnode.innovation_number)
                        .next()
                        .expect("fant ikke destinasjonsnoden til vekten"),
                ),
            };
            nye_vekter.push(ny_vekt);
        }

        Genome {
            node_genes: nye_noder,
            weight_genes: nye_vekter,
            original_ancestor_id: self.original_ancestor_id.clone(),
            allowed_to_change: self.allowed_to_change.clone(),
        }
    }
}

pub fn new_random_genome(ant_inputs: usize, ant_outputs: usize, innovation_number_global_counter: &mut ResMut<InnovationNumberGlobalCounter>) -> Genome {
    let mut node_genes = Vec::new();
    let mut thread_random = rng();
    let uniform_dist = Uniform::new(-1.0, 1.0).unwrap();
    // let mut input_layer2: Vec<NodeGene> = Vec::new();
    // let mut output_layer2: Vec<NodeGene> = Vec::new();
    for n in 0..ant_inputs {
        let label = format!("input {}", n);

        node_genes.push(NodeGene {
            // innovation_number: n as i32,
            innovation_number: innovation_number_global_counter.get_number(),
            bias: RwLock::new(thread_random.sample(uniform_dist)),
            enabled: RwLock::new(true),
            inputnode: true,
            outputnode: false,
            mutation_stability: 0.8,
            enabled_mutation_stability: 0.8,
            layer: 0,
            value: RwLock::new(0.0),
            label,
        });
    }

    for n in 0..ant_outputs {
        let label = format!("output {}", n);

        node_genes.push(NodeGene {
            innovation_number: innovation_number_global_counter.get_number(),
            // bias: thread_random.sample(uniform_dist),
            bias: RwLock::new(thread_random.sample(uniform_dist)),
            enabled: RwLock::new(true),
            inputnode: false,
            outputnode: true,
            mutation_stability: 0.8,
            enabled_mutation_stability: 0.8,
            layer: 0,
            value: RwLock::new(0.0),
            label: label,
        });
    }

    // Arc slik at jeg kan reffere til nodene inne i bevy sin asynkrone stuff, og slik at jeg kan ha flere blorrows

    let mut alle_noder_arc: Vec<Arc<NodeGene>> = Vec::new();
    for node in node_genes {
        let node_arc = Arc::new(node);
        alle_noder_arc.push(node_arc);
    }

    // start with no connections, start with fully connected, or random

    // fully connected input output
    let mut weight_genes = Vec::new();
    for n in alle_noder_arc.iter().filter(|node_gene| node_gene.inputnode) {
        // for n in 0..ant_inputs {
        //     for m in 0..ant_outputs {
        for m in alle_noder_arc.iter().filter(|node_gene| node_gene.outputnode) {
            weight_genes.push(WeightGene::default_weight_with_random_value(&mut thread_random, uniform_dist, n, m))
        }
    }
    return Genome {
        node_genes: alle_noder_arc,
        weight_genes: weight_genes,
        original_ancestor_id: random(),
        allowed_to_change: true,
    };
}

// pub(crate)  fn få_vekter_per_destinasjonskode(genome: &Genome) -> HashMap<Arc<NodeGene>, Vec<&WeightGene>> {
//     let mut weights_per_desination_node: HashMap<Arc<NodeGene>, Vec<&WeightGene>> = HashMap::new();
//
//     for weight in genome.weight_genes.iter() {
//         let list = weights_per_desination_node.entry(Arc::clone(&weight.destinasjonsnode)).or_insert_with(|| Vec::new());
//         // list.push(Arc::clone(&weight));
//         list.push(weight);
//     }
//     weights_per_desination_node
// }

impl Genome {
    // pub(crate) fn få_vekter_per_destinasjonskode(self: &Self) -> HashMap<Arc<NodeGene>, Vec<&WeightGene>> {
    pub(crate) fn få_aktive_vekter_per_aktive_destinasjonsnode(self: &Self) -> HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>> {
        let mut weights_per_desination_node: HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>> = HashMap::new();

        let aktive_vekter = self.weight_genes.iter().filter(|vekt| vekt.enabled).collect::<Vec<&WeightGene>>();
        // for weight in self.weight_genes.clone() {
        for weight in aktive_vekter {
            // for weight in self.weight_genes.clone().map(|weight| Arc::new(weight)) {
            {
                if weight.destinasjonsnode.enabled.read().unwrap().clone() == false {
                    // dont bother to add mapping for inaktive nodes
                    continue;
                }
                let list = weights_per_desination_node.entry(Arc::clone(&weight.destinasjonsnode)).or_insert_with(|| Vec::new());
                list.push(Arc::new(weight.clone()));
                // list.push(weight);
            }
            // for weight in genome.weight_genes.iter() {
        }
        weights_per_desination_node
    }
}
