use crate::monitoring::camera_stuff::{KnapperMenyCameraTag, resize_alle_individer_camera};
use bevy::app::{App, Plugin};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::color::palettes::basic::RED;
use bevy::color::palettes::tailwind::RED_800;
use bevy::prelude::*;

pub struct KnappMenyPlugin;

impl Plugin for KnappMenyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_knapp_meny));
    }
}

fn setup_knapp_meny(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut camera_query: Query<(Entity, &Camera), With<KnapperMenyCameraTag>>,
) {
    let (camera_entity, camera) = camera_query.single().unwrap();
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceAround,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            Outline::new(Val::Px(10.), Val::ZERO, RED.into()),
            UiTargetCamera(camera_entity), // UiTargetCamera brukes for UI ting. Ser ut til at bare trenger den på top noden
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        // width: Val::Px(100.),
                        height: Val::Px(50.),
                        // border: UiRect::all(Val::Px(100.)),
                        // margin: UiRect::all(Val::Px(10.)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    TextFont::default(),
                    Text::new("Environment camera on/off"),
                    BackgroundColor(Color::from(RED_800)),
                ))
                .observe(resize_alle_individer_camera);

            // knapp 2
            parent.spawn((
                Node {
                    height: Val::Px(50.),
                    // border: UiRect::all(Val::Px(100.)),
                    // margin: UiRect::all(Val::Px(10.)),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                TextFont::default(),
                Text::new("Node tegning på/av"),
                BackgroundColor(Color::from(RED_800)),
            ));
        });
}
