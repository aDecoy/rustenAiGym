use crate::environments::tre_d::individ_watching_3d_camera::IndividWatching3dCameraPlugin;
use crate::environments::tre_d::lunar_lander_individual_behavior::LunarLanderIndividBehaviors;
use crate::monitoring::camera_stuff::{RENDER_LAYER_ALLE_INDIVIDER, RENDER_LAYER_NETTVERK};
use avian3d::prelude::*;
use avian3d::prelude::{Physics, PhysicsTime};
use avian3d::PhysicsPlugins;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

pub(crate) struct LunarLanderEnvironment3d;
pub const PIXELS_PER_METER: f32 = 100.0;
pub const PHYSICS_RELATIVE_SPEED: f32 = 1.0;

impl Plugin for LunarLanderEnvironment3d {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins((PhysicsPlugins::default().with_length_unit(PIXELS_PER_METER),))
            .add_plugins((PhysicsPlugins::default()))
            .add_plugins((PhysicsDebugPlugin::default()))
            .insert_resource(Time::<Physics>::default().with_relative_speed(PHYSICS_RELATIVE_SPEED))
            .add_plugins(IndividWatching3dCameraPlugin)
            .add_plugins(LunarLanderIndividBehaviors)
            .add_systems(Startup, (spawn_ground));
    }
}

fn spawn_ground(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.2, 0.1))),
        RenderLayers::layer(RENDER_LAYER_ALLE_INDIVIDER),
        RigidBody::Static,
        Collider::cuboid(10.0, 0.1002, 5.00), 
        Restitution::new(0.0),
        Friction::new(0.5),
        CollisionLayers::new(0b0010, LayerMask::ALL), // todo forvirrende retning. kanskje gi transfor med z spesifikt opp ? Tror Plane3d setter y opp ... rart
    ));
    // // sphere
    // commands.spawn((
    //     Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(4).unwrap())),
    //     MeshMaterial3d(materials.add(Color::srgb(0.1, 0.4, 0.8))),
    //     Transform::from_xyz(1.5, 1.5, 1.5),
    //     RenderLayers::layer(1),
    // ));
    // light
    // commands.spawn((
    //     PointLight {
    //         intensity: 1_000_000.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     Transform::from_xyz(4.0, 8.0, 4.0),
    //     RenderLayers::layer(RENDER_LAYER_NETTVERK),
    // ));
    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
            RenderLayers::layer(RENDER_LAYER_NETTVERK), // Er dette et problem for skygger?? Eller mangler 

    ));

}
