use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use bevy::prelude::{Component, ResMut, Resource};
use rand::distributions::Uniform;
use rand::{random, thread_rng, Rng};

// #[derive(Debug, Component, Copy, Clone)] // todo spesifisker eq uten f32 verdiene
#[derive(Debug)] // todo spesifisker eq uten f32 verdiene
pub struct NodeGene {
    innovation_number: i32,
    // bias: f32,
    pub(crate) bias: RwLock<f32>, // fordi at andre ting (vekter) har refferanser til NodeGene, så er NodeGene pakket inn i en Arc (bevy kan ha flere systemer kjøre på entity med komponent samtidig, så asyncaronus).
    // For å få lov til å mutere bias, så må den være inne i en rwlock, siden hele klassen er inne i en Arc, som ikke gir skrivetilgang til vanlig
    enabled: bool,

    pub(crate) inputnode: bool,
    pub(crate) outputnode: bool,
    pub(crate) mutation_stability: f32, // 1 is compleat lock/static genome. 0 is a mutation for all genes
    layer: usize,

    // value: f32, // mulig denne blir flyttet til sin egen Node struct som brukes i nettverket, for å skille fra gen.

    //  Rwlock, slik at denne kan endres selv om den er inne i en Arc, som den må være om flere WeightGene skal kunne ha en peker på den,
    // og samtidig at jeg kan oppdatere verdien
    pub(crate) value: RwLock<f32>,
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
        self.enabled.hash(state);
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
    pub(crate) mutation_stability: f32,
}
impl PartialEq for WeightGene {
    fn eq(&self, other: &Self) -> bool {
        self.kildenode == other.kildenode &&
            self.destinasjonsnode == other.destinasjonsnode &&
            self.value == other.value
    }
}
impl Eq for WeightGene {}


#[derive(Debug, Clone, Resource)]
pub struct InnovationNumberGlobalCounter {
    pub(crate) count: i32,
}
impl InnovationNumberGlobalCounter {
    fn get_number(&mut self) -> i32 {
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
    pub original_ancestor_id: usize,
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
                    enabled: node.enabled,
                    inputnode: node.inputnode,
                    outputnode: node.outputnode,
                    mutation_stability: node.mutation_stability,
                    layer: 0,
                    value: RwLock::new(0.0),
                }
                );
            }
            nye_noder.push(ny_node);
        }
        let mut nye_vekter = Vec::new();
        for vekt in self.weight_genes.iter() {
            assert_eq!(nye_noder.iter().filter(|node| node.innovation_number == vekt.kildenode.innovation_number).count(), 1);
            assert_eq!(nye_noder.iter().filter(|node| node.innovation_number == vekt.destinasjonsnode.innovation_number).count(), 1);


            let ny_vekt = WeightGene {
                enabled: vekt.enabled,
                mutation_stability: vekt.mutation_stability,
                value: vekt.value,
                kildenode: Arc::clone(nye_noder.iter().filter(|node| node.innovation_number == vekt.kildenode.innovation_number).next().expect("fant ikke kildenode til vekten")),
                destinasjonsnode: Arc::clone(nye_noder.iter().filter(|node| node.innovation_number == vekt.destinasjonsnode.innovation_number).next().expect("fant ikke destinasjonsnoden til vekten")),
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

pub fn new_random_genome(ant_inputs: usize, ant_outputs: usize, innovationNumberGlobalCounter: &mut ResMut<InnovationNumberGlobalCounter>) -> Genome {
    let mut node_genes = Vec::new();
    let mut thread_random = thread_rng();
    let uniform_dist = Uniform::new(-1.0, 1.0);
    // let mut input_layer2: Vec<NodeGene> = Vec::new();
    // let mut output_layer2: Vec<NodeGene> = Vec::new();
    for n in 0..ant_inputs {
        node_genes.push(NodeGene {
            // innovation_number: n as i32,
            innovation_number: innovationNumberGlobalCounter.get_number(),
            bias: RwLock::new(thread_random.sample(uniform_dist)),
            enabled: true,
            inputnode: true,
            outputnode: false,
            mutation_stability: 0.0,
            layer: 0,
            value: RwLock::new(0.0),
        });
    }

    for n in 0..ant_outputs {
        node_genes.push(NodeGene {
            innovation_number: innovationNumberGlobalCounter.get_number(),
            // bias: thread_random.sample(uniform_dist),
            bias: RwLock::new(thread_random.sample(uniform_dist)),
            enabled: true,
            inputnode: false,
            outputnode: true,
            mutation_stability: 0.0,
            layer: 0,
            value: RwLock::new(0.0),
        });
    }

    // Arc slik at jeg kan reffere til nodene inne i bevy sin asynkrone stuff, og slik at jeg kan ha flere blorrows

    let mut alleNoderArc: Vec<Arc<NodeGene>> = Vec::new();
    for node in node_genes {
        let nodeArc = Arc::new(node);
        alleNoderArc.push(nodeArc);
    }


    // start with no connections, start with fully connected, or random

    // fully connected input output
    let mut weight_genes = Vec::new();
    for n in alleNoderArc.iter().filter(|node_gene| node_gene.inputnode) {
        // for n in 0..ant_inputs {
        //     for m in 0..ant_outputs {
        for m in alleNoderArc.iter().filter(|node_gene| node_gene.outputnode) {
            weight_genes.push(WeightGene {
                kildenode: Arc::clone(n),
                destinasjonsnode: Arc::clone(m),
                // kildenode: n as i32,
                // kildenode: n.innovation_number,
                // destinationsnode: m.innovation_number,
                // innovation_number: innovationNumberGlobalCounter.get_number(),

                value: thread_random.sample(uniform_dist),
                enabled: true,
                mutation_stability: 0.0,
            })
        }
    }
    return Genome { node_genes: alleNoderArc, weight_genes: weight_genes, original_ancestor_id: random(), allowed_to_change: true };
}
