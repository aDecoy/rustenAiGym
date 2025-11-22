use std::collections::HashMap;
use avian2d::prelude::{AngularVelocity, LinearVelocity};
use bevy::prelude::*;
use lazy_static::lazy_static;
use crate::{EnvValg, ACTIVE_ENVIROMENT};
use crate::environments::gammelt_2d::spawn_2d_individ_plugin::Spawn2dIndividPlugin;
use crate::evolusjon::phenotype_plugin::PlankPhenotype;

pub struct EvolutionStepsImplementationPlugin;

impl Plugin for EvolutionStepsImplementationPlugin{
    fn build(&self, app: &mut App) {
        app
            .add_plugins(Spawn2dIndividPlugin) 
            .add_systems(Update, (
                reset_to_star_pos_on_event // Kan ikke gjøre run in state hvis state blir byttet tilbake i samme Update runde som eventet sendes ut  .run_if(in_state(Kjøretilstand::EvolutionOverhead)),
            ));
    }
}


lazy_static! {
     static ref START_POSITION_PER_ENVIRONMENT:HashMap<EnvValg ,Vec2 > = {
 HashMap::from([
    ( EnvValg::Høyre, Vec2 { x: 0.0, y: 0.0 }), // y is determined by index in population
    ( EnvValg::Homing, Vec2 { x: 0.0, y: -0.0 }),
    ( EnvValg::HomingGroud, Vec2 { x: 100.0, y: 30.0 }),
    ( EnvValg::HomingGroudY, Vec2 { x: 100.0, y: 30.0 }),
    ])
    };
    pub static ref START_POSITION: Vec2 = START_POSITION_PER_ENVIRONMENT[&ACTIVE_ENVIROMENT];
}

// NB : Dette resetter ikke node verdiene i nettverket til phenotypen
fn reset_to_star_pos(
    mut query: Query<(
        &mut Transform,
        &mut PlankPhenotype,
        &mut LinearVelocity,
        &mut AngularVelocity,
        // Forces,
    )>,
) {
    for (
        mut transform,
        mut plank,
        mut linvel,
        mut angular_velocity, // , forces
    ) in query.iter_mut()
    {
        transform.translation.x = START_POSITION.x;
        if ACTIVE_ENVIROMENT != EnvValg::Høyre {
            transform.translation.y = START_POSITION.y;
        }
        transform.rotation = Quat::default();

        plank.score = transform.translation.x.clone();
        plank.obseravations = vec![transform.translation.x.clone(), transform.translation.y.clone()];
        // velocity.angvel = 0.0;
        linvel.x = 0.0;
        linvel.y = 0.0;

        angular_velocity.0 = 0.0;

        // if let Some(mut force) = forces {
        //     force.apply_local_force(Vector::ZERO); // kanskje ikke lenger nødvendig? dette burde jo teknisk sett ikke gjøre noe, Jeg tror forces ikke persiteres lenger etter avian sin 0.4 milestone
        // }
    }
}
fn reset_to_star_pos_on_event(
    mut reset_events: MessageReader<crate::evolusjon::evolusjon_steg_plugin::ResetToStartPositionsEvent>,
    // query: Query<(&mut Transform, &mut crate::PlankPhenotype, &mut Velocity), ( With<crate::PlankPhenotype>)>,
    // query: Query<(&mut Transform, &mut crate::PlankPhenotype, &mut LinearVelocity, Option<&mut ExternalForce>), ( With<crate::PlankPhenotype>)>,
    query: Query<(
        &mut Transform,
        &mut PlankPhenotype,
        &mut LinearVelocity,
        &mut AngularVelocity,
        // Forces,
    )>,
) {
    if reset_events.read().next().is_some() {
        reset_to_star_pos(query);
    }
}