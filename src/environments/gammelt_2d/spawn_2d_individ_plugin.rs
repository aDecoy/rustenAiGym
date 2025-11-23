use crate::environments::gammelt_2d::moving_plank_with_user_input_2d_plugin::{MovingPlankObservation, PLANK_HIGHT, PLANK_LENGTH};
use crate::evolusjon::evolusjon_steg_plugin::{SpawnNewIndividualMessage, SpawnNewIndividualWithGenomeMessage};
use crate::evolusjon::hjerne_fenotype::PhenotypeNeuralNetwork;
use crate::evolusjon::phenotype_plugin::{Individ, IndividFitnessLabelTextTag, PlankPhenotype};
use crate::genome::genome_stuff::{Genome, InnovationNumberGlobalCounter, new_random_genome};
use crate::monitoring::camera_stuff::RENDER_LAYER_ALLE_INDIVIDER;
use crate::{ACTIVE_ENVIROMENT, EnvValg};
use avian2d::prelude::{Collider, CollisionLayers, LinearVelocity, RigidBody};
use bevy::asset::io::ErasedAssetReader;
use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::basic::PURPLE;
use bevy::color::palettes::tailwind::CYAN_300;
use bevy::prelude::*;

pub struct Spawn2dIndividPlugin;

impl Plugin for Spawn2dIndividPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnNewIndividualMessage>()
            .add_systems(
                Startup,
                (spawn_new_2d_individ_meldingsspiser.after(crate::evolusjon::evolusjon_steg_plugin::spawn_start_population)),
            )
            .add_systems(
                Update,
                (
                    create_genome_and_send_spawn_message,
                    // spawn_new_2d_individ_meldingsspiser.after(create_genome_and_send_spawn_message),
                ),
            );
    }
}

fn create_genome_and_send_spawn_message(
    mut innovation_number_global_counter: ResMut<InnovationNumberGlobalCounter>,
    mut spawn_new_individual_message_reader: MessageReader<SpawnNewIndividualMessage>,
    mut spawn_new_individual_message_writer: MessageWriter<SpawnNewIndividualWithGenomeMessage>,
) {
    for message in spawn_new_individual_message_reader.read() {
        let genome = match ACTIVE_ENVIROMENT {
            EnvValg::HomingGroudY => new_random_genome(1, 1, &mut innovation_number_global_counter),
            _ => new_random_genome(2, 2, &mut innovation_number_global_counter),
        };

        spawn_new_individual_message_writer.write(SpawnNewIndividualWithGenomeMessage { new_genome: genome });
    }
}

fn spawn_new_2d_individ_meldingsspiser(
    mut spawn_new_individual_message: MessageReader<SpawnNewIndividualWithGenomeMessage>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(PLANK_LENGTH, PLANK_HIGHT));
    let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE).with_alpha(0.5));
    let hover_matl = materials.add(Color::from(CYAN_300));

    // todo virker som at ikke kan dereff eller flyyytte på meldinger siden flere systemer kan konsumere de i paralell. Kan derfor ikke ta eierskap https://docs.rs/bevy/latest/bevy/ecs/message/struct.Messages.html

    for message in spawn_new_individual_message.read() {
        // for mesage in spawn_new_individual_message.read().into_inner() {
        // for mesage in spawn_new_individual_message.read().into_iter() {
        // let message = mesage2;
        let new_genome: Genome = message.new_genome.clone();

        match ACTIVE_ENVIROMENT {
            EnvValg::Fall | EnvValg::FallVelocityHøyre => commands.spawn(create_plank_env_falling(
                material_handle.clone(),
                rectangle_mesh_handle.clone().into(),
                Vec3 {
                    x: 0.0,
                    y: -150.0 + 3.3 * 50.0,
                    // y: -150.0 + (n as f32 * 15.0),
                    z: 1.0,
                },
                new_genome,
            )),
            EnvValg::Høyre => commands.spawn(create_plank_env_moving_right(
                material_handle.clone(),
                rectangle_mesh_handle.clone().into(),
                Vec3 {
                    x: 0.0,
                    y: -150.0 + 3.3 * 50.0,
                    // y: -150.0 + message.n as f32 * 50.0,
                    z: 1.0,
                },
                new_genome,
            )),
            EnvValg::FallExternalForcesHøyre | EnvValg::Homing | EnvValg::HomingGroud | EnvValg::HomingGroudY => commands.spawn(create_plank_ext_force_env_falling(
                material_handle.clone(),
                rectangle_mesh_handle.clone().into(),
                Vec3 {
                    x: 0.0,
                    y: -150.0 + 3.3 * 50.0,
                    z: 0.0,
                },
                // Vec3 { x: 30.0, y: 100.0, z: 1.0 },
                new_genome,
            )),
        }
        .with_children(|builder| {
            builder.spawn((
                Text2d::new("Fitness label"),
                TextLayout::new_with_justify(Justify::Center),
                Transform::from_xyz(0.0, 0.0, 2.0),
                IndividFitnessLabelTextTag,
                RenderLayers::layer(1),
            ));
        });
    }
}

