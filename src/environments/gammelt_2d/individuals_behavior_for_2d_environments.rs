use crate::environments::felles_miljø_traits::EnvironmentSpesificIndividStuff;
use crate::environments::gammelt_2d::lunar_lander_environment2d::LANDING_SITE;
use crate::environments::gammelt_2d::moving_plank_with_user_input_2d_plugin::{PLANK_HIGHT, PLANK_LENGTH};
use crate::evolusjon::evolusjon_steg_plugin::{Kjøretilstand, SpawnNewIndividualMessage};
use crate::evolusjon::phenotype_plugin::PlankPhenotype;
use crate::genome::genome_stuff::{Genome, InnovationNumberGlobalCounter, new_random_genome};
use crate::monitoring::camera_stuff::AllIndividerWindowTag;
use crate::monitoring::simulation_teller::SimulationGenerationTimer;
use crate::{ACTIVE_ENVIROMENT, EnvValg};
use avian2d::prelude::*;
use bevy::asset::{Assets, Handle};
use bevy::color::Color;
use bevy::color::palettes::basic::PURPLE;
use bevy::color::palettes::tailwind::CYAN_300;
use bevy::math::{Vec2, vec2};
use bevy::mesh::Mesh;
use bevy::prelude::*;
use bevy::prelude::{ColorMaterial, Commands, Entity, NextState, Query, Rectangle, Res, ResMut, Time, Transform, Window, With};
use std::ops::Mul;

pub struct ToDimensjonelleMijøSpesifikkeIndividOppførsler;

// impl EnvironmentSpesificIndividStuff for ToDimensjonelleMijøSpesifikkeIndividOppførsler {
impl ToDimensjonelleMijøSpesifikkeIndividOppførsler {
    // fn agent_action(query: Query<Transform, With<Individual>>) {
    fn agent_action_and_fitness_evaluation(
        // mut query: Query<(&mut Transform, &mut PlankPhenotype, &mut LinearVelocity, Option<Forces>, Entity), (With<PlankPhenotype>)>,
        mut query: Query<
            (
                &mut Transform,
                &mut PlankPhenotype,
                // &mut LinearVelocity,
                Forces,
                Entity,
            ),
            (With<PlankPhenotype>),
        >,
        time: Res<Time>,
    ) {
        // Precision is adjusted so that the example works with
        // both the `f32` and `f64` features. Otherwise you don't need this.
        // let delta_time = time.delta_secs_f64().adjust_precision();

        for (
            mut transform,
            mut plank,
            // mut velocity,
            mut forces,
            entity,
        ) in query.iter_mut()
        {
            plank.obseravations = match ACTIVE_ENVIROMENT {
                EnvValg::HomingGroudY => vec![transform.translation.y.clone()],
                _ => vec![transform.translation.x.clone(), transform.translation.y.clone()],
            };
            // let input_values = vec![1.0, 2.0]; // 2 inputs
            // let input_values = vec![individual.translation.x.clone() * 0.002, individual.translation.y.clone()* 0.002]; // 2 inputs
            let input_values = plank.obseravations.clone();
            // dbg!(&input_values);
            let action = plank.phenotype_layers.decide_on_action2(input_values); // fungerer
            // dbg!(&action);
            // individual.translation.x += random::<f32>() * action * 5.0;
            // println!("action : {action}");
            // let mut forces : Forces = option_force.expect("did not have forces on individ!!? :( ");
            match ACTIVE_ENVIROMENT {
                EnvValg::Høyre | EnvValg::Fall => transform.translation.x += action[0] * 2.0,
                // EnvValg::FallVelocityHøyre => velocity.0.x += action[0],
                // EnvValg::FallVelocityHøyre => forces.apply_linear_impulse(vec2(action[0], 0.0)),
                EnvValg::FallVelocityHøyre => forces.apply_linear_impulse(vec2(action[0], 0.0)),
                // EnvValg::FallGlideBomb => velocity.0 += action,
                // EnvValg::FallExternalForcesHøyre => option_force.expect("did not have forces on individ!!? :( ").x = action,
                EnvValg::FallExternalForcesHøyre | EnvValg::Homing | EnvValg::HomingGroud => {
                    // a.x = 100.0 * action[0] * delta_time;
                    // a.y = 100.0 * action[1] * delta_time;
                    let x = 10.0 * action[0];
                    let y = 10.0 * action[1];
                    // forces.non_waking().apply_force(vec2(x, y).mul(100.0));
                    forces.apply_force(vec2(x, y).mul(1.0));
                    // forces.non_waking().apply_linear_acceleration(Vec2::new(x,y));

                    // a.y = action;
                    // NB: expternal force can be persitencte, or not. If not, then applyForce function must be called to do anything
                    // println!("applying force {:#?}, and now velocity is {:?}", a.force(), velocity);
                    // a.apply_force(Vector::ZERO);
                }
                EnvValg::HomingGroudY => {
                    let y = 10.0 * action[0];
                    forces.non_waking().apply_local_linear_acceleration(Vec2::new(0.0, y));
                }
            }
            // println!("option force {:#?}", a.clone());
            // individual.translation.x += random::<f32>() * plank.phenotype * 5.0;
            match ACTIVE_ENVIROMENT {
                EnvValg::Høyre | EnvValg::Fall | EnvValg::FallVelocityHøyre | EnvValg::FallExternalForcesHøyre => {
                    plank.score = transform.translation.x.clone();
                }
                EnvValg::Homing | EnvValg::HomingGroud => {
                    // distance score to landingsite =  (x-x2)^2 + (y-y2)^2
                    let distance = (LANDING_SITE.x - transform.translation.x).powi(2) + (LANDING_SITE.y - transform.translation.y).powi(2);
                    // println!("Entity {} : Landingsite {:?}, and xy {} has x distance {}, and y distance {}", entity.index(), LANDING_SITE, transform.translation.xy(),
                    //          (LANDING_SITE.x - transform.translation.x).powi(2), (LANDING_SITE.y - transform.translation.y).powi(2));
                    // smaller sitance is good
                    plank.score = 1000.0 / distance;
                }
                EnvValg::HomingGroudY => {
                    // distance score to landingsite =  (x-x2)^2 + (y-y2)^2
                    let distance = (LANDING_SITE.y - transform.translation.y).powi(2);
                    // println!("Entity {} : Landingsite {:?}, and xy {} has x distance {}, and y distance {}", entity.index(), LANDING_SITE, transform.translation.xy(),
                    //          (LANDING_SITE.x - transform.translation.x).powi(2), (LANDING_SITE.y - transform.translation.y).powi(2));
                    // smaller sitance is good
                    plank.score = 1000.0 / distance;
                }
            }
            // println!("individual {} chose action {} with inputs {}", plank.genotype.id.clone(), action ,plank.obseravations.clone()  );
        }
    }

