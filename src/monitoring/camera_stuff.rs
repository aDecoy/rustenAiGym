use bevy::color::palettes::basic::RED;
use bevy::color::palettes::css::BLUE;
use bevy::color::palettes::tailwind::CYAN_950;
use bevy::input::ButtonInput;
use bevy::math::{CompassOctant, UVec2, Vec3};
use bevy::prelude::*;
use bevy::render::camera::{RenderTarget, Viewport};
use bevy::render::view::RenderLayers;
// use bevy::utils::dbg;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::window::{PrimaryWindow, WindowRef, WindowResized};
use bevy_inspector_egui::egui::debug_text::print;
use std::cmp::{max, min};

pub struct MinCameraPlugin;

impl Plugin for MinCameraPlugin {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app.insert_state(CameraDragningJustering::PÅ)
            .insert_resource(LeftClickAction::Resize)
            .insert_resource(ResizeDir(7))
            .add_systems(PreStartup, ((setup_camera, set_camera_viewports).chain()))
            .add_systems(Startup, spawn_camera_resize_button_for_neuron_camera)
            .add_systems(Startup, spawn_camera_move_in_world_button_for_neuron_camera)
            .add_systems(
                Startup,
                spawn_camera_move_on_screen_button_for_neuron_camera,
            )
            // .add_systems(Startup, spawn_camera_margines)
            .add_systems(
                Update,
                (
                    set_camera_viewports,
                    // adjust_camera_drag_point_viewports,
                    adjust_camera_margines,
                    adjust_camera_drag_resize_button_transform_on_camera_movement,
                    adjust_camera_drag_move_camera_in_world_button_transform_on_camera_movement,
                    adjust_camera_drag_move_camera_in_window_button_transform_on_camera_movement,
                ),
            )
            .add_systems(
                Update,
                (juster_camera_størrelse_dragning_retning
                    .run_if(in_state(CameraDragningJustering::PÅ)),),
            );
    }
}

#[derive(Component, Debug)]
struct CameraPosition {
    pos: UVec2,
}
#[derive(Component)]
struct CameraAbsolutePosition {
    y_høyde: u32,
}

#[derive(Clone, PartialEq, Debug)]
enum CameraMode {
    KVART,
    HALV,
    HEL,
    AV,
}

#[derive(Component)]
pub struct CameraViewportSetting {
    camera_modes: Vec<CameraMode>,
    active_camera_mode_index: usize,
}

impl CameraViewportSetting {
    fn next_camera_mode_index(&mut self) {
        self.active_camera_mode_index =
            (self.active_camera_mode_index + 1) % self.camera_modes.len();
    }
    fn get_camera_mode(&self) -> CameraMode {
        self.camera_modes[self.active_camera_mode_index].clone()
    }
}

#[derive(Component)]
pub struct AllIndividerCameraTag;
#[derive(Component)]
pub struct NetverkInFokusCameraTag;
#[derive(Component)]
pub struct AllIndividerWindowTag;

#[derive(Component)]
pub struct PopulasjonMenyCameraTag;

#[derive(Component)]
pub struct KnapperMenyCameraTag;

#[derive(Component)]
struct SecondaryWindowTag;

pub const RENDER_LAYER_NETTVERK: usize = 0;
pub const RENDER_LAYER_ALLE_INDIVIDER: usize = 1;
pub const RENDER_LAYER_POPULASJON_MENY: usize = 2;
pub const RENDER_LAYER_TOP_BUTTON_MENY: usize = 3;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CameraDragningJustering {
    #[default]
    PÅ,
    AV,
}

/// Determine what do on left click.
#[derive(Resource, Debug)]
enum CameraDragningModus {
    /// Do nothing.
    Nothing,
    /// Move the window on left click.
    Move,
    /// Resize the window on left click.
    Resize,
}

/// What direction index should the window resize toward.
#[derive(Resource)]
pub struct ResizeDir(usize);

/// Liste over hvilke
// #[derive(Component)]
// struct CameraContainerRegisterList{
//     camera_entitis : Vec<Entity>
// }
#[derive(Component)]
struct CameraContainerRegisterList {
    camera_entitis: Vec<Entity>,
}

/// Determine what do on left click.
#[derive(Resource, Debug)]
enum LeftClickAction {
    /// Do nothing.
    // Nothing,
    /// Move the window on left click.
    // Move,
    /// Resize the window on left click.
    Resize,
}

/// Directions that the drag resizes the window toward.
const DIRECTIONS: [CompassOctant; 8] = [
    CompassOctant::North,
    CompassOctant::NorthEast,
    CompassOctant::East,
    CompassOctant::SouthEast,
    CompassOctant::South,
    CompassOctant::SouthWest,
    CompassOctant::West,
    CompassOctant::NorthWest,
];

