# Start kode

# oppgradere ust

error: package `bevy v0.14.0` cannot be built because it requires rustc 1.79.0 or newer, while the currently active rustc version is 1.76.0
```cmd
rustup update stable
```

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
```

# Enum

```rust
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
} 
```

# Key input / tasteinput

```rust
fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut user_input: Res<ButtonInput<KeyCode>>,
) {
    if user_input.pressed(KeyA) {
        next_state.set(AppState::InGame);
    }
}
```

# Vise og oppdatere tekststrenger

```rust 
fn vis_tekst(mut commands: Commands) {
    let text_style = TextStyle {
        font_size: 60.0,
        color: Color::WHITE,
        ..default()
    };
    let text_justification = JustifyText::Center;
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("En fin tekst: 0", text_style.clone())
                .with_justify(text_justification),
            ..default()
        },
        TellerTekst,
    ));
}

fn oppdater_tekst(mut query: Query<&mut Text, With<TellerTekst>>) {
    let mut tekst = query.single_mut().unwrap();
    tekst.sections[0].value = "En fin tekst: 1".to_string();
}
```

## Forenklet tekststreng

```rust
commands.spawn(
TextBundle::from("From an &str into a TextBundle with the default font!").with_style(
Style {
position_type: PositionType::Absolute,
bottom: Val::Px(5.0),
left: Val::Px(15.0),
..default ()
},
)
);
```

## Tekststreng med Font (default har ikke æøå)

Må laste ned font fil, og legge den i "assets" mappa

```rust
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "hello\nbevy!",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 100.0,
                ..default()
            },
        ) // Set the justification of the Text
            .with_text_justify(JustifyText::Center)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(5.0),
                right: Val::Px(5.0),
                ..default()
            }),
        ColorText,
    ));
```

# Component

```rust 
#[derive(Component)]
struct TellerTekst;
```

# Beveg Component

```rust
fn agent_action(mut query: Query<&mut Transform, With<Plank>>) {
    for mut individual in query.iter_mut() {
        individual.translation.x += 1.1;
    }
}
```

# Beveg med input

```rust 
fn move_plank(mut query: Query<&mut Transform, With<Plank>>,
              keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut delta_x = 0.0;
    if (keyboard_input.pressed(KeyA)) {
        delta_x -= PLANK_MOVEMENT_SPEED;
    }
    if (keyboard_input.pressed(KeyD)) {
        delta_x += PLANK_MOVEMENT_SPEED;
    }
    let mut transform = query.single_mut().unwrap();
    transform.translation.x += delta_x;
```

# State

Bevy har sin egen state ting

```

fn main() {  
    App::new()  
        .insert_state(KjøreTilstand::Meny)  
        .add_systems(Update, menu.run_if(in_state(KjøreTilstand::Menu)))  
		.add_systems(OnExit(KjøreTilstand::Menu), cleanup_menu)  
		.add_systems(OnEnter(KjøreTilstand::Kjørende), setup_game)
        .run();  
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]  
enum KjøreTilstand {  
    #[default]  
    Meny,  
    Kjørende,  
}
```

Endre state

```
fn change_state_to_menu(  
    mut next_state: ResMut<NextState<AppState>>,  
    mut user_input: Res<ButtonInput<KeyCode>>,  
) {  
    if user_input.pressed(KeyA) {  
        next_state.set(AppState::InGame);  
  
    }
}
```

## se state

Inside of systems, you can check the current state using the State<T> resource:

````rust
fn debug_current_gamemode_state(state: Res<State<MyGameModeState>>) {
eprintln!("Current state: {:?}", state.get());
}
````

# Hente inn kode med filnavn ÆØÅ

```rust 
#[path = "kjøretilstand_logikk.rs"]
mod kjøretilstand_logikk;
```

# Hente inn kode med mod

alt 1
````rust
mod ball;


.add_systems(Startup, (
spawn_camera,
ball::spawn_ball,
````

alternativ 2
```rust
mod ball;
mod bevy_application_teller;

use crate::bevy_application_teller::*;
```

# Hente inn dependency med kode som ikke er bygget til crate.io

Eksempel : Nyeste rapier master finnes i https://github.com/dimforge/bevy_rapier , men er ikke pushet til https://crates.io/crates/bevy_rapier2d/versions

Da kan vi bygge den selv, og inportere den. Eller enda lettere, peke på github

```toml
bevy_rapier2d = { git = "https://github.com/dimforge/bevy_rapier.git" }
```


# Rescource

```rust
#[derive(Resource, Debug)]
pub struct SpawnTimer {
    timer: Timer,
}

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnTimer {
            timer: Timer::from_seconds(SPAWN_TIME_SECONDS, TimerMode::Repeating),
        })
