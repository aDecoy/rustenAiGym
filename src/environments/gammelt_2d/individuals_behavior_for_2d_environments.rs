use crate::environments::felles_miljø_traits::EnvironmentSpesificIndividStuff;
use crate::environments::gammelt_2d::lunar_lander_environment2d::LANDING_SITE;
use crate::environments::gammelt_2d::moving_plank_2d::{ PLANK_HIGHT, PLANK_LENGTH,};
use crate::evolusjon::evolusjon_steg_plugin::{Kjøretilstand, SpawnNewIndividualMessage};
use crate::evolusjon::phenotype_plugin::{IndividFitnessLabelTextTag, PlankPhenotype};
use crate::genome::genome_stuff::{new_random_genome, InnovationNumberGlobalCounter};
use crate::monitoring::camera_stuff::AllIndividerWindowTag;
use crate::monitoring::simulation_teller::SimulationGenerationTimer;
use crate::{EnvValg, ACTIVE_ENVIROMENT};
use avian2d::prelude::*;
use bevy::asset::{Assets, Handle};
use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::basic::PURPLE;
use bevy::color::palettes::tailwind::CYAN_300;
use bevy::color::Color;
use bevy::math::{vec2, Vec2, Vec3};
use bevy::mesh::Mesh;
use bevy::prelude::*;
use bevy::prelude::{ColorMaterial, Commands, Entity, Justify, NextState, Query, Rectangle, Res, ResMut, Text2d, TextLayout, Time, Transform, Window, With};
use std::ops::Mul;
use crate::environments::gammelt_2d::spawn_2d_individ_plugin::{create_plank_env_falling, create_plank_env_moving_right, create_plank_ext_force_env_falling};

pub struct ToDimensjonelleMijøSpesifikkeIndividOppførsler;

// impl EnvironmentSpesificIndividStuff for ToDimensjonelleMijøSpesifikkeIndividOppførsler {
impl ToDimensjonelleMijøSpesifikkeIndividOppførsler {
    pub(crate) fn spawn_a_random_new_individual(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        innovation_number_global_counter: &mut ResMut<InnovationNumberGlobalCounter>,
        n: i32,
         spawn_new_individual_message_writer: &mut MessageWriter<SpawnNewIndividualMessage>,
    ) {
        let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(PLANK_LENGTH, PLANK_HIGHT));
        let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE));

        let hover_matl = materials.add(Color::from(CYAN_300));
        // println!("Har jeg klart å lage en genome fra entity = : {}", genome2.allowed_to_change);
        // let text_style = TextStyle {
        //     font_size: 20.0,
        //     color: Color::WHITE,
        //     ..default()
        // };
        let genome = match ACTIVE_ENVIROMENT {
            EnvValg::HomingGroudY => new_random_genome(1, 1, innovation_number_global_counter),
            _ => new_random_genome(2, 2, innovation_number_global_counter),
        };

        spawn_new_individual_message_writer.write(SpawnNewIndividualMessage{
            new_genome: genome,
            n : n
        });
    }

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
    fn spawn_a_random_new_individual2(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        mut innovation_number_global_counter: ResMut<InnovationNumberGlobalCounter>,
        mut spawn_new_individual_message_writer: MessageWriter<SpawnNewIndividualMessage>,
    ) {
        let n: i32 = 1;
        ToDimensjonelleMijøSpesifikkeIndividOppførsler::spawn_a_random_new_individual(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut innovation_number_global_counter,
            n,
            &mut spawn_new_individual_message_writer,

        )
    }

    fn check_if_done(
        mut query: Query<(&mut Transform, &mut PlankPhenotype), (With<PlankPhenotype>)>,
        mut next_state: ResMut<NextState<Kjøretilstand>>,
        simulation_timer: Res<SimulationGenerationTimer>,
        window: Query<&Window, With<AllIndividerWindowTag>>,
    ) {
        let max_width = window.single().unwrap().width() * 0.5;

        match ACTIVE_ENVIROMENT {
            EnvValg::Høyre | EnvValg::Fall | EnvValg::FallVelocityHøyre | EnvValg::FallExternalForcesHøyre => {
                // done if one is all the way to the right of the screen
                for (individual, _) in query.iter_mut() {
                    if individual.translation.x > max_width {
                    // println!("done");
                    ; // er det skalert etter reapier logikk eller pixler\?
                    next_state.set(Kjøretilstand::EvolutionOverhead)
                }
                }
            }
            EnvValg::Homing | EnvValg::HomingGroud | EnvValg::HomingGroudY => {
                if simulation_timer.main_timer.just_finished() {
                // println!("done");
                ; // er det skalert etter reapier logikk eller pixler\?
                next_state.set(Kjøretilstand::EvolutionOverhead);
            }
            }
        }
    }
}
