use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hasher;
use std::iter::Map;
use std::vec::Vec;

// use bevy::asset::io::memory::Value::Vec;
use bevy::color::palettes::basic::PURPLE;
use bevy::prelude::*;
use bevy::prelude::KeyCode::{KeyE, KeyK, KeyP};
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy_inspector_egui::egui::emath::Numeric;
use bevy_rapier2d::na::DimAdd;
use rand::random;

use crate::environments::moving_plank::{create_plank, MovingPlankPlugin, mutate_planks};
use crate::environments::simulation_teller::SimulationRunningTellerPlugin;

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
        .insert_state(EttHakkState::DISABLED)
        .add_systems(Startup, (
            setup_camera,
            spawn_x_individuals
        ))
        .add_systems(Update, (
            endre_kjøretilstand_ved_input,
            agent_action.run_if(in_state(Kjøretilstand::Kjørende)),
            check_if_done,
            (
                create_new_children,
                mutate_planks,
                mutate_existing_nodes,
                mutate_existing_weights,
                reset_to_star_pos,
                set_to_kjørende_state).chain().run_if(in_state(Kjøretilstand::EvolutionOverhead))
        ),
        )
        // Environment spesific : Later changed
        .add_plugins(MovingPlankPlugin)
        .add_plugins(SimulationRunningTellerPlugin);

    app.run();
}

fn spawn_x_individuals(mut commands: Commands,
                       mut meshes: ResMut<Assets<Mesh>>,
                       mut materials: ResMut<Assets<ColorMaterial>>, ) {
    for n in 0i32..10 {
        let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::default());
        let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE));
        commands.spawn(
            create_plank(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + n as f32 * 50.0, z: 1.0 }, random(), new_random_genome(2, 2))
        );
    }
}

// pub fn spawn_phenotypes_from_genomes(
//     mut commands: Commands,
//     query: Query<(&Genome,&Individ) >,
//
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ){
//     for (genome, individ) in query.iter(){
//         let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::default());
//         let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE));
//         commands.spawn(
//             create_plank_phenotype(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + 3.3 * 50.0, z: 1.0 }, random(), *individ)
//         );
//     }
// }

fn create_new_children(mut commands: Commands,
                       mut meshes: ResMut<Assets<Mesh>>,
                       mut materials: ResMut<Assets<ColorMaterial>>,
                       query: Query<(Entity, &PlankPhenotype), With<PlankPhenotype>>) {
    let mut population = Vec::new();

    //sort_individuals
    for (entity, plank) in query.iter() {
        population.push(plank)
    }
    println!("parents before sort: {:?}", population);
    // parents.sort_by(|a, b| if a.score < b.score { Ordering::Less } else if a.score > b.score { Ordering::Greater } else { Ordering::Equal });
    // sort desc
    population.sort_by(|a, b| if a.score > b.score { Ordering::Less } else if a.score < b.score { Ordering::Greater } else { Ordering::Equal });
    println!("parents after sort:  {:?}", population);

    // create 3 children for each top 3
    let mut children = Vec::new();


    // Parent selection is set to top 3
    for n in (0..3) {
        children.push(population[n]);
    }

    // todo : change to spawn new genome with individ component tag, instead of spwaning a new plankPhenotype

    for child in children {
        let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::default());
        let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE));
        commands.spawn(
            create_plank(material_handle, rectangle_mesh_handle.into(), Vec3 { x: 0.0, y: -150.0 + 3.3 * 50.0, z: 1.0 }, child.phenotype, new_random_genome(2, 2))
        );
    }
}

fn reset_to_star_pos(mut query: Query<(&mut Transform, &mut PlankPhenotype), ( With<PlankPhenotype>)>) {
    for (mut individual, mut plank) in query.iter_mut() {
        individual.translation.x = 0.0;
        plank.score = individual.translation.x.clone();
        plank.obseravations = individual.translation.x.clone();
    }
}

fn set_to_kjørende_state(
    mut next_state: ResMut<NextState<Kjøretilstand>>,
) {
    next_state.set(Kjøretilstand::Kjørende);
}
fn check_if_done(mut query: Query<(&mut Transform, &mut PlankPhenotype), ( With<PlankPhenotype>)>,
                 mut next_state: ResMut<NextState<Kjøretilstand>>,
                 window: Query<&Window>,
) {
    let max_width = window.single().width() * 0.5;

    // done if one is all the way to the right of the screen
    for (mut individual, mut plank) in query.iter_mut() {
        if individual.translation.x > max_width {
            ; // er det skalert etter reapier logikk eller pixler\?
            next_state.set(Kjøretilstand::EvolutionOverhead)
        }
    }
}

