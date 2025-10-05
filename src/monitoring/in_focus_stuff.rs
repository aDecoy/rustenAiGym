use crate::evolusjon::phenotype_plugin::PlankPhenotype;
use crate::genome::genome_stuff::Genome;
use crate::monitoring::draw_network::place_in_focus;
use crate::populasjon_handlinger::population_sammenligninger::{EliteTag, get_best_elite};
use bevy::app::{App, Plugin, Startup};
use bevy::asset::{Assets, Handle};
use bevy::color::palettes::basic::{GREEN, PURPLE, RED};
use bevy::color::palettes::tailwind::CYAN_300;
use bevy::color::{Alpha, Color};
use bevy::ecs::query::QueryIter;
use bevy::prelude::*;
use std::cmp::Ordering;
// mod populasjon_handlinger;

#[derive(Debug, Component)]
pub struct IndividInFocus;

#[derive(Event, Debug)]
pub struct IndividInFocusСhangedEvent {
    pub(crate) entity: Entity,
}

pub(crate) struct InFocusPlugin;

impl Plugin for InFocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<IndividInFocusСhangedEvent>()
            // .add_systems(
            //     PostStartup,
            //     (
            //         set_new_elite_in_focus,
            //         publiser_event_for_ny_individ_i_fokus,
            //         // color_change_on_pointer_out_of_individ
            //     ),
            // )
            .add_systems(Update, (fjern_fokus_fra_gammel_elite, set_new_elite_in_focus, publiser_event_for_ny_individ_i_fokus));
    }
}
fn color_change_on_pointer_out_of_individ(
    entity_som_forlates: Trigger<Pointer<Out>>,
    mut query: Query<
        (Entity, &mut MeshMaterial2d<ColorMaterial>, Option<&EliteTag>, Option<&IndividInFocus>),
        // With<(Individ, PlankPhenotype)>,
        With<(PlankPhenotype)>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let material_default: Handle<ColorMaterial> = materials.add(Color::from(PURPLE).with_alpha(0.5));
    let material_elite = materials.add(Color::from(RED));
    let material_in_focus = materials.add(Color::from(GREEN));

    if let Ok((entity, mut material_handle, elite, in_focus)) = query.get_mut(entity_som_forlates.target) {
        if elite.is_some() {
            material_handle.0 = material_elite.clone();
        } else if in_focus.is_some() {
            material_handle.0 = material_in_focus.clone();
        } else {
            material_handle.0 = material_default.clone();
        }
    }
}

/// Returns an observer that updates the entity's material to the one specified.
fn update_material_on<E>(new_material: Handle<ColorMaterial>) -> impl Fn(Trigger<E>, Query<&mut MeshMaterial2d<ColorMaterial>>) {
    // An observer closure that captures `new_material`. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // material. Instead, the event type is a generic, and the material is passed in.
    move |trigger, mut query| {
        if let Ok(mut material) = query.get_mut(trigger.target().entity()) {
            material.0 = new_material.clone();
        }
    }
}

pub fn set_new_elite_in_focus<'a>(
    // query: Query<(Entity, &PlankPhenotype), With<PlankPhenotype>>){
    mut commands: Commands,
    får_fokus_query: Query<Entity, (With<PlankPhenotype>, Without<IndividInFocus>, Added<EliteTag>)>,
) {
    // query: Query<'a, 'a, (Entity, &'a PlankPhenotype, &'a Genome), With<PlankPhenotype>>) {
    // let population = get_population_sorted_from_best_to_worst(query.iter());
    for entity in får_fokus_query.iter() {
        println!("setter beste individ til å være i fokus");
        commands.entity(entity).insert(IndividInFocus);
    }
}
fn fjern_fokus_fra_gammel_elite(mut mistet_elite_status_query: RemovedComponents<EliteTag>, mut commands: Commands) {
    for entity in mistet_elite_status_query.read() {
        commands.entity(entity).remove::<IndividInFocus>();
    }
}

fn publiser_event_for_ny_individ_i_fokus(query: Query<(Entity), Added<IndividInFocus>>, mut event_writer: EventWriter<IndividInFocusСhangedEvent>) {
    for entity in query.iter() {
        event_writer.write(IndividInFocusСhangedEvent { entity: entity });
    }
}

fn color_focus_green(mut commands: Commands, mut elite_query: Query<Entity, With<IndividInFocus>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    if let Ok(elite_entity) = elite_query.get_single() {
        let elite_material_handle: Handle<ColorMaterial> = materials.add(Color::from(GREEN));
        commands.entity(elite_entity).insert(MeshMaterial2d(elite_material_handle));
    }
}

fn add_hover_observers_to_individuals(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, individ_query: Query<Entity, With<PlankPhenotype>>) {
    let hover_matl = materials.add(Color::from(CYAN_300));
    for individ_entity in individ_query.iter() {
        commands
            .get_entity(individ_entity)
            .unwrap()
            .observe(update_material_on::<Pointer<Over>>(hover_matl.clone()))
            // .observe(pointer_out_of_individ)
            .observe(color_change_on_pointer_out_of_individ)
            .observe(place_in_focus);
    }
}
