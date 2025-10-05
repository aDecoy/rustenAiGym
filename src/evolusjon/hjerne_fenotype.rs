use crate::environments::moving_plank::PIXELS_PER_METER;
use crate::genome::genome_stuff::{Genome, NodeGene, WeightGene};
use bevy::prelude::Query;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct PhenotypeNeuralNetwork {
    ant_layers: usize,
    // Holder på objektene
    // alleNoder: Vec<NodeGene>,
    alle_vekter: Vec<WeightGene>,
    // hidden_layers: Vec<Vec<&'a NodeGene>>,
    // hidden_nodes: Vec<Vec<Arc<NodeGene>>>,
    input_layer: Vec<Arc<NodeGene>>,
    output_layer: Vec<Arc<NodeGene>>,
    // &'a to promise compiler that it lives the same length
    // weights_per_destination_node : HashMap<&'a NodeGene, Vec<&'a WeightGene>>,
    // weights_per_destination_node: HashMap<i32, Vec<WeightGene>>,
    weights_per_destination_node: HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>>,
    node_to_layer: HashMap<Arc<NodeGene>, i32>,
    layers_ordered_output_to_input: Vec<Vec<Arc<NodeGene>>>,
}

impl PhenotypeNeuralNetwork {
    pub(crate) fn decide_on_action2(&self, input_values: Vec<f32>) -> Vec<f32> {
        // Normalisering
        // ikke helt sikker på hvordan jeg skal normalisere input verdier enda.
        let mut clamped_input_values = Vec::new();
        clamped_input_values.reserve(input_values.len());
        for value in input_values {
            //     println!("raw input values {:?}", node);
            // clamped_input_values.push(value / PIXELS_PER_METER);
            clamped_input_values.push(value / PIXELS_PER_METER);
            // println!("new clamped input value {:?}", value);
        }
        // dbg!(&clamped_input_values);

        // Feed/load in input values
        // dbg!(&self.input_layer);
        for i in 0..clamped_input_values.len() {
            let node = &self.input_layer[i];
            if node.enabled.read().unwrap().clone() == false {
                continue;
            }
            let mut verdi = node.value.write().unwrap();
            let bias = node.bias.read().unwrap();
            let new_input_value = clamped_input_values[i];
            // dbg!(&node);
            // we intentionally keep the previous verdi. (it is calmed down after each use of the network
            *verdi = *verdi + new_input_value + *bias;
            // dbg!(&verdi);
            // println!("&self.input_layer[i] addr {:p}",*node);
            // dbg!(&node);
            // self.input_layer[i].value = clamped_input_values[i] + self.input_layer[i].bias;
        }
        // dbg!(&self.input_layer);

        // update all values
        // for i in (self.ant_layers..0) {
        // dbg!( &self.layers_ordered_output_to_input);

        for i in (0..self.ant_layers).rev() {
            // dbg!(i);
            for destination_node in &self.layers_ordered_output_to_input[i] {
                // dbg!(destination_node);
                self.absorber_inkommende_verdier_og_set_ny_verdi(destination_node);
                // dbg!(destination_node);
            }
        }
        // Read of output neurons  (always last layer)

        let mut output_values = Vec::new();
        for node in self.output_layer.iter() {
            let verdi = node.value.read().unwrap();
            output_values.push(verdi.clone());
            // dbg!(verdi.clone());
        }
        // burde kanksje normalisere output også....
        // dbg!(&output_values);
        return output_values;
    }

    fn absorber_inkommende_verdier_og_set_ny_verdi(&self, destination_node: &Arc<NodeGene>) {
        let relevant_weights = match self.weights_per_destination_node.get(destination_node) {
            Some(weights) => weights,
            None => &Vec::new(),
        };
        // dbg!(&destination_node);
        // dbg!(&relevant_weights);
        // dbg!(&self.weights_per_destination_node);
        let mut alle_inputs_til_destination_node: Vec<f32> = Vec::new();
        for weight in relevant_weights.iter() {
            {
                let kildenode = &weight.kildenode;
                let kildenode_verdi = kildenode.value.read().unwrap();
                let kildenode_påvirkning = *kildenode_verdi * weight.value;
                alle_inputs_til_destination_node.push(kildenode_påvirkning);
            }
        }
        // output noder blir lagt til i et layer selv om de er inaktive, vi gir de derfor 0.0 i verdi om de er inaktive, istedenfor å alltid fyre av bias verdien
        let total_påvirking: f32 = if destination_node.outputnode && !destination_node.enabled.read().unwrap().clone() {
            0.0
        } else {
            alle_inputs_til_destination_node.iter().sum()
        };
        // dbg!(&total_påvirking);
        {
            let mut verdi = destination_node.value.write().unwrap();
            // println!("verdi før halvering {}", verdi);
            *verdi = *verdi * 0.5; // Hvis ikke resetter alt til 0 hver hver gang, men istedenfor er akkumulativ, så kreves det en demper også for å ikke gå til uendelig.
            *verdi = *verdi + total_påvirking;
        }
    }