// fn agent_action(query: Query<Transform, With<Individual>>) {
fn agent_action(mut query: Query<(&mut Transform, &mut PlankPhenotype), ( With<PlankPhenotype>)>) {
    for (mut individual, mut plank) in query.iter_mut() {
        // let (action , genome )= create_phenotype_layers(&plank.genotype);
        let mut phenotype_layers = create_phenotype_layers(plank.genotype.clone());
        // PhenotypeLayers::decide_on_action();
        let action = phenotype_layers.decide_on_action();
        // plank.genotype = genome; // Give genome back to plank after it was borrwed to create network


        individual.translation.x += random::<f32>() * action * 5.0;
        // individual.translation.x += random::<f32>() * plank.phenotype * 5.0;
        plank.score = individual.translation.x.clone();
        plank.obseravations = individual.translation.x.clone();
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

#[derive(Component, Debug, )]
// #[derive(Component, Eq, Ord, PartialEq, PartialOrd, PartialEq)]
pub struct PlankPhenotype {
    pub score: f32,
    pub obseravations: f32,
    pub phenotype: f32,
    pub genotype: Genome, // gentype takes over Genotype after genotype - phenotype tranformation
}


#[derive(Component, Debug)]
// #[derive(Component, Eq, Ord, PartialEq, PartialOrd, PartialEq)]
pub struct Individ {}

#[derive(Debug, Component, Copy, Clone)] // todo spesifisker eq uten f32 verdiene
pub struct NodeGene {
    innovation_number: usize,
    bias: f32,
    enabled: bool,
    inputnode: bool,
    outputnode: bool,
    mutation_stability: f32,
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
    weights_per_destination_node: HashMap<NodeGene, Vec<WeightGene>>,
}

impl PhenotypeLayers {
    pub fn decide_on_action(&mut self) -> f32 {

        // how to use

        let input_values = vec![1.0, 2.0]; // 2 inputs

        for i in 0..input_values.len() {
            self.input_layer[i].value = input_values[i] + self.input_layer[i].bias;
        }

        for mut node in self.output_layer.iter_mut() {
            // let relevant_weigh_nodes : Vec<&WeightGene> =  self.genome.weight_genes.iter().filter(  | weight_gene: &&WeightGene | weight_gene.destinationsnode == node.innovation_number  ).collect::<Vec<&WeightGene>>();   // bruk nodene istedenfor en vektor, slik at jeg vet hvilke vekter jeg skal bruke. Alt 2, sett opp nettet som bare vek først. Men det virker litt værre.
            // let relevant_weigh_nodes : Vec<&WeightGene> =  self.weights_per_destination_node.get(&node);
            let mut acc_value = 0.0;
            // for weight_node in relevant_weigh_nodes{
            // let kildenode : NodeGene =  genome.node_genes.iter().filter( | node_gene: &&NodeGene | weight_node.kildenode ==  node_gene.innovation_number ).collect();
            //  acc_value += kildenode.value * weight_node.value;
            // }
            // node.value = acc_value + node.bias;
        }


        for node in self.output_layer.iter() {
            println!("output nodes {:?}", node);
        }
        return random::<f32>();
    }
}

#[derive(Debug, Component, Clone)]
pub struct WeightGene {
    innovation_number: usize,
    value: f32,
    enabled: bool,
    kildenode: usize,
    destinationsnode: usize,
    mutation_stability: f32,
}

// alternativ 2 er å ha Noder som components , og legge de på plankBundle, og querlyie for with plank
#[derive(Debug, Component, Clone)]
struct Genome {
    pub node_genes: Vec<NodeGene>,
    pub weight_genes: Vec<WeightGene>,
    // id: usize,
}


// skal layers absorbere genome, skal den returnere genome og layers, eller skal den ta inn en copy av genome?
// trenger vi genome senere etter env ? Ja.
// Prøver å returenre begge


// fn create_phenotype_layers (genome: &Genome) -> (PhenotypeLayers, &Genome) {

// alt 2 tar inn en klone
fn create_phenotype_layers(genome: Genome) -> (PhenotypeLayers) {

    // for now just connect input output directly, and ignore hidden

    // let mut input_layer2 : Vec<&NodeGene>= Vec::new();
    // let mut  output_layer2: Vec<&NodeGene> = Vec::new();

    let mut input_layer2: Vec<NodeGene> = Vec::new();
    let mut output_layer2: Vec<NodeGene> = Vec::new();

    // for node in genome.node_genes.iter(){
    for node in genome.node_genes {
        if node.outputnode {
            output_layer2.push(node);
        } else if node.inputnode { input_layer2.push(node) }
    }

    // let input_layer = genome.node_genes.iter().filter( |node_gene: &&NodeGene | node_gene.inputnode ).collect();
    // let output_layer = genome.node_genes.iter().filter( |node_gene: &&NodeGene | node_gene.outputnode ).collect();
    // let mut layers = PhenotypeLayers { ant_layers: 2 , hidden_layers : Vec::new(), input_layer, output_layer };

    let mut weights_per_destination_node = HashMap::new();

    for node in output_layer2.iter() {
        // let relevant_weigh_nodes: Vec<&WeightGene> = genome.weight_genes.iter().filter(|weight_gene: &&WeightGene| weight_gene.destinationsnode == node.innovation_number).collect::<Vec<&WeightGene>>();   // bruk nodene istedenfor en vektor, slik at jeg vet hvilke vekter jeg skal bruke. Alt 2, sett opp nettet som bare vek først. Men det virker litt værre.
        let relevant_weigh_nodes: Vec<WeightGene> = for weight_gene in genome.weight_genes {
            if weights_per_destination_node.contains_key(weight_gene.destinationsnode){

            }

            https://stackoverflow.com/questions/32300132/why-cant-i-store-a-value-and-a-reference-to-that-value-in-the-same-struct
        }
            // .iter().filter(|weight_gene: &&WeightGene| weight_gene.destinationsnode == node.innovation_number).collect::<Vec<WeightGene>>();   // bruk nodene istedenfor en vektor, slik at jeg vet hvilke vekter jeg skal bruke. Alt 2, sett opp nettet som bare vek først. Men det virker litt værre.
        weights_per_destination_node.insert(node, relevant_weigh_nodes);
    }

    let mut layers = PhenotypeLayers { ant_layers: 2, hidden_layers: Vec::new(), input_layer: input_layer2, output_layer: output_layer2, weights_per_destination_node: weights_per_destination_node };


    // println!("output nodes {:?}", layers.output_layer.iter().map( | node_gene: NodeGene | node_gene ));
    // return (layers , genome);
    return (layers);
}

pub fn new_random_genome(ant_inputs: usize, ant_outputs: usize) -> Genome {
    let mut node_genes = Vec::new();
    for n in 0..ant_inputs {
        node_genes.push(NodeGene {
            innovation_number: n,
            bias: 0.0,
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
            innovation_number: n,
            bias: 0.0,
            enabled: true,
            inputnode: false,
            outputnode: true,
            mutation_stability: 0.0,
            layer: 0,
            value: 0.0,
        });
    }
    // start with no connections, start with fully connected, or random
    return Genome { node_genes: node_genes, weight_genes: Vec::new() };
}

pub fn mutate_existing_nodes(mut node_genes: Query<&mut NodeGene>) {
    for mut node_gene in node_genes.iter_mut() {
        if random::<f32>() < node_gene.mutation_stability {
            node_gene.bias += random::<f32>() * 2.0 - 1.0;
            node_gene.mutation_stability += random::<f32>() * 2.0 - 1.0;
            // enabling
        }
    }
}

pub fn mutate_existing_weights(mut weight_genes: Query<&mut WeightGene>) {
    for mut weight_gene in weight_genes.iter_mut() {
        if random::<f32>() < weight_gene.mutation_stability {
            weight_gene.value += random::<f32>() * 2.0 - 1.0;
            weight_gene.mutation_stability += random::<f32>() * 2.0 - 1.0;
        }
        if random::<f32>() < weight_gene.mutation_stability {
            weight_gene.enabled = !weight_gene.enabled;
        }

        // evo devo eller hardkoded layer?
        if random::<f32>() < weight_gene.mutation_stability {
            weight_gene.enabled = !weight_gene.enabled;
        }
    }
}
