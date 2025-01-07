use crate::{Genome, NodeGene, WeightGene};
use bevy::color::palettes::basic::{BLUE, GREEN, RED};
use bevy::color::ColorCurve;
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::Arc;

pub fn draw_network_in_genome(
     commands: Commands,
     meshes: ResMut<Assets<Mesh>>,
     materials: ResMut<Assets<ColorMaterial>>,
    query: Query<&Genome>,
) {
    let genome = query.single();
    draw_network_in_genome2(commands, meshes, materials, genome);
}

pub fn draw_network_in_genome2(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    genome: &Genome,
) {
    let weights_per_desination_node = Genome::få_vekter_per_destinasjonskode(&genome);
    // let weights_per_kildenode = få_vekter_per_kildenode(genome);
    /////////////////////////////// få_vekter_per_kildenode
    let (node_to_layer, layers_ordered_output_to_input) =
        lag_lag_av_nevroner_sortert_fra_output(genome, &weights_per_desination_node);
    // let (node_to_layer, layers_ordered_output_to_input) = genome.lag_lag_av_nevroner_sortert_fra_output( &weights_per_desination_node);
    //////////////////////////////

    let point_per_node = kordinater_per_node(genome, node_to_layer, layers_ordered_output_to_input);

    // draw connections
    tegn_forbindelser(
        &mut commands,
        &mut meshes,
        &mut materials,
        genome,
        &point_per_node,
    );
    tegn_og_spawn_noder(&mut commands, meshes, materials, genome, &point_per_node);
    // commands.spawn((genome));
}

#[derive(Component, Debug)]
pub(crate) struct NodeRefForDrawing {
    node: Arc<NodeGene>,
}
#[derive(Component, Debug)]
pub(crate) struct DrawingTag;

#[derive(Component, Debug)]
pub(crate) struct NodeLabelTag;