fn juster_camera_størrelse_dragning_retning(
    // mut action: ResMut<LeftClickAction>,
    mut dir: ResMut<ResizeDir>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyS) {
        dir.0 = dir
            .0
            .checked_sub(1)
            .unwrap_or(DIRECTIONS.len().saturating_sub(1));
    }
    if input.just_pressed(KeyCode::KeyD) {
        dir.0 = (dir.0 + 1) % DIRECTIONS.len();
    }
}

fn juster_camera_størrelse_med_dragning(
    mut windows: Query<&mut Window>,
    mut cameras: Query<&mut Window>,
    action: Res<CameraDragningModus>,
    input: Res<ButtonInput<MouseButton>>,
    dir: Res<ResizeDir>,
) {
    // Both `start_drag_move()` and `start_drag_resize()` must be called after a
    // left mouse button press as done here.
    //
    // winit 0.30.5 may panic when initiated without a left mouse button press.
    if input.just_pressed(MouseButton::Left) {
        for mut window in windows.iter_mut() {
            match *action {
                CameraDragningModus::Nothing => (),
                CameraDragningModus::Move => window.start_drag_move(),
                CameraDragningModus::Resize => {
                    let d = DIRECTIONS[dir.0];
                    window.start_drag_resize(d);
                }
            }
        }
    }
}

trait FindTargetWindowForCamera {
    fn is_window_target_primary(&self) -> bool;
    fn get_window_target_entity(&self) -> Option<Entity>;
}

impl FindTargetWindowForCamera for RenderTarget {
    fn is_window_target_primary(&self) -> bool {
        match self {
            RenderTarget::Window(window_ref) => {
                return match window_ref {
                    WindowRef::Primary => true,
                    _ => false,
                };
            }
            _ => false,
        }
    }
    fn get_window_target_entity(&self) -> Option<Entity> {
        match self {
            RenderTarget::Window(window_ref) => {
                return match window_ref {
                    WindowRef::Primary => Option::None,
                    WindowRef::Entity(entity) => Some(*entity),
                };
            }
            _ => Option::None,
        }
    }
}

#[derive(Component)]
struct KameraEdgeResizeDragButton;

#[derive(Component)]
struct KameraEdgeMoveCameraInTheWorldDragButton;
#[derive(Component)]
struct KameraEdgeMoveCameraInTheWindowDragButton;

pub fn spawn_camera_move_in_world_button_for_neuron_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    kamera_query: Query<(Entity), With<NetverkInFokusCameraTag>>, // kan gjøres generisk ved å ta inn denne taggen som en <T>
) {
    let material_handle: Handle<ColorMaterial> = materials.add(Color::from(RED));
    let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(10.0, 10.0));
    kamera_query
        .get_single()
        .expect("Kamera eksisterer ikke :(");
    for (kamera_entity) in kamera_query.iter() {
        let mut parent_kamera = commands.get_entity(kamera_entity);

        parent_kamera.unwrap().with_children(|parent_builder| {
            parent_builder
                .spawn((
                    Mesh2d(rectangle_mesh_handle.clone().into()),
                    MeshMaterial2d(material_handle.clone().into()),
                    Transform::from_xyz(0.0, 0.0, 10.0), // will be adjusted on camera move events
                    KameraEdgeMoveCameraInTheWorldDragButton,
                    RenderLayers::layer(RENDER_LAYER_NETTVERK),
                ))
                .observe(camera_drag_to_move_camera_in_the_world);
        });
    }
}

pub fn spawn_camera_move_on_screen_button_for_neuron_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    kamera_query: Query<(Entity), With<NetverkInFokusCameraTag>>, // kan gjøres generisk ved å ta inn denne taggen som en <T>
) {
    let material_handle: Handle<ColorMaterial> = materials.add(Color::from(BLUE));
    let mesh_handle: Handle<Mesh> = meshes.add(Triangle2d::new(
        Vec2::new(-15., 0.),
        Vec2::new(15., 0.),
        Vec2::new(0., 15.),
    ));

    kamera_query.single().expect("Kamera eksisterer ikke :(");
    for (kamera_entity) in kamera_query.iter() {
        let mut parent_kamera = commands.get_entity(kamera_entity);

        parent_kamera.unwrap().with_children(|parent_builder| {
            parent_builder
                .spawn((
                    Mesh2d(mesh_handle.clone().into()),
                    MeshMaterial2d(material_handle.clone().into()),
                    Transform::from_xyz(0.0, 0.0, 10.0), // will be adjusted on camera move events
                    KameraEdgeMoveCameraInTheWindowDragButton,
                    RenderLayers::layer(RENDER_LAYER_NETTVERK),
                ))
                .observe(camera_drag_to_move_camera_in_the_window);
        });
    }
}

