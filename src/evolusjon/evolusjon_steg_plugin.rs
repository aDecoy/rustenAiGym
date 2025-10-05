use crate::environments::lunar_lander_environment::LANDING_SITE;
use crate::environments::moving_plank::{PLANK_HIGHT, PLANK_LENGTH, create_plank_env_falling, create_plank_env_moving_right, create_plank_ext_force_env_falling};
use crate::evolusjon::hjerne_fenotype::nullstill_nettverk_verdier_til_0;
use crate::evolusjon::phenotype_plugin::{
    IndividFitnessLabelTextTag, PhentypeAndGenome, PlankPhenotype, add_observers_to_individuals, update_phenotype_network_for_changed_genomes,
};
use crate::genome::genom_muteringer::{MutasjonerErAktive, mutate_genomes};
use crate::genome::genome_stuff::{Genome, InnovationNumberGlobalCounter};
use crate::monitoring::camera_stuff::{AllIndividerCameraTag, AllIndividerWindowTag};
use crate::monitoring::simulation_teller::SimulationGenerationTimer;
use crate::{
    ACTIVE_ENVIROMENT, ANT_INDIVIDER_SOM_OVERLEVER_HVER_GENERASJON, ANT_PARENTS_HVER_GENERASJON, EnvValg, Kjøretilstand, START_POPULATION_SIZE, spawn_start_population,
};
use avian2d::math::{AdjustPrecision, Vector};
use avian2d::prelude::{AngularVelocity, ExternalForce, LinearVelocity};
use bevy::color::palettes::basic::PURPLE;
use bevy::color::palettes::tailwind::CYAN_300;
use bevy::prelude::KeyCode::{KeyR, KeyT};
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use lazy_static::lazy_static;
use rand::prelude::IndexedRandom;
use rand::thread_rng;
use std::cmp::{Ordering, max, min};
use std::collections::HashMap;

pub struct EvolusjonStegPlugin;

impl Plugin for EvolusjonStegPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResetToStartPositionsEvent>()
            .add_systems(
                Startup,
                (
                    (spawn_start_population, add_observers_to_individuals.after(spawn_start_population), reset_to_star_pos).chain(),
                ))
            .add_systems(
            Update,
            (
                reset_event_ved_input,
                reset_to_star_pos_on_event,
                check_if_done,
                (
                    kill_worst_individuals,
                    reset_to_star_pos,
                    mutate_genomes.run_if(in_state(MutasjonerErAktive::Ja)),
                    // updatePhenotypeNetwork for entities with mutated genomes .run_if(in_state(MutasjonerErAktive::Ja)),
                    update_phenotype_network_for_changed_genomes.run_if(in_state(MutasjonerErAktive::Ja)),
                    nullstill_nettverk_verdier_til_0,
                    create_new_children,
                    add_observers_to_individuals.after(create_new_children),
                )
                    .chain()
                    .run_if(in_state(Kjøretilstand::EvolutionOverhead)),
            ),
        );
    }
}

fn kill_worst_individuals(mut commands: Commands, query: Query<(Entity, &PlankPhenotype), With<PlankPhenotype>>) {
    let mut population = Vec::new();

    //sort_individuals
    for (entity, plank) in query.iter() {
        population.push((entity, plank))
    }
    // println!("population before sort: {:?}", population);
    // sort asc
    //     population.sort_by(| a, b| if a.1.score > b.1.score { Ordering::Greater } else if a.1.score < b.1.score { Ordering::Less } else { Ordering::Equal });
    population.sort_by(|(_, a), (_, b)| {
        if a.score > b.score {
            Ordering::Greater
        } else if a.score < b.score {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    });
    // println!("population after sort:  {:?}", population);
    // let number_of_individuals_to_kill = min(4, population.len() - 1);

    // println!("pop size {}, want to kill pop size - 3 = {}. Max killing 0", population.len(), population.len()  as i32- number_of_individuals_to_leave_alive);
    // println!("pop size {}, want to kill pop size - 3 = {}. Max killing 0, ressulting in {}", population.len(), population.len() - number_of_individuals_to_leave_alive, max(0, population.len() - number_of_individuals_to_leave_alive));

    let number_of_individuals_to_kill: usize = max(0, population.len() as i32 - ANT_INDIVIDER_SOM_OVERLEVER_HVER_GENERASJON) as usize;
    println!("killing of {} entities", number_of_individuals_to_kill);
    for (entity, _) in &population[0..number_of_individuals_to_kill] {
        // println!("despawning entity {} ", entity.index());
        commands.entity(*entity).despawn_recursive();
    }
}

fn extinction_on_t(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity), With<PlankPhenotype>>,
    key_input: Res<ButtonInput<KeyCode>>,
    innovation_number_global_counter: ResMut<InnovationNumberGlobalCounter>,
) {
    if key_input.just_pressed(KeyT) {
        for (entity) in query.iter() {
            commands.entity(entity).despawn();
        }
        spawn_start_population(commands, meshes, materials, innovation_number_global_counter)
    }
}