pub(crate) fn oppdater_node_tegninger(
    mut query: Query<(
        &mut MeshMaterial2d<ColorMaterial>,
        &NodeRefForDrawing,
        &Children,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut text_query: Query<&mut Text2d, With<NodeLabelTag>>,
) {
    // println!("oppdater_node_tegninger");
    for (mut nodeforbindelse, noderef, mut children) in query.iter_mut() {
        // dbg!(&noderef);
        // println!("&&noderef.node addr {:p}", &&&noderef.node);
        // println!(" en ");
        let node_value = { noderef.node.value.read().unwrap().clone() };
        // dbg!(node_value);
        let a_color = get_color_for_node_value(node_value);
        let new_handle: Handle<ColorMaterial> = materials.add(a_color);
        nodeforbindelse.0 = new_handle;

        let child_entity = children[0];
        let mut text = text_query.get_mut(child_entity).unwrap();
        text.0 = node_value.clone().round_to_decimal(3).to_string();
        // println!("endrer verdi til {}", node_value.clone().to_string());
    }
    // println!("oppdater_node_tegninger ferdig");
}

pub(crate) fn remove_drawing_of_network_for_best_individ(
    mut commands: Commands,
    mut query: Query<Entity, With<DrawingTag>>,
) {
    // println!("inne i remove_drawing_of_network_for_best_individ");
    for (entity) in query.iter_mut() {
        // dbg!(entity);
        commands.entity(entity).despawn_recursive();
    }
    // println!("ut av remove_drawing_of_network_for_best_individ");
}

trait Round {
    fn round_to_decimal(self, decimals: u32) -> f32;
}

impl Round for f32 {
    fn round_to_decimal(self, decimals: u32) -> f32 {
        let y = 10i32.pow(decimals) as f32;
        (self * y).round() / y
    }
}

fn round(x: f32, decimals: u32) -> f32 {
    let y = 10i32.pow(decimals) as f32;
    (x * y).round() / y
}

fn tegn_og_spawn_noder(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    genome: &Genome,
    point_per_node: &HashMap<Arc<NodeGene>, Vec2>,
) {
    for node in genome.node_genes.iter() {
        let Some(point) = point_per_node.get(&Arc::clone(node)) else {
            return;
        };

        let point = point_per_node[&Arc::clone(node)];
        let node_value = { node.value.read().unwrap().clone() };
        let a_color = get_color_for_node_value(node_value);

        commands
            .spawn((
                Mesh2d(if (node.outputnode) {
                    meshes.add(RegularPolygon::new(25.0, 6))
                } else if node.inputnode {
                    meshes.add(RegularPolygon::new(25.0, 8))
                } else {
                    meshes.add(Circle { radius: 25.0 }).clone()
                }),
                MeshMaterial2d(materials.add(a_color)),
                Transform::from_xyz(
                    // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
                    // -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                    point.x, point.y, 1.0,
                ),
                NodeRefForDrawing {
                    node: Arc::clone(node),
                },
                DrawingTag,
                // Text::from_section("START!!!", text_style.clone()).with_justify(text_justification),
            ))
            .with_children(|builder| {
                builder.spawn((
                    Text2d::new({ node_value.round_to_decimal(3).to_string() }),
                    TextLayout::new_with_justify(JustifyText::Center),
                    // , text_style.clone())                        .with_justify(text_justification),
                    Transform::from_xyz(0.0, 0.0, 2.0),
                    NodeLabelTag,
                ));
                // IndividLabelText,
            });
    }
}

fn get_color_for_node_value(node_value: f32) -> Color {
    // let gradient = ColorCurve::new([RED, GREEN, BLUE]);
    // let color = gradient.unwrap().graph().sample(0.4).unwrap().1;
    let a_color = Color::linear_rgb(
        if node_value > 0.0 { node_value } else { 0. },
        // if node_value.abs() > 0.999 { 0.0 } else if  node_value.abs() < 0.1 { 0.2 } else { 0.1 },
        if node_value.abs() > 0.999 { 0.0 } else { 0.1 },
        // 0.1,
        if node_value < 0.0 {
            node_value.abs()
        } else {
            0.
        },
    )
    .with_alpha(1.0);
    a_color
    // return color;
}

fn tegn_forbindelser(
    commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    genome: &Genome,
    point_per_node: &HashMap<Arc<NodeGene>, Vec2>,
) {
    for weight in genome.weight_genes.iter() {
        let first_point = point_per_node.get(&Arc::clone(&weight.kildenode));
        let second_point = point_per_node.get(&Arc::clone(&weight.destinasjonsnode));
        // nodes that never impacts outputs are not placed in a layer, and sort of irrelevant for our network
        if first_point.is_some() && second_point.is_some() {
            let points = vec![first_point.unwrap(), second_point.unwrap()];

            let x_diff = (points[0].x - points[1].x);
            let y_diff = (points[0].y - points[1].y);
            let length = (x_diff.powi(2) + y_diff.powi(2)).sqrt();
            let angle = y_diff.atan2(x_diff);
            let middle_point = Vec2::new(
                (points[0].x + points[1].x) * 0.5,
                (points[0].y + points[1].y) * 0.5,
            );
            // println!("x_diff {},y_diff {},angle {},", x_diff, y_diff, angle);

            let vekt_value = genome.weight_genes[0].value.clone();

            let vekt_color = Color::linear_rgb(
                if vekt_value > 0.0 { vekt_value } else { 0. },
                0.1,
                if vekt_value < 0.0 {
                    vekt_value.abs()
                } else {
                    0.
                },
            )
            .with_alpha(1.);

            commands
                .spawn((
                    // mesh:   Mesh2dHandle(meshes.add(Polyline2d::new( vec![ Vec2::new(50.0, 100.0)]))),
                    Mesh2d(meshes.add(Rectangle::new(length, 10.0))),
                    MeshMaterial2d(materials.add(vekt_color)),
                    Transform::from_xyz(
                        // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
                        // -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                        middle_point.x,
                        middle_point.y,
                        0.0,
                        // ).rotate_z(10.0), //(Quat::from_axis_angle()),
                        // ).with_rotation(Quat::from_rotation_z(0.125 * std::f32::consts::PI)),
                    )
                    .with_rotation(Quat::from_rotation_z(angle)),
                    DrawingTag,
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text2d::new({
                            // a_b_vekt.weight_value.clone().to_string()
                            weight.value.clone().round_to_decimal(3).to_string()
                        }),
                        TextLayout::new_with_justify(JustifyText::Center),
                        // , text_style.clone())                    .with_justify(text_justification),
                        Transform::from_xyz(0.0, 0.0, 2.0)
                            .with_rotation(Quat::from_rotation_z(-angle)),
                        // IndividLabelText,
                    ));
                    builder.spawn((
                        Mesh2d(meshes.add(Triangle2d::new(
                            Vec2::Y * 10.0,
                            Vec2::new(-10.0, -10.0),
                            Vec2::new(10.0, -10.0),
                        ))),
                        MeshMaterial2d(materials.add(vekt_color)),
                        // , text_style.clone())                    .with_justify(text_justification),
                        Transform::from_xyz(0.0, 0.0, -1.0)
                            // .with_rotation(Quat::from_rotation_z(-angle ))
                            // .with_rotation(Quat::from_rotation_z(0.0 ))
                            .with_rotation(Quat::from_rotation_z(PI * 0.5)),
                        // IndividLabelText,
                    ));
                });
        }
    }
}