pub fn spawn_camera_resize_button_for_neuron_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    kamera_query: Query<(Entity, &Camera), With<NetverkInFokusCameraTag>>, // kan gjøres generisk ved å ta inn denne taggen som en <T>
) {
    let material_handle: Handle<ColorMaterial> = materials.add(Color::from(RED));

    // let top_left_corner = UVec2::default();
    // let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(top_left_corner.x as f32, top_left_corner.y as f32));
    let rectangle_mesh_handle: Handle<Mesh> = meshes.add(Rectangle::new(10.0, 10.0));
    kamera_query
        .get_single()
        .expect("Kamera eksisterer ikke :(");
    for (kamera_entity, kamera) in kamera_query.iter() {
        let viewport = kamera.clone().viewport.unwrap().clone();
        // camera in top_left_corner has physical positon 0.0. Transform 0.0 is drawn at center of camera.
        // but as a child to the parent camera, the transform is relative to parent , and not global
        // let left_side = viewport.physical_position.x as f32 - (viewport.physical_size.x as f32 * 0.5);
        // let top_side = viewport.physical_position.y as f32 - (viewport.physical_size.y as f32 * 0.5);
        let left_side = -(viewport.physical_size.x as f32 * 0.5);
        let top_side = (viewport.physical_size.y as f32 * 0.5);
        dbg!(left_side, top_side);
        let mut parent_kamera = commands.get_entity(kamera_entity);

        parent_kamera.unwrap().with_children(|parent_builder| {
            parent_builder
                .spawn((
                    Mesh2d(rectangle_mesh_handle.clone().into()),
                    MeshMaterial2d(material_handle.clone().into()),
                    Transform::from_xyz(left_side, top_side, 10.0),
                    KameraEdgeResizeDragButton,
                    RenderLayers::layer(RENDER_LAYER_NETTVERK),
                ))
                .observe(camera_drag_to_resize);
        });
    }
}

pub fn setup_camera(
    mut commands: Commands,
    query: Query<Entity, With<Window>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    // mut meshes: ResMut<Assets<Mesh>>,
) {
    // let color_material_handle: Handle<ColorMaterial> = materials.add(Color::from(RED));
    let color_material_handle: Handle<ColorMaterial> = materials.add(Color::from(CYAN_950));
    // commands.spawn(Camera2d::default());
    commands
        .entity(query.single().unwrap())
        .insert((AllIndividerWindowTag));

    // Spawn a second window
    let second_window = commands
        .spawn((
            Window {
                title: "Second window".to_owned(),
                ..default()
            },
            // AllIndividerWindowTag
        ))
        .id();

    // todo. Window som brukes av et camera er en enum component

    let camera_pos_1 = Vec3::new(0.0, 200.0, 150.0);
    let camera_pos_2 = Vec3::new(150.0, 150., 50.0);
    commands
        .spawn((
            Camera2d::default(),
            // Transform::from_translation(camera_pos_1).looking_at(Vec3::ZERO, Vec3::Y),
            Camera {
                // Renders cameras with different priorities to prevent ambiguities
                order: 0 as isize,
                target: RenderTarget::Window(WindowRef::Entity(second_window)),
                viewport: Some(Viewport::default()),
                ..default()
            },
            CameraPosition {
                pos: UVec2::new((0 % 2) as u32, (0 / 2) as u32),
                // pos: UVec2::new((0 % 2) as u32, (0) as u32),
            },
            CameraViewportSetting {
                camera_modes: vec![
                    CameraMode::HALV,
                    CameraMode::KVART,
                    CameraMode::AV,
                    CameraMode::HEL,
                ],
                active_camera_mode_index: 1,
            },
            NetverkInFokusCameraTag,
            RenderLayers::from_layers(&[RENDER_LAYER_NETTVERK]),
        ))
        .with_children(|spawner| {
            spawn_camera_margins(
                color_material_handle.clone(),
                spawner,
                RENDER_LAYER_NETTVERK,
            );
        });

    commands.spawn((
        Camera2d::default(),
        // Transform::from_translation(camera_pos_1).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            // Renders cameras with different priorities to prevent ambiguities
            order: 1 as isize,
            ..default()
        },
        CameraPosition {
            pos: UVec2::new((1 % 2) as u32, (1 / 2) as u32),
            // pos: UVec2::new((1 % 2) as u32, (1) as u32),
        },
        // AllIndividerCamera{ camera_mode: CameraMode::HALV },
        AllIndividerCameraTag,
        CameraViewportSetting {
            camera_modes: vec![
                CameraMode::HALV,
                CameraMode::KVART,
                CameraMode::AV,
                CameraMode::HEL,
            ],
            active_camera_mode_index: 3,
        },
        RenderLayers::from_layers(&[RENDER_LAYER_ALLE_INDIVIDER]),
    ));

    commands
        .spawn((
            Camera2d::default(),
            // Transform::from_translation(camera_pos_1).looking_at(Vec3::ZERO, Vec3::Y),
            Camera {
                // Renders cameras with different priorities to prevent ambiguities
                order: 2 as isize,
                target: RenderTarget::Window(WindowRef::Entity(second_window)),
                ..default()
            },
            CameraViewportSetting {
                camera_modes: vec![
                    CameraMode::HALV,
                    CameraMode::KVART,
                    CameraMode::AV,
                    CameraMode::HEL,
                ],
                active_camera_mode_index: 1,
            },
            CameraPosition {
                pos: UVec2::new((2 % 2) as u32, (2 / 2) as u32),
                // pos: UVec2::new((1 % 2) as u32, (1) as u32),
            },
            PopulasjonMenyCameraTag,
            RenderLayers::from_layers(&[RENDER_LAYER_POPULASJON_MENY]),
        ))
        .with_children(|parent_builder: &mut ChildSpawnerCommands<'_>| {
            // default values since they will be changed when camera is moved (events)
            spawn_camera_margins(
                color_material_handle.clone(),
                parent_builder,
                RENDER_LAYER_POPULASJON_MENY,
            );
        });

    commands
        .spawn((
            Camera2d::default(),
            Camera {
                // Renders cameras with different priorities to prevent ambiguities
                order: 3 as isize,
                target: RenderTarget::Window(WindowRef::Entity(second_window)),
                ..default()
            },
            CameraAbsolutePosition {
                y_høyde: 50,
                // pos: UVec2::new((1 % 2) as u32, (1) as u32),
            },
            KnapperMenyCameraTag,
            RenderLayers::from_layers(&[RENDER_LAYER_TOP_BUTTON_MENY]),
        ))
        .with_children(|parent_builder| {
            // default values since they will be changed when camera is moved (events)
            spawn_camera_margins(
                color_material_handle.clone(),
                parent_builder,
                RENDER_LAYER_TOP_BUTTON_MENY,
            );
        });
}

