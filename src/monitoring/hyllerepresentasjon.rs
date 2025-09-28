use crate::environments::moving_plank::{PLANK_HIGHT, PLANK_LENGTH};
use crate::genome::genome_stuff::Genome;
use crate::monitoring::camera_stuff::{PopulasjonMenyCameraTag, RENDER_LAYER_POPULASJON_MENY};
use crate::{
    IndividFitnessLabelText, IndividFitnessLabelTextTag, MenyTagForIndivid, PhentypeAndGenome,
    PlankPhenotype, get_population_sorted_from_best_to_worst_v2, place_in_focus_from_meny,
};
use bevy::app::App;
use bevy::asset::{Assets, Handle};
use bevy::color::Color;
use bevy::color::palettes::basic::{PURPLE, RED};
use bevy::color::palettes::tailwind::RED_300;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

struct HyllerepresentasjonPlugin;

impl Plugin for HyllerepresentasjonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                setup_population_meny, // todo , trengr å oppdatere meny også
            ),
        );
    }
}

fn setup_population_meny(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &PlankPhenotype, &Genome)>,
    camera_query: Query<(Entity, &Camera), With<PopulasjonMenyCameraTag>>,
) {
    let population: Vec<PhentypeAndGenome> =
        get_population_sorted_from_best_to_worst_v2(query.iter()).clone();
    let best = population[0].clone();

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
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    // justify_content: JustifyContent::SpaceBetween,
                    // justify_content: JustifyContent::Stretch,
                    justify_content: JustifyContent::SpaceEvenly,
                    flex_direction: FlexDirection::Column,
                    // justify_content: JustifyContent::Center,
                    ..default()
                },
                Outline::new(Val::Px(10.), Val::ZERO, RED.into()),
                // RenderLayers::layer(RENDER_LAYER_POPULASJON_MENY), // https://github.com/bevyengine/bevy/issues/12461
                UiTargetCamera(camera_entity), // UiTargetCamera brukes for UI ting. Ser ut til at bare trenger den på top noden
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
                                            Text::new(format!(
                                                "Ancestor_id {ancestor_id}, score:  "
                                            )),
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
}
