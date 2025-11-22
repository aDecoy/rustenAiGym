use crate::environments::tre_d::individ_watching_3d_camera::IndividWatching3dCameraPlugin;
use crate::environments::tre_d::spawn_lunar_lander_individ_plugin::SpawnLunarLanderPlugin;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use crate::monitoring::camera_stuff::RENDER_LAYER_NETTVERK;

pub(crate) struct LunarLanderEnvironment3d;

impl Plugin for LunarLanderEnvironment3d {
    fn build(&self, app: &mut App) {
        app.add_plugins(IndividWatching3dCameraPlugin)
            .add_plugins(SpawnLunarLanderPlugin)
            .add_systems(Startup, (spawn_ground));
    }
}

fn spawn_ground(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.2, 0.1))),
        RenderLayers::layer(1),
    ));
    // // sphere
    // commands.spawn((
    //     Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(4).unwrap())),
    //     MeshMaterial3d(materials.add(Color::srgb(0.1, 0.4, 0.8))),
    //     Transform::from_xyz(1.5, 1.5, 1.5),
    //     RenderLayers::layer(1),
    // ));
    // light
    commands.spawn((
        PointLight {
            intensity: 1_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        RenderLayers::layer(RENDER_LAYER_NETTVERK),
    ));
}
