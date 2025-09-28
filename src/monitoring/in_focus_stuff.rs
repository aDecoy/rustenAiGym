use crate::genome::genome_stuff::Genome;
use crate::monitoring::draw_network::place_in_focus;
use crate::populasjon_handlinger::population_sammenligninger::{EliteTag, get_best_elite};
use crate::{PhentypeAndGenome, PlankPhenotype, rotate_on_drag, update_material_on};
use bevy::app::{App, Plugin, Startup};
use bevy::asset::{Assets, Handle};
use bevy::color::palettes::basic::{GREEN, PURPLE, RED};
use bevy::color::palettes::tailwind::CYAN_300;
use bevy::color::{Alpha, Color};
use bevy::ecs::query::QueryIter;
use bevy::prelude::{
    ColorMaterial, Commands, Component, Entity, Event, MeshMaterial2d, Out, Over, Pointer, Query,
    ResMut, Trigger, Update, With,
};
use std::cmp::Ordering;
// mod populasjon_handlinger;

#[derive(Debug, Component)]
pub struct IndividInFocus;

#[derive(Event, Debug)]
pub struct IndividInFocusСhangedEvent {
    pub(crate) entity: Entity,
}

struct InFocusPlugin();

impl Plugin for InFocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                set_best_individ_in_focus,
                // color_change_on_pointer_out_of_individ
            ),
        )
        .add_systems(Update, (set_best_individ_in_focus,));
    }
}
fn color_change_on_pointer_out_of_individ(
    entity_som_forlates: Trigger<Pointer<Out>>,
    mut query: Query<
        (
            Entity,
            &mut MeshMaterial2d<ColorMaterial>,
            Option<&EliteTag>,
            Option<&IndividInFocus>,
        ),
        // With<(Individ, PlankPhenotype)>,
        With<(PlankPhenotype)>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let material_default: Handle<ColorMaterial> =
        materials.add(Color::from(PURPLE).with_alpha(0.5));
    let material_elite = materials.add(Color::from(RED));
    let material_in_focus = materials.add(Color::from(GREEN));

    if let Ok((entity, mut material_handle, elite, in_focus)) =
        query.get_mut(entity_som_forlates.target)
    {
        if elite.is_some() {
            material_handle.0 = material_elite.clone();
        } else if in_focus.is_some() {
            material_handle.0 = material_in_focus.clone();
        } else {
            material_handle.0 = material_default.clone();
        }
    }
}

pub fn set_best_individ_in_focus<'a>(
    // query: Query<(Entity, &PlankPhenotype), With<PlankPhenotype>>){
    mut commands: Commands,
    query: Query<(Entity, &PlankPhenotype, &Genome), With<PlankPhenotype>>,
) {
    // query: Query<'a, 'a, (Entity, &'a PlankPhenotype, &'a Genome), With<PlankPhenotype>>) {
    // let population = get_population_sorted_from_best_to_worst(query.iter());
    let elite = get_best_elite(query.iter());
    commands
        .get_entity(elite.entity)
        .unwrap()
        .insert(IndividInFocus);
}

fn color_focus_green(
    mut commands: Commands,
    mut elite_query: Query<Entity, With<IndividInFocus>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if let Ok(elite_entity) = elite_query.get_single() {
        let elite_material_handle: Handle<ColorMaterial> = materials.add(Color::from(GREEN));
        commands
            .entity(elite_entity)
            .insert(MeshMaterial2d(elite_material_handle));
    }
}

fn add_hover_observers_to_individuals(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    individ_query: Query<Entity, With<PlankPhenotype>>,
) {
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