    pub(crate) fn new(genome: &Genome) -> Self {
        // let alle_vekter: Vec<WeightGene> = genome.weight_genes.clone();

        let weights_per_desination_node = genome.få_aktive_vekter_per_aktive_destinasjonsnode();
        // dbg!(&weights_per_desination_node);
        let (node_to_layer, layers_ordered_output_to_input) = PhenotypeNeuralNetwork::lag_lag_av_nevroner_sortert_fra_output(genome, &weights_per_desination_node);

        // WE DONT flip it around to be sorted so that output nodes are last . No need. just fill in input node values and iterate         for i in (ant_layers..0){

        let mut input_layer: Vec<Arc<NodeGene>> = Vec::new();
        let mut output_layer: Vec<Arc<NodeGene>> = Vec::new();

        for node_arc in genome.node_genes.iter() {
            // let  node_arc = Arc::new(node);
            if node_arc.inputnode {
                input_layer.push(Arc::clone(node_arc))
            } else if node_arc.outputnode {
                output_layer.push(Arc::clone(node_arc));
            }
        }
        let alle_enabled_vekter: Vec<WeightGene> = genome.weight_genes.clone().into_iter().filter(|weight_gene| weight_gene.enabled).collect();

        /* `PhenotypeNeuralNetwork` value */
        PhenotypeNeuralNetwork {
            ant_layers: layers_ordered_output_to_input.len(),
            // alleNoder: alleNoder,
            // alleNoderArc: alleNoderArc,  // remove!! ikke i bruk !!!
            alle_vekter: alle_enabled_vekter,
            // hidden_nodes: vec![],
            input_layer: input_layer,
            output_layer: output_layer,
            // weights_per_destination_node: weights_per_desination_node,
            weights_per_destination_node: weights_per_desination_node,
            node_to_layer,
            layers_ordered_output_to_input,
        }
    }

    pub(crate) fn lag_lag_av_nevroner_sortert_fra_output(
        genome: &Genome,
        // weights_per_desination_node: &HashMap<Arc<NodeGene>, Vec<&WeightGene>>)
        weights_per_desination_node: &HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>>,
    ) -> (HashMap<Arc<NodeGene>, i32>, Vec<Vec<Arc<NodeGene>>>) {
        let output_nodes: Vec<Arc<NodeGene>> = genome.node_genes.clone().iter().filter(|node| node.outputnode).map(|node| Arc::clone(node)).collect();

        // Start on input, and look at what connects.  STARTER PÅ OUTPUT OG BEVEGEWR OSS MOT INPUT
        // Starter på output for å bare inkludere noder og vekter som faktisk påvirker utfallet
        let mut node_to_layer = HashMap::new();
        output_nodes.iter().for_each(|node| {
            node_to_layer.insert(node.clone(), 0);
        });
        let mut layers_ordered_output_to_input: Vec<Vec<Arc<NodeGene>>> = vec![output_nodes];

        // dbg!(&node_to_layer);

        // I tilfeller vi har sykler, så vil vi hindre å evig flytte ting bakover i nettet. På et punkt så må vi bare godta en node kan få input som ikke er fra "venstre side". Bygger opp fra høyre side med outputs og jobber oss mot venstre.
        // Dette er løst ved å kun flytte en node en gang per vekt. (dette vil gjøre at sykluser kan gi hidden noder som er til venstre for input noder).
        // Merk at syklus noder vil gjøre litt ekstra forsterkning av sine verdier i forhold til andre vanlige hidden noder om de er til venstre for input noder.  Disse vil "ta inn nåtid data + sin fortid data og gi ut begge"
        let mut node_to_vekt_som_flyttet_på_noden: HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>> = HashMap::new();
        // let mut node_to_vekt_som_flyttet_på_noden: HashMap<Arc<NodeGene>, Vec<&WeightGene>> = HashMap::new();
        // let next_layer = få_neste_lag(&weights_per_desination_node, &mut node_to_layer, &mut layers_ordered_output_to_input, &mut node_to_vekt_som_flyttet_på_noden, 1);
        // layers_ordered_output_to_input.push(next_layer);

        let mut layer_index = 1;
        loop {
            // dbg!(&layer_index);
            let next_layer = PhenotypeNeuralNetwork::få_neste_lag(
                &weights_per_desination_node,
                &mut node_to_layer,
                &mut layers_ordered_output_to_input,
                &mut node_to_vekt_som_flyttet_på_noden,
                layer_index,
            );
            layer_index += 1;
            // dbg!(&next_layer);
            // dbg!(&next_layer.len());
            if next_layer.len() == 0 {
                break;
            }
            layers_ordered_output_to_input.push(next_layer);
            // break;
        }
        // dbg!(&layers_ordered_output_to_input);

        (node_to_layer, layers_ordered_output_to_input)
    }