fn spawn_camera_margins(
    color_material_handle: Handle<ColorMaterial>,
    spawner: &mut RelatedSpawnerCommands<ChildOf>,
    render_target: usize,
) {
    // default values since they will be changed when camera is moved (events)
    spawner.spawn((
        CameraMarginDirection::TOPP,
        Transform::from_xyz(0.0, 0.0, 1.0),
        Mesh2d::default(),
        MeshMaterial2d(color_material_handle.clone()),
        RenderLayers::from_layers(&[render_target]),
    ));
    spawner.spawn((
        CameraMarginDirection::VENSTRE,
        Transform::from_xyz(0.0, 0.0, 1.0),
        Mesh2d::default(),
        MeshMaterial2d(color_material_handle.clone()),
        RenderLayers::from_layers(&[render_target]),
    ));
    spawner.spawn((
        CameraMarginDirection::HØYRE,
        Transform::from_xyz(0.0, 0.0, 1.0),
        Mesh2d::default(),
        MeshMaterial2d(color_material_handle.clone()),
        RenderLayers::from_layers(&[render_target]),
    ));
    spawner.spawn((
        CameraMarginDirection::BUNN,
        Transform::from_xyz(0.0, 0.0, 1.0),
        Mesh2d::default(),
        MeshMaterial2d(color_material_handle.clone()),
        RenderLayers::from_layers(&[render_target]),
    ));
}

