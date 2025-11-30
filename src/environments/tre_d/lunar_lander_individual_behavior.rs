use crate::environments::tre_d::moving_plank_with_user_input_3d_plugin::MovingPlankWithUserInput3dPlugin;
use crate::environments::tre_d::spawn_lunar_lander_individ_plugin::SpawnLunarLanderPlugin;
use crate::evolusjon::evolusjon_steg_plugin::{IndividerSkalFåFitnessEvaluertMessage, IndividerSkalTenkeOgHandleMessage};
use crate::evolusjon::phenotype_plugin::PlankPhenotype;
use crate::monitoring::simulation_teller::SimulationGenerationTimer;
use avian3d::math::Vector;
use avian3d::prelude::*;
use bevy::prelude::*;
use lazy_static::lazy_static;
use std::ops::Mul;

pub struct LunarLanderIndividBehaviors;

lazy_static! {

    static ref LANDING_SITE: Vec<f32> = vec![0.0, 0.0, 0.0];
// static ref START_POSITION: Vec<f32> = vec![0.0, 0.0, 0.5];
    pub static ref START_POSITION: Vec3 = Vec3::new(0.5, 1.5, 0.5);
}

// impl EnvironmentSpesificIndividStuff for LunarLanderIndividBehaviors {
impl Plugin for LunarLanderIndividBehaviors {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(SpawnLunarLanderPlugin)
            .add_plugins(MovingPlankWithUserInput3dPlugin)
            .add_systems(
            Update,
            (agent_fitness_evaluation, agent_observation_and_action, check_if_done, reset_to_star_pos_on_event),
        );
    }
}

fn agent_fitness_evaluation(
    mut message_reader: MessageReader<IndividerSkalFåFitnessEvaluertMessage>,
    mut query: Query<(&Transform, &mut PlankPhenotype, Entity), (With<PlankPhenotype>)>,
    time: Res<Time>,
) {
    if !message_reader.is_empty() {
        for (
            transform,
            mut plank,
            // mut velocity,
            entity,
        ) in query.iter_mut()
        {
            // distance score to landingsite =  (x-x2)^2 + (y-y2)^2
            let distance = (LANDING_SITE[0] - transform.translation.x).powi(2)
                + (LANDING_SITE[1] - transform.translation.y).powi(2)
                + (LANDING_SITE[2] - transform.translation.z).powi(2);
            // println!("Entity {} : Landingsite {:?}, and xy {} has x distance {}, and y distance {}", entity.index(), LANDING_SITE, transform.translation.xy(),
            //          (LANDING_SITE.x - transform.translation.x).powi(2), (LANDING_SITE.y - transform.translation.y).powi(2));
            // smaller sitance is good
            plank.score = 1000.0 / distance;
        }
    }
    message_reader.read();
}

fn agent_observation_and_action(
    mut message_reader: MessageReader<IndividerSkalTenkeOgHandleMessage>,
    mut query: Query<(&Transform, &mut PlankPhenotype, Forces, Entity), (With<PlankPhenotype>)>,
    time: Res<Time>,
) {
    if !message_reader.is_empty() {
        for (transform, mut plank, mut forces, entity) in query.iter_mut() {
            // plank.obseravations = vec![transform.translation.x.clone(), transform.translation.y.clone()]
            plank.obseravations = transform.translation.xyz().to_array().clone().to_vec();
            let input_values = plank.obseravations.clone();
            // dbg!(&input_values);
            let action = plank.phenotype_layers.decide_on_action2(input_values);
            let x = 0.01 * action[0];
            let y = 0.01 * action[1];
            let z = 0.01 * action[2];
            let force_change = vec3(x, y, z).mul(1.0);
            // forces.non_waking().apply_force(vec2(x, y).mul(100.0));
            // dbg!(&force_change);
            forces.apply_force(force_change);
            // forces.non_waking().apply_local_linear_acceleration(Vec2::new(0.0, y));
        }
    }
    message_reader.read();
}

fn check_if_done(
    // query: Query<(&mut Transform, &mut PlankPhenotype), (With<PlankPhenotype>)>,
    mut message_reader: MessageReader<crate::evolusjon::evolusjon_steg_plugin::CheckIfDoneRequest>,
    mut message_writer: MessageWriter<crate::evolusjon::evolusjon_steg_plugin::GenerationIsDone>,
    simulation_timer: Res<SimulationGenerationTimer>,
) {
    if !message_reader.is_empty() {
        if simulation_timer.main_timer.just_finished() {
            // println!("done");
            message_writer.write(crate::evolusjon::evolusjon_steg_plugin::GenerationIsDone);
        }
    }
    message_reader.read();
}

fn reset_to_star_pos_on_event(
    mut query: Query<(
        &mut Transform,
        &mut PlankPhenotype,
        &mut LinearVelocity,
        &mut AngularVelocity,
        // Forces,
    )>,
    mut reset_events: MessageReader<crate::evolusjon::evolusjon_steg_plugin::ResetToStartPositionsEvent>,
) {
    if !reset_events.is_empty() {
        for (
            mut transform,
            mut plank,
            mut linvel,
            mut angular_velocity, // , forces
        ) in query.iter_mut()
        {
            transform.translation.x = START_POSITION.x;
                transform.translation.y = START_POSITION.y;
                transform.translation.z = START_POSITION.z;
            
            transform.rotation = Quat::default();

            plank.score = transform.translation.x.clone();
            plank.obseravations = transform.translation.xyz().to_array().clone().to_vec();
            // velocity.angvel = 0.0;
            linvel.x = 0.0;
            linvel.y = 0.0;
            linvel.z = 0.0;

            // angular_velocity.0 = 0.0;
            angular_velocity.0 = Vector::ZERO;

            // if let Some(mut force) = forces {
            //     force.apply_local_force(Vector::ZERO); // kanskje ikke lenger nødvendig? dette burde jo teknisk sett ikke gjøre noe, Jeg tror forces ikke persiteres lenger etter avian sin 0.4 milestone
            // }
        }
    }
    reset_events.read();
}
