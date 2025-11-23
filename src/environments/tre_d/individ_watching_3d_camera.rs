use crate::monitoring::camera_stuff::{AllIndividerCameraTag, AllIndividerWindowTag, RENDER_LAYER_ALLE_INDIVIDER};
use bevy::app::{App, Plugin, PreStartup};
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{Camera, Camera3d};
use bevy::math::Vec3;
use bevy::prelude::{Commands, Entity, Query, Transform, Window, With, default};

pub struct IndividWatching3dCameraPlugin;

impl Plugin for IndividWatching3dCameraPlugin {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app.add_systems(PreStartup, (setup_individ_watching_camera));
    }
}

// todo Kanskje legge til et camera som følger elite target

fn setup_individ_watching_camera(mut commands: Commands, query: Query<Entity, With<Window>>) {
    let main_camera = commands
        .spawn((
            Camera {
                // Renders cameras with different priorities to prevent ambiguities
                order: 1 as isize,
                ..default()
            },
            // AllIndividerCamera{ camera_mode: CameraMode::HALV },
            AllIndividerCameraTag,
            RenderLayers::from_layers(&[RENDER_LAYER_ALLE_INDIVIDER]),
        ))
        .id();
    commands.entity(query.single().unwrap()).insert((AllIndividerWindowTag));
    commands
        .get_entity(main_camera)
        .unwrap()
        .insert((Camera3d::default(), Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y)));
}
