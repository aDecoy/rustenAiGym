use crate::Kjøretilstand;
use crate::environments::gammelt_2d::moving_plank_with_user_input_2d_plugin::{PLANK_HIGHT, PLANK_LENGTH};
use crate::evolusjon::evolusjon_steg_plugin::PopulationIsSpawnedMessage;
use crate::evolusjon::phenotype_plugin::{IndividFitnessLabelText, IndividFitnessLabelTextTag, PhentypeAndGenome, PlankPhenotype};
use crate::genome::genome_stuff::Genome;
use crate::monitoring::camera_stuff::{PopulasjonMenyCameraTag, RENDER_LAYER_POPULASJON_MENY};
use crate::monitoring::in_focus_stuff::{IndividInFocus, IndividInFocusСhangedEvent};
use crate::populasjon_handlinger::population_sammenligninger::get_population_sorted_from_best_to_worst_v2;
use bevy::app::App;
use bevy::asset::{Assets, Handle};
use bevy::camera::visibility::RenderLayers;
use bevy::color::Color;
use bevy::color::palettes::basic::{PURPLE, RED};
use bevy::color::palettes::tailwind::RED_300;
use bevy::picking::Pickable;
use bevy::prelude::*;

pub struct HyllerepresentasjonPlugin;

impl Plugin for HyllerepresentasjonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostStartup,
            (
                setup_population_meny_etter_populasjon_spawned_in, // todo , trengr å oppdatere meny også
            ),
        )
        .add_systems(
            Update,
            (
                setup_population_meny_etter_populasjon_spawned_in, // todo ikke sikker på om jeg også må slette gammelt når ny generasjon har spawnet inn...
                label_plank_with_current_score_in_meny,
                // eventer hvis individ i fokus skifter
            )
                .chain()
                .run_if(in_state(Kjøretilstand::Kjørende)),
        );
    }
}

fn place_in_focus_from_meny(
    focus_trigger_click: On<Pointer<Click>>,
    mut commands: Commands,
    old_focus_query: Query<Entity, With<IndividInFocus>>,
    // mut individ_query: Query<Entity, With<Genome>>,
    meny_individ_box_query: Query<(Entity, &MenyTagForIndivid)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Hvis jeg kan få X        fra Y                         => { Gjør dette med  X }
    if let Ok(old_focus) = old_focus_query.single() {
        commands.entity(old_focus).remove::<IndividInFocus>();
        // back to default color
        let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE));
        commands.entity(old_focus).insert(MeshMaterial2d(material_handle));
    }

    let meny_bokks_for_individ_entity = meny_individ_box_query.get(focus_trigger_click.target()).unwrap();
    let individ_entity = meny_bokks_for_individ_entity.1.individ_entity;

    if let Ok(mut invidiv_entity_commandering) = commands.get_entity(individ_entity) {
        invidiv_entity_commandering.insert(IndividInFocus);
        commands.send_event(IndividInFocusСhangedEvent { entity: individ_entity });
    }
}
#[derive(Component, Debug)]
struct MenyTagForIndivid {
    individ_entity: Entity,
}

fn label_plank_with_current_score_in_meny(mut query: Query<(&mut TextSpan, &IndividFitnessLabelText)>, phenotype_query: Query<&PlankPhenotype>) {
    // println!("endrer TextSpan for meny-bokks med fitness");
    for (mut span, individ_fitness_label_text) in query.iter_mut() {
        if let Ok(pheontype) = phenotype_query.get(individ_fitness_label_text.entity) {
            **span = format!("{:.5}", pheontype.score);
        }
    }
}