fn create_new_children(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &PlankPhenotype, &Genome), With<PlankPhenotype>>,
    camera: Query<Entity, With<AllIndividerCameraTag>>,
) {
    let mut population = Vec::new();
    //sort_individuals
    for (entity, plank, genome) in query.iter() {
        population.push(PhentypeAndGenome {
            phenotype: plank,
            genome: genome,
            entity_index: entity.index(),
            entity: entity,
            entity_bevy_generation: entity.generation(),
        })
    }
    // println!("population size when making new individuals: {}", population.len() );
    // println!("parents before sort: {:?}", population);
    // todo legge inn generation_rank som en komponent, og sortere i ett system ??
    // todo alt. ha sorterte Plank også ta inn genom eller entity/ eller (phenotype,genom) tuples eller ny struct som bare brukes til dette. ..
    // sadfasdf
    // sort desc
    population.sort_by(|a, b| {
        if a.phenotype.score > b.phenotype.score {
            Ordering::Less
        } else if a.phenotype.score < b.phenotype.score {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
    // println!("parents after sort:  {:?}", population);

    let mut parents = Vec::new();

    // Select potential Parent
    for n in 0..min(ANT_PARENTS_HVER_GENERASJON, population.len()) {
        let parent = population[n].clone();
        println!(
            "the lucky winner was parent with entity index {}, with entity generation {} that had score: {} ",
            parent.entity_index, parent.entity_bevy_generation, parent.phenotype.score
        );
        parents.push(parent);
    }

    // For now, simple fill up population to pop  size . Note this does ruin some evolution patters if competition between indiviuals are a thing in the environment
    let pop_to_fill = START_POPULATION_SIZE - population.len() as i32;
    let mut thread_random = thread_rng();
    for _ in 0..pop_to_fill {
        // let uniform_dist = Uniform::new(-1.0, 1.0);
        // https://stackoverflow.com/questions/34215280/how-can-i-randomly-select-one-element-from-a-vector-or-array
        // let parent: &PlankPhenotype = parents.sample(&mut thread_random);

        // let mut new_genome : Genome = commands.get_entity(parents.choose(&mut thread_random).expect("No potential parents :O !?").genotype).expect("burde eksistere").clone();
        let parent: &PhentypeAndGenome = parents.choose(&mut thread_random).expect("No potential parents :O !?");
        // println!("the lucky winner was parent with entity index {}, that had score {} ", parent.entity_index, parent.phenotype.score);
        let mut new_genome: Genome = parent.genome.clone();

        // NB: mutation is done in a seperate bevy system
        new_genome.allowed_to_change = true;

        let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(PLANK_LENGTH, PLANK_HIGHT));
        let material_handle: Handle<ColorMaterial> = materials.add(Color::from(PURPLE).with_alpha(0.5));
        let hover_matl = materials.add(Color::from(CYAN_300));

        // let text_style = TextStyle {
        //     font_size: 20.0,
        //     color: Color::WHITE,
        //     ..default()
        // };
        match ACTIVE_ENVIROMENT {
            EnvValg::Fall | EnvValg::FallVelocityHøyre => commands.spawn(create_plank_env_falling(
                material_handle.clone(),
                rectangle_mesh_handle.into(),
                Vec3 {
                    x: 0.0,
                    y: -150.0 + 3.3 * 50.0,
                    z: 1.0,
                },
                new_genome,
            )),
            EnvValg::Høyre => commands.spawn(create_plank_env_moving_right(
                material_handle.clone(),
                rectangle_mesh_handle.into(),
                Vec3 {
                    x: 0.0,
                    y: -150.0 + 3.3 * 50.0,
                    z: 1.0,
                },
                new_genome,
            )),
            EnvValg::FallExternalForcesHøyre | EnvValg::Homing | EnvValg::HomingGroud | EnvValg::HomingGroudY => commands.spawn(create_plank_ext_force_env_falling(
                material_handle.clone(),
                rectangle_mesh_handle.into(),
                Vec3 {
                    x: 0.0,
                    y: -150.0 + 3.3 * 50.0,
                    z: 0.0,
                },
                new_genome,
            )),
        }
        .with_children(|builder| {
            builder.spawn((
                Text2d::new("Fitness label"),
                TextLayout::new_with_justify(JustifyText::Center),
                Transform::from_xyz(0.0, 0.0, 2.0),
                IndividFitnessLabelTextTag,
                RenderLayers::layer(1),
            ));
        });
    }
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

#[derive(Event, Debug, Default)]
struct ResetToStartPositionsEvent;

fn reset_to_star_pos_on_event(
    mut reset_events: EventReader<ResetToStartPositionsEvent>,
    // query: Query<(&mut Transform, &mut crate::PlankPhenotype, &mut Velocity), ( With<crate::PlankPhenotype>)>,
    // query: Query<(&mut Transform, &mut crate::PlankPhenotype, &mut LinearVelocity, Option<&mut ExternalForce>), ( With<crate::PlankPhenotype>)>,
    query: Query<(
        &mut Transform,
        &mut PlankPhenotype,
        &mut LinearVelocity,
        &mut AngularVelocity,
        Option<&mut ExternalForce>,
    )>,
) {
    if reset_events.read().next().is_some() {
        reset_to_star_pos(query);
    }
}

fn reset_event_ved_input(user_input: Res<ButtonInput<KeyCode>>, mut reset_events: EventWriter<ResetToStartPositionsEvent>) {
    if user_input.pressed(KeyR) {
        reset_events.send_default();
    }
}

// fn agent_action(query: Query<Transform, With<Individual>>) {
fn agent_action_and_fitness_evaluation(
    mut query: Query<(&mut Transform, &mut PlankPhenotype, &mut LinearVelocity, Option<&mut ExternalForce>, Entity), (With<PlankPhenotype>)>,
    time: Res<Time>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_secs_f64().adjust_precision();

    for (mut transform, mut plank, mut velocity, option_force, entity) in query.iter_mut() {
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
        let mut a = option_force.expect("did not have forces on individ!!? :( ");
        match ACTIVE_ENVIROMENT {
            EnvValg::Høyre | EnvValg::Fall => transform.translation.x += action[0] * 2.0,
            EnvValg::FallVelocityHøyre => velocity.0.x += action[0],
            // EnvValg::FallGlideBomb => velocity.0 += action,
            // EnvValg::FallExternalForcesHøyre => option_force.expect("did not have forces on individ!!? :( ").x = action,
            EnvValg::FallExternalForcesHøyre | EnvValg::Homing | EnvValg::HomingGroud => {
                // a.x = 100.0 * action[0] * delta_time;
                // a.y = 100.0 * action[1] * delta_time;
                a.x = 10.0 * action[0];
                a.y = 10.0 * action[1];

                // a.y = action;
                // NB: expternal force can be persitencte, or not. If not, then applyForce function must be called to do anything
                // println!("applying force {:#?}, and now velocity is {:?}", a.force(), velocity);
                // a.apply_force(Vector::ZERO);
            }
            EnvValg::HomingGroudY => {
                a.y = 10.0 * action[0];
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
        Option<&mut ExternalForce>,
    )>,
) {
    for (mut transform, mut plank, mut linvel, mut angular_velocity, option_force) in query.iter_mut() {
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

        if let Some(mut force) = option_force {
            force.apply_force(Vector::ZERO);
        }
    }
}