```

git remote add origin https://github.com/aDecoy/balanceAgain.git


# GPU hardware log warn spamming

`` ID3D12CommandQueue::ExecuteCommandLists: Using IDXGISwapChain::Present on Command List (0x00000220BA3AE130:'Internal DXGI CommandList'): Resource state (0x4: D3D12_RESOURCE_STATE_RENDER_TARGET) of resource (0x00000220BA424F20:'Unnamed ID3D12Resource Object') (subresource: 0) is invalid for use as a PRESENT_SOURCE.  Expected State Bits (all): 0x0: D3D12_RESOURCE_STATE_[COMMON|PRESENT], Actual State: 0x4: D3D12_RESOURCE_STATE_RENDER_TARGET, Missing State: 0x0: D3D12_RESOURCE_STATE_[COMMON|PRESENT]. [ EXECUTION ERROR #538: INVALID_SUBRESOURCE_STATE]
``

````rust
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
````
````rust
  .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::DX12),
                ..default()
            }),
            synchronous_pipeline_compilation: false,
        }))
````

# World inspector plugin 


``bevy-inspector-egui = "0.24.0"``
````rust
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F12)),
````

# Close on ESC

``         .add_systems(Update, bevy::window::close_on_esc)``

# Rapier

```rust 
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
```

# Rapier ball

````rust
use bevy::sprite::MaterialMesh2dBundle;
use bevy_rapier2d::prelude::*;
````

```rust
#[derive(Component)]
struct Ball;

const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const BALL_STARTING_POSITION: Vec3 = Vec3 { x: 10.0, y: 10.0, z: 0.0 };
const BALL_DIAMETER: f32 = 50.0;

fn spawn_ball(mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>, ) {
    commands.spawn((
        RigidBody::Dynamic,
        Ball,
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(BALL_COLOR),
            transform: Transform::from_translation(BALL_STARTING_POSITION)
                .with_scale(Vec3 { x: BALL_DIAMETER, y: BALL_DIAMETER, z: BALL_DIAMETER }),
            ..default()
        },
        GravityScale(1.1),
        Sleeping::disabled(),
        Collider::ball(0.50),
        Velocity {
            linvel: Vec2::new(100.0, 2.0),
            angvel: 0.2,
        },
        Restitution::coefficient(1.7),
        Friction::coefficient(1.0),
    )
    );
}

```

# Telle antall Bevy itterasjoner / gameloops

```rust
// All diagnostics should have a unique DiagnosticPath. (https://github.com/bevyengine/bevy/blob/main/examples/diagnostics/custom_diagnostic.rs)
pub const SYSTEM_ITERATION_COUNT: DiagnosticPath = DiagnosticPath::const_new("system_iteration_count");

```

````rust
            .register_diagnostic(Diagnostic::new(SYSTEM_ITERATION_COUNT).with_suffix(" iterations"))
````

Kombinere Diagnostic, og Time, plassere frameCount inn i en Ressurs variabel : 

```rust
#[derive(Resource, Default)]
pub struct DiagnosticFrameCount(u32);

```
```rust
            .init_resource::<DiagnosticFrameCount>()
