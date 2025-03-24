use crate::environments::camera_stuff::{AllIndividerWindowTag, RENDER_LAYER_ALLE_INDIVIDER, RENDER_LAYER_TOP_BUTTON_MENY};
use crate::Kjøretilstand;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;
use bevy::window::WindowResized;

pub struct SimulationRunningTellerPlugin;

impl Plugin for SimulationRunningTellerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SimulationTotalRuntimeRunningTeller>()
            .init_resource::<SimulationGenerationTeller>()
            .insert_resource(SimulationGenerationTimer { main_timer: Timer::from_seconds(GENERATION_TIME, TimerMode::Repeating) })
            .add_systems(Startup, spawn_simulation_tellertekst)
            .add_systems(Startup, spawn_simulation_generation_time_tellertekst)
            .add_systems(Startup, spawn_simulation_timer_tekst)
            .add_systems(Update, ((
                add_one_to_simulation_running_teller::<SimulationTotalRuntimeRunningTeller>,
                oppdater_bevy_simulation_tellertekst::<SimulationTotalRuntimeRunningTellerTekst, SimulationTotalRuntimeRunningTeller>,
                resize_simulation_tellertekst::<SimulationTotalRuntimeRunningTellerTekst>,
                timer_tick,
                oppdater_simulation_timer_tekst::<SimulationGenerationRunningTimerTekst>,
            ).chain()).run_if(in_state(Kjøretilstand::Kjørende)),
            );
    }
}

static GENERATION_TIME: f32 = 10.0;
// static GENERATION_TIME: f32 = 5.0;

#[derive(Component)]
pub struct SimulationTotalRuntimeRunningTellerTekst;

#[derive(Component)]
pub struct SimulationGenerationRunningTellerTekst;

#[derive(Resource, Default, Debug)]
pub struct SimulationTotalRuntimeRunningTeller {
    pub(crate) count: u32,
}

#[derive(Resource, Default, Debug)]
struct SimulationGenerationTeller {
    pub(crate) count: u32,
}


trait CounterResource {
    fn counter_count_value(&self) -> u32;
    fn increment_counter_by_one(&mut self);
    fn increment_counter_by_time_delta(&self) -> u32;
}

impl CounterResource for SimulationTotalRuntimeRunningTeller {
    fn counter_count_value(&self) -> u32 {
        self.count
    }

    fn increment_counter_by_one(&mut self) {
        self.count += 1;
    }

    fn increment_counter_by_time_delta(&self) -> u32 {
        todo!()
    }
}

pub(crate) fn spawn_simulation_tellertekst(mut commands: Commands, window: Query<&Window, With<AllIndividerWindowTag>>) {
    let window = window.single();

    // let text_style = TextStyle {
    //     font_size: 30.0,
    //     color: Color::WHITE,
    //     ..default()
    // };
    let text_justification = JustifyText::Center;
    commands.spawn((
        Text2d::new("START"),
        TextLayout::new_with_justify(JustifyText::Center),
        // Transform::from_xyz(250.0, 250.0, 0.0),
        Transform::from_xyz(window.width() * 0.5 - 200.0, window.height() * 0.5 - 50.0, 0.0),
        // global_GlobalTransform::from_xyz(0.0, 0.0, 0.0),
        SimulationTotalRuntimeRunningTellerTekst,
        RenderLayers::from_layers(&[RENDER_LAYER_ALLE_INDIVIDER]),
    ));
}

pub(crate) fn spawn_simulation_generation_time_tellertekst(mut commands: Commands,
                                                           window_query: Query<&Window, With<AllIndividerWindowTag>>
) {
    let window = window_query.single();
    // let text_style = TextStyle {
    //     font_size: 30.0,
    //     color: Color::WHITE,
    //     ..default()
    // };
    let text_justification = JustifyText::Center;
    commands.spawn((
        Text2d::new("START"),
        TextLayout::new_with_justify(JustifyText::Center),
        // Transform::from_xyz(250.0, 250.0, 0.0),
        Transform::from_xyz(window.width() * 0.5 - 200.0, window.height() * 0.5 - 50.0, 0.0),
        // global_GlobalTransform::from_xyz(0.0, 0.0, 0.0),

        SimulationGenerationRunningTellerTekst,
        RenderLayers::from_layers(&[RENDER_LAYER_ALLE_INDIVIDER]),
    ));
}