fn set_camera_viewports(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    secondary_windows: Query<&Window, Without<PrimaryWindow>>,
    mut resize_events: EventReader<WindowResized>,
    mut absolutt_kamera_query: Query<
        (&CameraAbsolutePosition, &mut Camera),
        (
            With<KnapperMenyCameraTag>,
            Without<CameraPosition>,
            Without<AllIndividerCameraTag>,
        ),
    >,
    mut kamera_med_størrelse_og_posisjon_settings_query: Query<
        (&CameraPosition, &mut Camera, &CameraViewportSetting),
        (Without<CameraAbsolutePosition>),
    >,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.read() {
        // let primary_window = primary_window.get(resize_event.window);

        if let Ok(window) = primary_window.get(resize_event.window) {
            // Only main camera nees to be resized
            println!("resize_event for PRIMARY window");

            let window_size = window.physical_size();
            let kvart_skjerm_størrelse = window_size / 2;
            let halv_skjerm_størrelse = UVec2::new(window_size.x / 2, window_size.y);

            println!("------------");
            for (camera_position, mut camera, viewport_settings) in
                &mut kamera_med_størrelse_og_posisjon_settings_query
            {
                if !camera.target.is_window_target_primary() {
                    println!(
                        "camera er ikke i primary window, og trenger ikke å resize når primary vinduer endrer seg"
                    );
                    continue;
                }
                adjust_camera_viewport_according_to_settings(
                    window_size,
                    kvart_skjerm_størrelse,
                    halv_skjerm_størrelse,
                    camera_position,
                    &mut camera,
                    viewport_settings,
                );
            }
        };
        if let Ok(window) = secondary_windows.get(resize_event.window) {
            // Only cameras in secondary windows needs to be resized
            println!("resize_event for secondary window");

            // Knapp meny er alltid i andre windu
            let (knapp_meny_position, mut knapp_meny_camera) =
                absolutt_kamera_query.single_mut().unwrap();

            println!(
                "knapp_meny_camera er i primary window : {} ,  ",
                knapp_meny_camera.target.is_window_target_primary(),
                // knapp_meny_camera.target.get_window_target_entity().unwrap()
            );
            let window_without_meny_size = window.physical_size()
                - UVec2 {
                    x: 0,
                    y: knapp_meny_position.y_høyde,
                };
            let kvart_skjerm_størrelse = window_without_meny_size / 2;
            let halv_skjerm_størrelse =
                UVec2::new(window_without_meny_size.x / 2, window_without_meny_size.y);

            // TODO FINN HVILKE KAMERASER SOM ER I WINDOW

            println!("------------");
            for (camera_position, mut camera, viewport_settings) in
                &mut kamera_med_størrelse_og_posisjon_settings_query
            {
                if camera.target.is_window_target_primary() {
                    println!(
                        "camera er i primary window, og trenger ikke å resize når secondary vinduer endrer seg"
                    );
                    continue;
                }
                if let Some(entity) = camera.target.get_window_target_entity() {
                    if entity != resize_event.window {
                        println!(
                            "camera er i feil seconday window, og trenger ikke å resize når et annet secondary vindu endrer seg"
                        );
                        continue;
                    }
                }
                adjust_camera_viewport_according_to_settings(
                    window_without_meny_size,
                    kvart_skjerm_størrelse,
                    halv_skjerm_størrelse,
                    camera_position,
                    &mut camera,
                    viewport_settings,
                );
            }
            knapp_meny_camera.viewport = Some(Viewport {
                physical_position: UVec2 {
                    x: 0,
                    y: window_without_meny_size.y,
                },
                physical_size: UVec2 {
                    x: window_without_meny_size.x,
                    y: knapp_meny_position.y_høyde,
                },
                ..default()
            });
        };

        // let window = windows.get(resize_event.window).unwrap();

        // let window_without_meny_size = window.physical_size()- UVec2::new(10 as u32, 10 as u32) ;
        // let window_without_meny_size = window.physical_size();

        // dbg!(&window_without_meny_size);
        // dbg!(&kvart_skjerm_størrelse);
        // println!("------------");

        // for (camera_position, mut camera) in &mut kvart_kamera_query {
        //     // println!("Kvart-kamera justeres");
        //     // dbg!(&kvart_skjerm_størrelse);
        //     // dbg!(&camera_position.pos);
        //     camera.viewport = Some(Viewport {
        //         physical_position: camera_position.pos * kvart_skjerm_størrelse,
        //         physical_size: kvart_skjerm_størrelse,
        //         ..default()
        //     });
        // dbg!(&camera.viewport);
    }
}

// #[derive(Component, Debug)]
// struct CameraMargin {
//     topp: CameraMarginRectangle,
//     høyre: CameraMarginRectangle,
//     venstre: CameraMarginRectangle,
//     bunn: CameraMarginRectangle,
// }

// #[derive(Component, Debug)]
// struct CameraMargin {

#[derive(Component, Debug)]
// #[require(Transform)]
struct CameraMarginRectangle {
    direction: CameraMarginDirection,
    transform: Transform,
    mesh: Mesh2d,
    mesh_material: MeshMaterial2d<ColorMaterial>,
    // render_layers: RenderLayers,
}
#[derive(Component, Debug)]
enum CameraMarginDirection {
    TOPP,
    VENSTRE,
    HØYRE,
    BUNN,
}