fn setup_population_meny_etter_populasjon_spawned_in(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &PlankPhenotype, &Genome)>,
    camera_query: Query<(Entity, &Camera), With<PopulasjonMenyCameraTag>>,
    mut population_done_spawning_in: MessageReader<PopulationIsSpawnedMessage>,
) {
    if !population_done_spawning_in.is_empty() {
        println!("kjører setup_population_meny_etter_populasjon_spawned_in");
        let population: Vec<PhentypeAndGenome> = get_population_sorted_from_best_to_worst_v2(query.iter()).clone();
        // let best = population[0].clone();

        let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(PLANK_LENGTH, PLANK_HIGHT));
        let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE));
        // commands.spawn((
        //     Mesh2d(rectangle_mesh_handle),
        //     MeshMaterial2d(material_handle),
        //     RenderLayers::layer(RENDER_LAYER_POPULASJON),
        // ));
        if let Ok((camera_entity, camera)) = camera_query.single() {
            // root node
            commands
                .spawn((
                    Node {
                        width: Val::Percent(90.0),
                        height: Val::Percent(90.0),
                        // left: px(10.0),
                        // right: px(10.0),
                        // justify_content: JustifyContent::SpaceBetween,
                        // justify_content: JustifyContent::Stretch,
                        justify_content: JustifyContent::SpaceEvenly,
                        flex_direction: FlexDirection::Column,
                        // justify_content: JustifyContent::Center,
                        ..default()
                    },
                    Outline::new(Val::Px(10.), Val::ZERO, RED.into()),
                    UiTargetCamera(camera_entity), // UiTargetCamera brukes for UI ting. Ser ut til at bare trenger den på top noden.
                                                   // Bevy UI doesn't support `RenderLayers`. Each UI layout can only have one render target, selected using `UiTargetCamera`
                ))
                .with_children(|parent| {
                    // kolonner som fyller hele veien ned
                    // todo kanskje trenger en knapp rad i et eget vindu. Vil jo kanskje minimere vekk menyen og neruron tegning
                    // meny knapper div
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            // justify_content: JustifyContent::SpaceEvenly,
                            justify_content: JustifyContent::SpaceBetween,
                            // align_items: AlignItems::Center,
                            // width: Val::Px(700.),
                            ..default()
                        })
                        .with_children(|parent| {
                            // knapp 1
                            parent.spawn((
                                Node {
                                    width: Val::Px(100.),
                                    height: Val::Px(50.),
                                    // border: UiRect::all(Val::Px(100.)),
                                    // margin: UiRect::all(Val::Px(10.)),
                                    overflow: Overflow::scroll_y(),
                                    ..default()
                                },
                                TextFont::default(),
                                Text::new("en knapp"),
                                BackgroundColor(Color::from(RED_300)),
                                // RenderLayers::layer(RENDER_LAYER_POPULASJON_MENY), // https://github.com/bevyengine/bevy/issues/12461
                                // UiTargetCamera(camera_entity),  // Target camera brukes for UI ting
                            ));
                            // knapp 2
                            parent.spawn((
                                Node {
                                    width: Val::Px(100.),
                                    height: Val::Px(50.),
                                    // border: UiRect::all(Val::Px(100.)),
                                    // margin: UiRect::all(Val::Px(10.)),
                                    overflow: Overflow::scroll_y(),
                                    ..default()
                                },
                                TextFont::default(),
                                Text::new("en knapp til"),
                                BackgroundColor(Color::from(RED_300)),
                                // RenderLayers::layer(RENDER_LAYER_POPULASJON_MENY), // https://github.com/bevyengine/bevy/issues/12461
                                // UiTargetCamera(camera_entity),
                            ));
                        });

                    // populasjon_handlinger grid
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            // justify_content: JustifyContent::SpaceEvenly,
                            justify_content: JustifyContent::SpaceBetween,
                            // align_items: AlignItems::Center,
                            // width: Val::Px(700.),
                            ..default()
                        })
                        .with_children(|parent| {
                            // EN BOKS PER INDIVID
                            for phenotype_and_genome in population {
                                let ancestor_id = phenotype_and_genome.genome.original_ancestor_id;
                                let fitness_score = phenotype_and_genome.phenotype.score;
                                parent
                                    .spawn((
                                        Node {
                                            width: Val::Px(100.),
                                            height: Val::Px(100.),
                                            // border: UiRect::all(Val::Px(100.)),
                                            margin: UiRect::all(Val::Px(10.)),
                                            overflow: Overflow::scroll_y(),
                                            ..default()
                                        },
                                        TextFont::default(),
                                        BackgroundColor(Color::srgb(0.65, 0.65, 0.65)),
                                        RenderLayers::layer(RENDER_LAYER_POPULASJON_MENY), // https://github.com/bevyengine/bevy/issues/12461
                                        MenyTagForIndivid {
                                            individ_entity: phenotype_and_genome.entity,
                                        },
                                    ))
                                    .observe(place_in_focus_from_meny)
                                    // Tekst som er inne i den grå bokksen
                                    .with_children(|parent| {
                                        parent
                                            .spawn((
                                                // NB: Tekst inside of NODE can not be text2d. Text2d does not care about UI grid stuff
                                                Text::new(format!("Ancestor_id {ancestor_id}, score:  ")),
                                                TextFont::from_font_size(10.0),
                                                RenderLayers::layer(RENDER_LAYER_POPULASJON_MENY), // https://github.com/bevyengine/bevy/issues/12461
                                                Pickable::IGNORE,
                                            ))
                                            .with_child((
                                                TextFont::from_font_size(15.0),
                                                TextSpan::default(), // tekst er inne i textSpan når den er en child, og ting blir tullete om det er inne i en text
                                                IndividFitnessLabelTextTag,
                                                IndividFitnessLabelText {
                                                    entity: phenotype_and_genome.entity,
                                                },
                                                RenderLayers::layer(RENDER_LAYER_POPULASJON_MENY), // https://github.com/bevyengine/bevy/issues/12461
                                                Pickable::IGNORE,
                                            ));
                                    });
                            }
                        });

                    // en annen kolonne

                    // parent
                    //     .spawn((
                    //         Node {
                    //             width: Val::Px(20.),
                    //             border: UiRect::all(Val::Px(2.)),
                    //             ..default()
                    //         },
                    //         BackgroundColor(Color::srgb(0.65, 0.65, 0.65)),
                    //         RenderLayers::layer(RENDER_LAYER_POPULASJON),
                    //     ));
                });
        }
        population_done_spawning_in.read();
    }
}