```

```rust
fn diagnostics_report(
    mut diagnostics: Diagnostics,
    mut frame_count: ResMut<DiagnosticFrameCount>,
    time: Res<Time<Real>>,
) {
    let delta = time.delta_seconds_f64();
    if delta == 0. { return; }
    diagnostics.add_measurement(&SYSTEM_ITERATION_COUNT, || {
        frame_count.0 as f64 / delta
    });
    frame_count.0 = 0;
}
```

# Kjøre-rekkefølge

```rust
        .add_systems(Update, (ball::reset_ball,
                              endre_kjøretilstand_ved_input,
                              (update_BevyAppRunningFrameCount,
                              oppdater_bevy_application_tellertekst).chain(),
        ))
```

# Bevy rapier kjøre rekkefølge 

```rust
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0).with_default_system_setup(false))

    .add_systems(Update, (
    RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend).in_set(PhysicsSet::SyncBackend),
    RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::StepSimulation).in_set(PhysicsSet::StepSimulation),
    RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback).in_set(PhysicsSet::Writeback),
    ).chain()// overasknede viktig. uten den så lagger ting
    .run_if(in_state(Kjøretilstand::Kjørende))
)
```


# Bevy close one esc


```rust
        .add_systems(Update, close_on_esc)
```
```rust
pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}
```


# Bevy : Pakke inn ting i plugins

Litt mer enn bare å flytte ut til fil og bruke mod.

```rust

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OrbitCamera>();
        app.add_systems(Update, apply_camera_controls);
        app.add_systems(Update, update_camera.after(apply_camera_controls));
    }
}
```

```rust
mod camera; // fila heter camera.rs
      .add_plugins(camera::OrbitCameraPlugin)

```

# Rust : hente inn med mod fra sudirectories

Ser ut som at vi trenger en mod.rs fil under mappa som igjen bruker mod til å hente inn det som skal være public utenfor mappa

```rust
use crate::environments::bouncing_ball::BouncingBallPlugin;

mod environments;
```
inne i envioronments/mod.rs :
```rust
pub mod bouncing_ball;
```

Og så må det man henter inn i main faktisk være public også :) 


# Bevy finn window size

https://github.com/bevyengine/bevy/blob/main/examples/window/window_settings.rs

```rust
fn get_window(window: Query<&Window>) {
    let window = window.single().unwrap();

    let width = window.width();
    let height = window.height();
```


```rust
pub(crate) fn spawn_simulation_tellertekst(mut commands: Commands, window: Query<&Window>) {
    let window = window.single().unwrap();

    info!("window size accoarding to Query<&Window>, {}, {}" , window.width(), window.height() );
    let text_style = TextStyle {
        font_size: 30.0,
        color: Color::WHITE,
        ..default()
    };
    let text_justification = JustifyText::Center;
    commands.spawn((
        Text2dBundle {
            text: Text::from_section("START", text_style.clone()).with_justify(text_justification),
            // transform: Transform::from_xyz(250.0, 250.0, 0.0),
            transform: Transform::from_xyz(window.width() *0.5- 200.0, window.height()*0.5 - 50.0, 0.0),
            // global_transform: GlobalTransform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        SimulationRunningTellerTekst,
    ));
}

fn resize_simulation_tellertekst(resize_events: EventReader<WindowResized>, mut query: Query<&mut Transform, With<SimulationRunningTellerTekst>>) {
    for e in resize_events.read() {
        let mut transform = query.single_mut().unwrap();
        println!("old translation {}", transform.translation);
        transform.translation.x =  e.width*0.5  - 200.0;
        // transform.translation.x =  200.0;
        transform.translation.y =  e.height*0.5 - 20.0;
        // transform.translation.y =  200.0;
        println!("new translation {}", transform.translation);

        println!("width = {} height = {}", e.width, e.height);
    }
}
```

# App.update()

Det viser seg at at app.update ikke kan vise frem GUI vinduet. Dette er fordi at winit som bevy bruker vil ha kontrol på alt, 
så vi sender på en måte inn en lambda funksjon til den. Derfor kan vi ikke stoppe midt i den og vente på nytt input slik som vi gjør i box2d

# Problemer med addSystem

https://bevy-cheatbook.github.io/pitfalls/into-system.html

https://bevy-cheatbook.github.io/builtins.html#systemparams

