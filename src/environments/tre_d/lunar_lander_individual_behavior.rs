use std::ops::Mul;
use avian3d::math::Vector;
use crate::evolusjon::evolusjon_steg_plugin::{Kjøretilstand, SpawnNewIndividualMessage};
use crate::evolusjon::phenotype_plugin::PlankPhenotype;
use crate::genome::genome_stuff::{InnovationNumberGlobalCounter, new_random_genome};
use crate::monitoring::camera_stuff::AllIndividerWindowTag;
use crate::monitoring::simulation_teller::SimulationGenerationTimer;
use avian3d::prelude::{AngularVelocity, Forces, LinearVelocity, RigidBodyForces};
use bevy::asset::Assets;
use bevy::mesh::Mesh;
use bevy::prelude::*;
use lazy_static::lazy_static;
use crate::{EnvValg, ACTIVE_ENVIROMENT};

pub struct LunarLanderIndividBehaviors;

lazy_static! {
     
    static ref LANDING_SITE: Vec<f32> = vec![0.0, 0.0, 0.0];
// static ref START_POSITION: Vec<f32> = vec![0.0, 0.0, 0.5];
static ref START_POSITION: Vec3 = Vec3::new(0.0, 0.0, 0.5);
}

// impl EnvironmentSpesificIndividStuff for LunarLanderIndividBehaviors {
impl LunarLanderIndividBehaviors {
    pub(crate) fn spawn_a_random_new_individual(
        mut spawn_new_individual_message: MessageWriter<SpawnNewIndividualMessage>,
        innovation_number_global_counter: &mut ResMut<InnovationNumberGlobalCounter>,
        n: i32,
    ) {
        let genome = new_random_genome(2, 2, innovation_number_global_counter);
        spawn_new_individual_message.write(SpawnNewIndividualMessage { new_genome: genome, n: n });
    }
        
    pub fn agent_fitness_evaluation(mut query: Query<(&Transform, &mut PlankPhenotype, Entity), (With<PlankPhenotype>)>, time: Res<Time>) {
        for (
            transform,
            mut plank,
            // mut velocity,
            entity,
        ) in query.iter_mut()
        {
            // distance score to landingsite =  (x-x2)^2 + (y-y2)^2
            let distance = (LANDING_SITE[0] - transform.translation.x).powi(2) + (LANDING_SITE[1] - transform.translation.y).powi(2)+ (LANDING_SITE[2] - transform.translation.z).powi(2);
            // println!("Entity {} : Landingsite {:?}, and xy {} has x distance {}, and y distance {}", entity.index(), LANDING_SITE, transform.translation.xy(),
            //          (LANDING_SITE.x - transform.translation.x).powi(2), (LANDING_SITE.y - transform.translation.y).powi(2));
            // smaller sitance is good
            plank.score = 1000.0 / distance;
        }
    }
    
    pub fn agent_observation_and_action(mut query: Query<(&Transform, &mut PlankPhenotype, Forces, Entity), (With<PlankPhenotype>)>, time: Res<Time>) {
        for (
            transform,
            mut plank,
            mut forces,
            entity,
        ) in query.iter_mut()
        {
            // plank.obseravations = vec![transform.translation.x.clone(), transform.translation.y.clone()]
            plank.obseravations = transform.translation.xyz().to_array().clone().to_vec();
            let input_values = plank.obseravations.clone();
            // dbg!(&input_values);
            let action = plank.phenotype_layers.decide_on_action2(input_values);
            let x = 1.0 * action[0];
            let y = 1.0 * action[1];
            let z = 1.0 * action[2];
            // forces.non_waking().apply_force(vec2(x, y).mul(100.0));
            forces.apply_force(vec3(x, y, z).mul(1.0));
            // forces.non_waking().apply_local_linear_acceleration(Vec2::new(0.0, y));
        }
    }

    pub fn spawn_a_random_new_individual2(
        commands: Commands,
        meshes: ResMut<Assets<Mesh>>,
        materials: ResMut<Assets<ColorMaterial>>,
        innovation_number_global_counter: ResMut<InnovationNumberGlobalCounter>,
    ) {
        todo!()
    }

    pub fn check_if_done(
        // query: Query<(&mut Transform, &mut PlankPhenotype), (With<PlankPhenotype>)>,
        mut message_reader: MessageReader<crate::evolusjon::evolusjon_steg_plugin::CheckIfDoneRequest>,
        mut message_writer: MessageWriter<crate::evolusjon::evolusjon_steg_plugin::GenerationIsDone>,
        simulation_timer: Res<SimulationGenerationTimer>,
    ) {
        if !message_reader.is_empty(){
        if simulation_timer.main_timer.just_finished() {
            // println!("done");
            message_writer.write(crate::evolusjon::evolusjon_steg_plugin::GenerationIsDone);
        }
        }
        message_reader.read();
    }
    
    pub fn reset_to_star_pos_on_event(
        mut query: Query<(
            &mut Transform,
            &mut PlankPhenotype,
            &mut LinearVelocity,
            &mut AngularVelocity,
            // Forces,
        )>,
        mut reset_events: MessageReader<crate::evolusjon::evolusjon_steg_plugin::ResetToStartPositionsEvent>,

    ) {
        if !reset_events.is_empty(){
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
}
