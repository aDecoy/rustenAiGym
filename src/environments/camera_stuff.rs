use bevy::input::ButtonInput;
use bevy::math::{CompassOctant, UVec2, Vec3};
use bevy::prelude::*;
use bevy::render::camera::{RenderTarget, Viewport};
use bevy::render::view::RenderLayers;
use bevy::window::{PrimaryWindow, WindowRef, WindowResized};

pub struct MinCameraPlugin;

impl Plugin for MinCameraPlugin {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app.insert_state(CameraDragningJustering::PÅ)
            .insert_resource(ResizeDir(7))
            .add_systems(PreStartup, ((setup_camera,)))
            .add_systems(Update, (set_camera_viewports,));
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
                }
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
                }
            }
            _ => Option::None,
        }
    }
}

pub(crate) fn setup_camera(mut commands: Commands, query: Query<Entity, With<Window>>) {
    // commands.spawn(Camera2d::default());
    commands
        .entity(query.single())
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
    let camera = commands
        .spawn((
            Camera2d::default(),
            // Transform::from_translation(camera_pos_1).looking_at(Vec3::ZERO, Vec3::Y),
            Camera {
                // Renders cameras with different priorities to prevent ambiguities
                order: 0 as isize,
                target: RenderTarget::Window(WindowRef::Entity(second_window)),
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
            RenderLayers::from_layers(&[RENDER_LAYER_NETTVERK]),
        ))
        .id();
    let camera = commands
        .spawn((
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
                active_camera_mode_index: 0,
            },
            RenderLayers::from_layers(&[RENDER_LAYER_ALLE_INDIVIDER]),
        ))
        .id();
    let camera = commands
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
        .id();

    let camera = commands.spawn((
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
    // mut kvart_kamera_query: Query<
    //     (&CameraPosition, &mut Camera),
    //     (
    //         Without<AllIndividerCameraTag>,
    //         Without<CameraAbsolutePosition>,
    //     ),
    // >,
    mut kamera_med_størrelse_og_posisjon_settings_query: Query<
        (&CameraPosition, &mut Camera, &CameraViewportSetting),
        (Without<CameraAbsolutePosition>),
    >,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.

    // todo bruke  window_drag_move.rs eksempel til å endre på camera viewport størrelser inne i windu.

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
                    println!("camera er ikke i primary window, og trenger ikke å resize når primary vinduer endrer seg");
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
            let (knapp_meny_position, mut knapp_meny_camera) = absolutt_kamera_query.single_mut();

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
                    println!("camera er i primary window, og trenger ikke å resize når secondary vinduer endrer seg");
                    continue;
                }
                if let Some(entity) = camera.target.get_window_target_entity() {
                    if entity != resize_event.window {
                        println!("camera er i feil seconday window, og trenger ikke å resize når et annet secondary vindu endrer seg");
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
    let (entity, camera, mut camera_viewport_settings) = cameras.get_single_mut().unwrap();
    camera_viewport_settings.next_camera_mode_index();
    // Trigger event to make system that handles viewport changes to run
    let (window_entity, window) = windows.get_single().unwrap();
    resize_camera_windows.send(WindowResized {
        window: window_entity,
        width: window.size().x,
        height: window.size().y,
    });
}