    // Turns out Rust dont have any good default parameter solutions. At least none that i like. Ok kanskje det er noen ok løsninger. https://www.thecodedmessage.com/posts/default-params/

    fn check_if_done(
        mut message_reader: MessageReader<crate::evolusjon::evolusjon_steg_plugin::CheckIfDoneRequest>,
        mut message_writer: MessageWriter<crate::evolusjon::evolusjon_steg_plugin::GenerationIsDone>,
        mut query: Query<(&mut Transform, &mut PlankPhenotype), (With<PlankPhenotype>)>,
        mut next_state: ResMut<NextState<Kjøretilstand>>,
        simulation_timer: Res<SimulationGenerationTimer>,
        window: Query<&Window, With<AllIndividerWindowTag>>,
    ) {
        if !message_reader.is_empty() {
            let max_width = window.single().unwrap().width() * 0.5;

            match ACTIVE_ENVIROMENT {
                EnvValg::Høyre | EnvValg::Fall | EnvValg::FallVelocityHøyre | EnvValg::FallExternalForcesHøyre => {
                    // done if one is all the way to the right of the screen
                    for (individual, _) in query.iter_mut() {
                        if individual.translation.x > max_width {
                            // println!("done");
                            message_writer.write(crate::evolusjon::evolusjon_steg_plugin::GenerationIsDone);
                        }
                    }
                }
                EnvValg::Homing | EnvValg::HomingGroud | EnvValg::HomingGroudY => {
                    if simulation_timer.main_timer.just_finished() {
                        // println!("done");
                        message_writer.write(crate::evolusjon::evolusjon_steg_plugin::GenerationIsDone);
                    }
                }
            }
        }
        message_reader.read();
    }
}