    fn få_neste_lag<'a>(
        // weights_per_desination_node: &HashMap<Arc<NodeGene>, Vec<&'a WeightGene>>,
        weights_per_desination_node: &HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>>,
        layer_per_node: &mut HashMap<Arc<NodeGene>, i32>,
        layers_output_to_input: &mut Vec<Vec<Arc<NodeGene>>>,
        // node_to_vekt_som_flyttet_på_noden: &mut HashMap<Arc<NodeGene>, Vec<&'a WeightGene>>,
        node_to_vekt_som_flyttet_på_noden: &mut HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>>,
        lag_index: i32,
    ) -> Vec<Arc<NodeGene>> {
        let mut next_layer = vec![];
        // for node in input_layer.iter() {
        // for node in layers_output_to_input.iter().last().unwrap() {
        // let last_layer = layers_output_to_input.iter().last();
        // for node in layers_output_to_input.last().iter() {
        // dbg!(&lag_index);
        // dbg!(&weights_per_desination_node);
        // dbg!(&layers_output_to_input);

        for node in layers_output_to_input.last().unwrap() {
            // for node in layers_output_to_input.iter().last().iter() {
            //     let node2 : &Arc<NodeGene> = *node;
            //     for weight in weights_per_desination_node.get(&Arc::clone(node)).expect("burde eksistere") {
            let mut vekter_allerede_brukt = match node_to_vekt_som_flyttet_på_noden.get_mut(&Arc::clone(node)) {
                None => Vec::new(),
                Some(liste) => liste.clone(),
            };
            // dbg!(&node);
            match weights_per_desination_node.get(node) {
                Some(weights) => {
                    for weight in weights {
                        if !vekter_allerede_brukt.contains(weight) {
                            // It can be multiple weights that would have added the same node to the layer, but we only want one arc refference to the node. But we want vekter allerede brukt to be updated
                            if !next_layer.contains(&weight.kildenode) {
                                next_layer.push(Arc::clone(&weight.kildenode));
                            }
                            layer_per_node.insert(Arc::clone(&weight.kildenode), lag_index);
                            vekter_allerede_brukt.push(Arc::clone(weight));
                        }
                    }
                }
                _ => {}
            };
            node_to_vekt_som_flyttet_på_noden.insert(Arc::clone(node), vekter_allerede_brukt);
        }
        next_layer
    }
}

// NB : Dette fjerner potensielt opplærte feedbackloops i nettverket i tilfeller det eksisterer en "barndom" for individet.
// Men denne er nyttig for å se om resten er deterministisk
// En annen ting som kan tenkes å være nyttig er å gi random verdier, slik at vi trener at individer kan hente seg inn fra dårlig init inputs
pub fn nullstill_nettverk_verdier_til_0(mut query: Query<&mut Genome>) {
    for mut genome in query.iter_mut() {
        for node_gen in genome.node_genes.iter_mut() {
            let mut verdi = node_gen.value.write().unwrap();
            *verdi = 0.0;
        }
    }
}