fn resize_simulation_tellertekst<TellerTekst: bevy::prelude::Component>(resize_event: Res<Events<WindowResized>>, mut query: Query<&mut Transform, With<TellerTekst>>) {
    let mut reader = resize_event.get_reader();
    for e in reader.read(&resize_event) {
        let mut transform = query.single_mut();
        transform.translation.x = e.width * 0.5 - 200.0;
        transform.translation.y = e.height * 0.5 - 20.0;
    }
}

pub(crate) fn oppdater_bevy_simulation_tellertekst<TellerTekst: bevy::prelude::Component, Teller: CounterResource + bevy::prelude::Resource>(mut query: Query<&mut Text2d, With<TellerTekst>>,
                                                                                                                                             teller1: Res<Teller>, ) {
    // println!("query empty={}, query size = {}", query.is_empty(), query.iter().count());
    let mut tekst = query.single_mut();
    // tekst.sections[0].value = "En fin tekst: ".to_string() + &teller1.0.to_string();
    tekst.0 = "Simulation Counter: ".to_string() + &teller1.counter_count_value().to_string();
}

pub(crate) fn add_one_to_simulation_running_teller<Teller: CounterResource + bevy::prelude::Resource>(mut frame_count: ResMut<Teller>) {
    frame_count.increment_counter_by_one();

    // For fremtiden når jeg kanskje vil se på tid istedenfor frames
    // let speed = 1.0;
    // if speed == std::f32::INFINITY {
    //     context.overstep = Duration::MAX;
    // } else {
    //     context.overstep = context.overstep.saturating_add(delta.mul_f32(speed));
    // }
}

////// Timer /////////////////


#[derive(Resource, Debug)]
pub struct SimulationGenerationTimer {
    pub main_timer: Timer,
    // trigger_time: f64,
}
#[derive(Component)]
pub struct SimulationGenerationRunningTimerTekst;

fn timer_tick(time: Res<Time>, mut countdown: ResMut<SimulationGenerationTimer>) {
    countdown.main_timer.tick(time.delta());
}

pub(crate) fn spawn_simulation_timer_tekst(mut commands: Commands, window: Query<&Window, With<AllIndividerWindowTag>>) {
    let window = window.single();

    // let text_style = TextStyle {
    //     font_size: 30.0,
    //     color: Color::WHITE,
    //     ..default()
    // };
    let text_justification = JustifyText::Center;
    commands.spawn((
        Text2d::new("START"),
        TextLayout::new_with_justify(JustifyText::Center),
        // Transform::from_xyz(250.0, 250.0, 0.0),
        // Transform::from_xyz(window.width() * 0.5 - 200.0, window.height() * 0.5 - 50.0, 0.0),
        Transform::from_xyz(-window.width() * 0.5 + 200.0, window.height() * 0.5 - 25.0, 0.0),
        SimulationGenerationRunningTimerTekst,
        RenderLayers::from_layers(&[RENDER_LAYER_ALLE_INDIVIDER]),
    ));
}
pub(crate) fn oppdater_simulation_timer_tekst<TellerTekst: bevy::prelude::Component>(mut query: Query<&mut Text2d, With<TellerTekst>>, teller1: Res<SimulationGenerationTimer>) {
    let mut tekst = query.single_mut();
    // tekst.sections[0].value = "En fin tekst: ".to_string() + &teller1.0.to_string();
    tekst.0 = "Simulation timer: ".to_string() + &teller1.main_timer.elapsed_secs().round().to_string();
}