fn adjust_camera_margines(
    // mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    kamera_query: Query<
        (Entity, &Camera, &Transform, &Children),
        (Changed<Camera>, Without<CameraMarginDirection>),
    >,
    mut margin_query: Query<
        (&CameraMarginDirection, &mut Mesh2d, &mut Transform),
        With<CameraMarginDirection>,
    >,
) {
    for (kamera_entity, camera, camera_transform, barn_marginer) in kamera_query.iter() {
        if let Some(viewport) = &camera.viewport {
            let view_dimensions = Vec2 {
                x: viewport.physical_size.x as f32,
                y: viewport.physical_size.y as f32,
            };

            let bredde_rectangle_mesh_handle: Handle<Mesh> =
                meshes.add(Rectangle::new(view_dimensions.x, 10.0));
            let høyde_rectangle_mesh_handle: Handle<Mesh> =
                meshes.add(Rectangle::new(10.0, view_dimensions.y));
            // let color_material_handle = materials.add(Color::from(RED));
            // let color_material_handle = materials.add(Color::from(CYAN_950));

            // camera in top_left_corner has physical positon 0.0. Transform 0.0 is drawn at center of camera
            // Siden marginer er children av kamera, så er transform.translation til margin relativ til parent-kamera sin transform.translation
            let padding = 5.0;
            let venstre_side_x = -(view_dimensions.x * 0.5) + padding;
            let høyre_side_x = (view_dimensions.x * 0.5) - padding;
            let top_side_y = (view_dimensions.y * 0.5) - padding;
            let bunn_side_y = -(view_dimensions.y * 0.5) + padding;
            for barn_margin_ref in barn_marginer {
                if let Ok((direction, mut mesh, mut transform)) =
                    margin_query.get_mut(*barn_margin_ref)
                {
                    match direction {
                        CameraMarginDirection::TOPP => {
                            mesh.0 = bredde_rectangle_mesh_handle.clone();
                            transform.translation.x = 0.0;
                            transform.translation.y = top_side_y;
                        }
                        CameraMarginDirection::VENSTRE => {
                            mesh.0 = høyde_rectangle_mesh_handle.clone();
                            transform.translation.x = venstre_side_x;
                            transform.translation.y = 0.0;
                        }
                        CameraMarginDirection::HØYRE => {
                            mesh.0 = høyde_rectangle_mesh_handle.clone();
                            transform.translation.x = høyre_side_x;
                            transform.translation.y = 0.0;
                        }
                        CameraMarginDirection::BUNN => {
                            mesh.0 = bredde_rectangle_mesh_handle.clone();
                            transform.translation.x = 0.0;
                            transform.translation.y = bunn_side_y;
                        }
                    }
                }
            }
        };
    }
}

fn adjust_camera_drag_resize_button_transform_on_camera_movement(
    kamera_query: Query<
        (&Camera, &Children),
        (Changed<Camera>, Without<KameraEdgeResizeDragButton>),
    >,
    mut button_query: Query<&mut Transform, With<KameraEdgeResizeDragButton>>,
) {
    for (camera, barn_marginer) in kamera_query.iter() {
        if let Some(viewport) = &camera.viewport {
            let view_dimensions = Vec2 {
                x: viewport.physical_size.x as f32,
                y: viewport.physical_size.y as f32,
            };

            // camera in top_left_corner has physical positon 0.0. Transform 0.0 is drawn at center of camera
            // Siden marginer er children av kamera, så er transform.translation til margin relativ til parent-kamera sin transform.translation
            let padding = 5.0;
            let venstre_side_x = -(view_dimensions.x * 0.5) + padding;
            let høyre_side_x = (view_dimensions.x * 0.5) - padding;
            let top_side_y = (view_dimensions.y * 0.5) - padding;
            let bunn_side_y = -(view_dimensions.y * 0.5) + padding;
            for barn_margin_ref in barn_marginer {
                // akkurat nå så er knappen alltid nede til høyre
                if let Ok((mut transform)) = button_query.get_mut(*barn_margin_ref) {
                    transform.translation.x = høyre_side_x;
                    transform.translation.y = bunn_side_y;
                }
            }
        };
    }
}

fn adjust_camera_drag_move_camera_in_world_button_transform_on_camera_movement(
    kamera_query: Query<
        (&Camera, &Children),
        (
            Changed<Camera>,
            Without<KameraEdgeMoveCameraInTheWorldDragButton>,
        ),
    >,
    mut button_query: Query<&mut Transform, With<KameraEdgeMoveCameraInTheWorldDragButton>>,
) {
    for (camera, barn_marginer) in kamera_query.iter() {
        if let Some(viewport) = &camera.viewport {
            let view_dimensions = Vec2 {
                x: viewport.physical_size.x as f32,
                y: viewport.physical_size.y as f32,
            };

            // camera in top_left_corner has physical positon 0.0. Transform 0.0 is drawn at center of camera
            // Siden marginer er children av kamera, så er transform.translation til margin relativ til parent-kamera sin transform.translation
            let padding = 15.0;
            let venstre_side_x = -(view_dimensions.x * 0.5) + padding;
            let høyre_side_x = (view_dimensions.x * 0.5) - padding;
            let top_side_y = (view_dimensions.y * 0.5) - padding;
            let bunn_side_y = -(view_dimensions.y * 0.5) + padding;
            for barn_margin_ref in barn_marginer {
                // akkurat nå så er knappen alltid nede til høyre
                if let Ok((mut transform)) = button_query.get_mut(*barn_margin_ref) {
                    transform.translation.x = høyre_side_x;
                    transform.translation.y = bunn_side_y;
                }
            }
        };
    }
}

