use bevy::color::Color;
use bevy::prelude::*;
use bevy::window::WindowResized;

use crate::Kjøretilstand;

pub struct SimulationRunningTellerPlugin;

impl Plugin for SimulationRunningTellerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SimulationRunningTeller>()
            .add_systems(Startup, spawn_simulation_tellertekst)
            .add_systems(Update, ((
                add_one_to_simulation_running_teller,
                oppdater_bevy_simulation_tellertekst,
                resize_simulation_tellertekst,
            ).chain()).run_if(in_state(Kjøretilstand::Kjørende)),
            );
    }
}
// todo pause rapier physics om det ikke er kjørende!!!1
asdfasdfasdfasdf

#[derive(Component)]
pub struct SimulationRunningTellerTekst;

#[derive(Resource, Default, Debug)]
pub struct SimulationRunningTeller{
    pub(crate) count: u32
}

pub(crate) fn spawn_simulation_tellertekst(mut commands: Commands, window: Query<&Window>) {
    let window = window.single();

    let text_style = TextStyle {
        font_size: 30.0,
        color: Color::WHITE,
        ..default()
    };
    let text_justification = JustifyText::Center;
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("START", text_style.clone()).with_justify(text_justification),
            // transform: Transform::from_xyz(250.0, 250.0, 0.0),
            transform: Transform::from_xyz(window.width() * 0.5 - 200.0, window.height() * 0.5 - 50.0, 0.0),
            // global_transform: GlobalTransform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        SimulationRunningTellerTekst,
    ));
}

fn resize_simulation_tellertekst(resize_event: Res<Events<WindowResized>>, mut query: Query<&mut Transform, With<SimulationRunningTellerTekst>>) {
    let mut reader = resize_event.get_reader();
    for e in reader.read(&resize_event) {
        let mut transform = query.single_mut();
        transform.translation.x = e.width * 0.5 - 200.0;
        transform.translation.y = e.height * 0.5 - 20.0;
    }
}

pub(crate) fn oppdater_bevy_simulation_tellertekst(mut query: Query<&mut Text, With<SimulationRunningTellerTekst>>,
                                                   teller1: Res<SimulationRunningTeller>, ) {
    let mut tekst = query.single_mut();
    // tekst.sections[0].value = "En fin tekst: ".to_string() + &teller1.0.to_string();
    tekst.sections[0].value = "Simulation Counter: ".to_string() + &teller1.count.to_string();
}

pub(crate) fn add_one_to_simulation_running_teller(mut frame_count: ResMut<SimulationRunningTeller>) {
    frame_count.count += 1;

    // For fremtiden når jeg kanskje vil se på tid istedenfor frames
    // let speed = 1.0;
    // if speed == std::f32::INFINITY {
    //     context.overstep = Duration::MAX;
    // } else {
    //     context.overstep = context.overstep.saturating_add(delta.mul_f32(speed));
    // }
}