// todo ha tegning og nettverk hente fra samme sted. Kanskje flytte dette til en phenotypeLayers/ pheonotypeNeuralNetwork og tegne det istedenfor Genome

// fn lag_lag_av_nevroner_sortert_fra_output(genome: &Genome, weights_per_desination_node: &HashMap<Arc<NodeGene>, Vec<&WeightGene>>) -> (HashMap<Arc<NodeGene>, i32>, Vec<Vec<Arc<NodeGene>>>) {
fn lag_lag_av_nevroner_sortert_fra_output(
    genome: &Genome,
    weights_per_desination_node: &HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>>,
) -> (HashMap<Arc<NodeGene>, i32>, Vec<Vec<Arc<NodeGene>>>) {
    let output_nodes: Vec<Arc<NodeGene>> = genome
        .node_genes
        .clone()
        .iter()
        .filter(|node| node.outputnode)
        .map(|node| Arc::clone(node))
        .collect();

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
    let mut node_to_vekt_som_flyttet_på_noden: HashMap<Arc<NodeGene>, Vec<&WeightGene>> =
        HashMap::new();
    let mut node_to_vekt_som_flyttet_på_noden: HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>> =
        HashMap::new();
    // let next_layer = få_neste_lag(&weights_per_desination_node, &mut node_to_layer, &mut layers_ordered_output_to_input, &mut node_to_vekt_som_flyttet_på_noden, 1);
    // layers_ordered_output_to_input.push(next_layer);

    let mut layer_index = 1;
    loop {
        // dbg!(&layer_index);
        let next_layer = få_neste_lag(
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
    (node_to_layer, layers_ordered_output_to_input)
}

fn få_neste_lag<'a>(
    weights_per_desination_node: &HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>>,
    // weights_per_desination_node: &HashMap<Arc<NodeGene>, Vec<&'a WeightGene>>,
    layer_per_node: &mut HashMap<Arc<NodeGene>, i32>,
    layers_output_to_input: &mut Vec<Vec<Arc<NodeGene>>>,
    node_to_vekt_som_flyttet_på_noden: &mut HashMap<Arc<NodeGene>, Vec<Arc<WeightGene>>>,
    // node_to_vekt_som_flyttet_på_noden: &mut HashMap<Arc<NodeGene>, Vec<&'a WeightGene>>,
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
        let mut vekter_allerede_brukt =
            match node_to_vekt_som_flyttet_på_noden.get_mut(&Arc::clone(node)) {
                None => Vec::new(),
                Some(liste) => liste.clone(),
            };
        // dbg!(&node);
        match weights_per_desination_node.get(node) {
            Some(weights) => {
                for weight in weights {
                    if !vekter_allerede_brukt.contains(weight) {
                        next_layer.push(Arc::clone(&weight.kildenode));
                        layer_per_node.insert(Arc::clone(&weight.kildenode), lag_index);
                        // vekter_allerede_brukt.push(weight);
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

fn få_vekter_per_kildenode(genome: &Genome) -> HashMap<Arc<NodeGene>, Vec<&WeightGene>> {
    let mut weights_per_kildenode: HashMap<Arc<NodeGene>, Vec<&WeightGene>> = HashMap::new();
    for weight in genome.weight_genes.iter() {
        let list = weights_per_kildenode
            .entry(Arc::clone(&weight.kildenode))
            .or_insert_with(|| Vec::new());
        // list.push(Arc::clone(&weight));
        list.push(weight);
    }
    weights_per_kildenode
}

fn kordinater_per_node(
    genome: &Genome,
    layer_per_node: HashMap<Arc<NodeGene>, i32>,
    layers_output_to_input: Vec<Vec<Arc<NodeGene>>>,
) -> HashMap<Arc<NodeGene>, Vec2> {
    let mut point_per_node = HashMap::new();

    let mut x_output_layer = -150.0;
    let distanc_x = -200.0;
    for node in genome.node_genes.iter() {
        match (layer_per_node.get(node)) {
            Some(layer) => {
                let index = layers_output_to_input[*layer as usize]
                    .iter()
                    .position(|n| n == node)
                    .expect("skal finnes");
                let y = index * 100;
                let x = x_output_layer + *layer as f32 * distanc_x;
                point_per_node.insert(Arc::clone(node), Vec2::new(x.clone(), y as f32));
            }
            // nodes that never impacts outputs are not placed in a layer
            _ => {}
        }
    }
    point_per_node
}