fn adjust_camera_drag_move_camera_in_window_button_transform_on_camera_movement(
    kamera_query: Query<
        (&Camera, &Children),
        (
            Changed<Camera>,
            Without<KameraEdgeMoveCameraInTheWindowDragButton>,
        ),
    >,
    mut button_query: Query<&mut Transform, With<KameraEdgeMoveCameraInTheWindowDragButton>>,
) {
    for (camera, barn_marginer) in kamera_query.iter() {
        if let Some(viewport) = &camera.viewport {
            let view_dimensions = Vec2 {
                x: viewport.physical_size.x as f32,
                y: viewport.physical_size.y as f32,
            };

            // camera in top_left_corner has physical positon 0.0. Transform 0.0 is drawn at center of camera
            // Siden marginer er children av kamera, så er transform.translation til margin relativ til parent-kamera sin transform.translation
            let padding = 20.0;
            let venstre_side_x = -(view_dimensions.x * 0.5) + padding;
            let høyre_side_x = (view_dimensions.x * 0.5) - padding;
            let top_side_y = (view_dimensions.y * 0.5) - padding;
            let bunn_side_y = -(view_dimensions.y * 0.5) + padding;
            for barn_margin_ref in barn_marginer {
                // akkurat nå så er knappen alltid nede til høyre
                if let Ok((mut transform)) = button_query.get_mut(*barn_margin_ref) {
                    transform.translation.x = høyre_side_x;
                    transform.translation.y = bunn_side_y;
                }
            }
        };
    }
}

fn adjust_camera_viewport_according_to_settings(
    window_size: UVec2,
    kvart_skjerm_størrelse: UVec2,
    halv_skjerm_størrelse: UVec2,
    camera_position: &CameraPosition,
    camera: &mut Mut<Camera>,
    viewport_settings: &CameraViewportSetting,
) {
    println!("variabel-kamera er i vindu som ble endret og vil bli justert");
    let camera_viewport_size_setting = viewport_settings.get_camera_mode();
    // dbg!(&camera_viewport_size_setting);
    // dbg!(&camera_position.pos);

    println!(
        "variabel_kamera_query er i primary window : {}  ",
        camera.target.is_window_target_primary(),
        // camera.target.get_window_target_entity().unwrap()
    );

    if camera_viewport_size_setting != CameraMode::AV {
        camera.viewport = Some(Viewport {
            physical_position: match camera_viewport_size_setting {
                CameraMode::HALV => camera_position.pos * halv_skjerm_størrelse,
                CameraMode::KVART => camera_position.pos * kvart_skjerm_størrelse,
                CameraMode::AV => UVec2::default(),
                CameraMode::HEL => UVec2::default(),
            },
            physical_size: match camera_viewport_size_setting {
                CameraMode::HALV => halv_skjerm_størrelse,
                CameraMode::KVART => kvart_skjerm_størrelse,
                CameraMode::AV => UVec2::default(),
                CameraMode::HEL => window_size,
            },
            ..default()
        });
    }

    if camera_viewport_size_setting == CameraMode::AV {
        camera.is_active = false;
    } else if !camera.is_active {
        camera.is_active = true;
    }
}

pub fn resize_alle_individer_camera(
    trigger: Trigger<Pointer<Click>>,
    // mut cameras: Query<&mut Camera &AllIndividerCamera, With<AllIndividerCamera>>,
    mut cameras: Query<
        (Entity, &mut Camera, &mut CameraViewportSetting),
        With<AllIndividerCameraTag>,
    >,
    // TENKE TENKE... KAN HA HVILKE KAMERAER SOM ER AKTIVE I EN RES. ELLER JEG KAN GI DE EN KOMPENENT MED BOLEAN AKTIVE INAKTIVE VERDI, OG LYTTE PÅ CHAGES
    //     // kANSKJE OGSÅ TRIGGRE WINDOW RESIZE EVENT, SLIK AT DE BLIR TILPASSET PÅ NYTT....
    //     // iSTEDENFOR ON OFF. Kvart skjer, halv skjerm, hel skjerm, av.
    mut resize_camera_windows: EventWriter<WindowResized>,
    mut commands: Commands,
    windows: Query<(Entity, &Window), With<AllIndividerWindowTag>>,
) {
    if let Ok((entity, camera, mut camera_viewport_settings)) = cameras.single_mut() {
        camera_viewport_settings.next_camera_mode_index();
        // Trigger event to make system that handles viewport changes to run
        let (window_entity, window) = windows.get_single().unwrap();
        resize_camera_windows.send(WindowResized {
            window: window_entity,
            width: window.size().x,
            height: window.size().y,
        });
    }
}