pub fn create_plank_env_moving_right(
    material_handle: Handle<ColorMaterial>,
    mesh2d_handle: Handle<Mesh>,
    start_position: Vec3,
    genome: Genome,
) -> (
    Mesh2d,
    Transform,
    MeshMaterial2d<ColorMaterial>,
    PlankPhenotype,
    Genome,
    Collider,
    MovingPlankObservation,
    LinearVelocity,
) {
    (
        Mesh2d(mesh2d_handle),
        Transform::from_translation(start_position),
        // .with_scale(Vec2 { x: PLANK_LENGTH, y: PLANK_HIGHT }.extend(1.)),
        MeshMaterial2d(material_handle),
        PlankPhenotype {
            score: 0.0,
            obseravations: vec![0.0, 0.0],
            // phenotype_layers: create_phenotype_layers(genome.clone()),
            phenotype_layers: PhenotypeNeuralNetwork::new(&genome),
            // genotype: genome_entity,
        }, // alt 1
        genome,
        // Collider::cuboid(0.5, 0.5),
        Collider::rectangle(0.5, 0.5),
        MovingPlankObservation { x: 0.0, y: 0.0 }, // alt 2,
        // RigidBody::Dynamic,
        // individ, // taged so we can use queryies to make evolutionary choises about the individual based on preformance of the phenotype
        // Velocity {
        //     // linvel: Vec2::new(100.0, 2.0),
        //     linvel: Vec2::new(0.0, 0.0),
        //     angvel: 0.0,
        // },
        LinearVelocity { 0: Vec2::new(0.0, 0.0) },
    )
}

pub fn create_plank_env_falling(
    material_handle: Handle<ColorMaterial>,
    mesh2d_handle: Handle<Mesh>,
    start_position: Vec3,
    genome: Genome,
) -> (
    Mesh2d,
    Transform,
    MeshMaterial2d<ColorMaterial>,
    PlankPhenotype,
    Genome,
    Collider,
    RigidBody,
    CollisionLayers,
    LinearVelocity,
) {
    (
        Mesh2d(mesh2d_handle),
        Transform::from_translation(start_position).with_scale(Vec2 { x: PLANK_LENGTH, y: PLANK_HIGHT }.extend(1.)),
        MeshMaterial2d(material_handle),
        PlankPhenotype {
            score: 0.0,
            obseravations: vec![0.0, 0.0],
            // phenotype_layers:  create_phenotype_layers(genome.clone()),
            phenotype_layers: PhenotypeNeuralNetwork::new(&genome),
            // genotype: genome_entity,
        }, // alt 1
        genome,
        Collider::rectangle(1.0, 1.0),
        // Collider::cuboid(0.5, 0.5),
        RigidBody::Dynamic,
        // MovingPlankObservation { x: 0.0, y: 0.0 }, // alt 2,
        // CollisionGroups::new(
        //     // almost looked like it runs slower with less collisions?
        //     // Kan være at det bare er mer ground kontakt, siden alle ikke hvilker på en blokk som er eneste som rører bakken
        //     Group::GROUP_1,
        //     if INDIVIDUALS_COLLIDE_IN_SIMULATION { Group::GROUP_1 } else {
        //         Group::GROUP_2
        //     },
        // ),
        CollisionLayers::new(0b0001, 0b0010),
        // Velocity {
        //     // linvel: Vec2::new(100.0, 2.0),
        //     linvel: Vec2::new(0.0, 0.0),
        //     angvel: 0.0,
        // },
        LinearVelocity { 0: Vec2::new(0.0, 0.0) },
    )
}
pub fn create_plank_ext_force_env_falling(
    material_handle: Handle<ColorMaterial>,
    mesh2d_handle: Handle<Mesh>,
    start_position: Vec3,
    genome: Genome,
) -> (
    Mesh2d,
    MeshMaterial2d<ColorMaterial>,
    Transform,
    PlankPhenotype,
    Genome,
    Collider,
    RigidBody,
    CollisionLayers,
    LinearVelocity,
    // Forces,
    TextLayout,
    RenderLayers,
    Individ,
) {
    // let text_style = TextStyle {
    //     font_size: 30.0,
    //     color: Color::   WHITE,
    //     ..default()
    // };
    (
        Mesh2d(mesh2d_handle),
        MeshMaterial2d(material_handle),
        Transform::from_translation(start_position).with_scale(Vec2 { x: 1.0, y: 1.0 }.extend(1.)),
        PlankPhenotype {
            score: 0.0,
            obseravations: vec![0.0, 0.0],
            // phenotype_layers: create_phenotype_layers(genome.clone()),
            phenotype_layers: PhenotypeNeuralNetwork::new(&genome),
            // genotype: genome_entity,
        }, // alt 1
        genome,
        // Collider::rectangle(1.0, 1.0),
        Collider::rectangle(PLANK_LENGTH, PLANK_HIGHT),
        RigidBody::Dynamic,
        CollisionLayers::new(0b0001, 0b0010),
        LinearVelocity { 0: Vec2::new(0.0, 0.0) },
        // Forces { force: Vec2::new(0.0, 0.0), persistent: false , ..default()} ,
        // Forces::new(Vec2::X).with_persistence(false),
        TextLayout::new_with_justify(Justify::Center),
        RenderLayers::layer(RENDER_LAYER_ALLE_INDIVIDER),
        Individ {},
        // RenderLayers::from_layers(&[1]),
    )
}