fn camera_drag_to_resize(
    drag: Trigger<Pointer<Drag>>,
    mut kamera_query: Query<(&mut Camera), With<NetverkInFokusCameraTag>>,
    window_query: Query<&Window>,
) {
    // println!("dragging camera");
    // if let Ok(mut camera) = kamera_query.get_mut(drag.target) {
    if let Ok(mut camera) = kamera_query.get_single_mut() {
        println!("resizing camera2");

        // finn window størrelse for å vite max endring vi kan gjøre (krasjer hvis går over vindu)
        let window = window_query
            .get(camera.target.get_window_target_entity().unwrap())
            .unwrap(); // jeg er usikker på om camera i primary vindu har ressultat av get_window_target_entity.
        let window_size = window.physical_size();

        let potensiell_viewport: &mut Option<Viewport> = &mut camera.viewport;

        if let Some(viewport) = potensiell_viewport {
            let u32_vektor: &mut UVec2 = &mut viewport.physical_size;
            drag_u32_vektor_med_potensielt_negative_i32_verdier(drag, u32_vektor);

            // camera.pos.x + camera.size.x <= window_size.x
            // camera.size.x <= window_size.x - camera.pos.x
            u32_vektor.x = min(window_size.x - viewport.physical_position.x, u32_vektor.x);
            u32_vektor.y = min(window_size.y - viewport.physical_position.y, u32_vektor.y);
        }
    }
}

fn drag_u32_vektor_med_potensielt_negative_i32_verdier(
    drag: Trigger<Pointer<Drag>>,
    u32_vektor: &mut UVec2,
) {
    let mut new_x_value = u32_vektor.x as i32;
    new_x_value += drag.delta.x as i32;
    u32_vektor.x = max(0, new_x_value) as u32;

    let mut new_y_value = u32_vektor.y as i32;
    new_y_value += drag.delta.y as i32;
    u32_vektor.y = max(0, new_y_value) as u32;
}

fn camera_drag_to_move_camera_in_the_world(
    drag: Trigger<Pointer<Drag>>,
    mut query: Query<(&mut Transform), With<NetverkInFokusCameraTag>>,
) {
    println!("dragging camera");
    // if let Ok(mut camera) = kamera_query.get_mut(drag.target) {
    if let Ok(mut transform) = query.get_single_mut() {
        println!("moving camera2");
        // more intuiative to invert
        transform.translation.x -= drag.delta.x;
        transform.translation.y += drag.delta.y;
    }
}

fn camera_drag_to_move_camera_in_the_window(
    drag: Trigger<Pointer<Drag>>,
    mut kamera_query: Query<(&mut Camera), With<NetverkInFokusCameraTag>>,
    window_query: Query<&Window>,
) {
    // println!("trigger moving camera in window");

    if let Ok(mut camera) = kamera_query.single_mut() {
        // println!("moving camera in window");
        // finn window størrelse for å vite max endring vi kan gjøre (krasjer hvis går over vindu)
        let window = window_query
            .get(camera.target.get_window_target_entity().unwrap())
            .unwrap();
        let window_size = window.physical_size();

        // move viewport

        if let Some(viewport) = &mut camera.viewport {
            let u32_vektor: &mut UVec2 = &mut viewport.physical_position;
            drag_u32_vektor_med_potensielt_negative_i32_verdier(drag, u32_vektor);

            u32_vektor.x = min(window_size.x - viewport.physical_size.x, u32_vektor.x);
            u32_vektor.y = min(window_size.y - viewport.physical_size.y, u32_vektor.y);
        }
    }
}

// if let &Some(mut old_viewport) = &camera.viewport {
// camera.viewport.unwrap().physical_size.x+=1;
//     old_viewport.physical_size.x +=1;
//     // camera.viewport.unwrap().physical_size.x += 20;
//     // old_viewport.physical_size.x += 1;
//     camera.viewport = Some(Viewport {
//         physical_position: UVec2 { x: old_viewport.physical_position.x + 10, y: old_viewport.physical_position.y },
//         physical_size: old_viewport.physical_size,
//         depth: old_viewport.depth,
//     }
//     );
// }
// }
// }
