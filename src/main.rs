//#![windows_subsystem = "windows"] // Remove comment to turn off console log output

use bevy::{
    core::FixedTimestep,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, //sprite::collide_aabb::{collide, Collision},
    prelude::*,
};
use bevy_render::camera::{DepthCalculation, ScalingMode, WindowOrigin};
//use bevy_window::*;
//use bevy_winit::*;
mod math;

// TODO: Cooler asset loader: https://www.nikl.me/blog/2021/asset-handling-in-bevy-apps/#:~:text=Most%20games%20have%20some%20sort%20of%20loading%20screen,later%20states%20can%20use%20them%20through%20the%20ECS.
// TODO: Inspector:  https://bevy-cheatbook.github.io/setup/bevy-tools.html

// Resource = Prefab
// System = Behavior
// Component = Component
// Entity = Entity

const TIME_STEP: f32 = 1.0 / 60.0;
const PROJECT: &'static str = "AST4!";
const WIDTH: f32 = 800.0f32;
const HEIGHT: f32 = 600.0f32;

#[derive(Component)]
struct GameOverComponent;

#[derive(Component)]
struct Wrapped2dComponent;

#[derive(Component)]
struct FrameRateComponent;

#[derive(Component, Default)]
struct PlayerComponent {
    pub thrust: f32,
    pub player_index: u8, // Or 1, for 2 players
    pub friction: f32,
}

#[derive(Component, Default)]
struct VelocityComponent {
    pub v: Vec3,
    pub max_speed: f32, // magnitude.
}

impl VelocityComponent {
    pub fn apply_thrust(&mut self, thrust: f32, direction: &Quat, time: &Res<Time>) {
        let (_, _, angle_radians) = direction.to_euler(EulerRot::XYZ);
        let thrust_vector =
            thrust * Vec3::new(-f32::sin(angle_radians), f32::cos(angle_radians), 0f32);
        self.v = self.v + thrust_vector; // * time.delta_seconds();
        self.v = self.v.clamp_length_max(self.max_speed);
        println!(
            "max_speed: {}  thrust_vectory: {}",
            self.max_speed, thrust_vector
        );
    }

    pub fn apply_friction(&mut self, friction: f32, time: &Time) {
        self.v *= friction;
        if self.v.length() < 1f32 {
            self.v = Vec3::new(0f32, 0f32, 0f32);
        }
    }
}

#[derive(Component, Default)]
struct RotatorComponent {
    pub snap_angle: Option<f32>,
    pub rotate_speed: f32,    //= 150f;
    pub angle_increment: f32, // = 5.0f;
}

impl RotatorComponent {
    pub fn rotate_to_angle_with_snap(
        &mut self,
        transform: &mut Transform,
        horz: f32,
        time: &Res<Time>,
    ) {
        let (_, _, cur_angle) = transform.rotation.to_euler(EulerRot::XYZ); // cur angle in radians.
        if horz != 0.0f32 {
            // Assume horz is 1.0 or -1.0
            let angle_to_rotate = horz * self.rotate_speed * time.delta_seconds();
            let target_angle = cur_angle + angle_to_rotate;

            // create the change in rotation around the Z axis (pointing through the 2d plane of the screen)
            let rotation_delta = Quat::from_rotation_z(angle_to_rotate);
            // update the ship rotation with our rotation delta
            transform.rotation *= rotation_delta;

            // In case we have to stop, this will be the snap angle.
            let nearest = math::round_to_nearest_multiple(
                target_angle + horz * self.angle_increment, // tbd: this may be laggy.
                self.angle_increment,
            );

            // Snap to this angle on next frame if button released.
            self.snap_angle = Some(nearest);
        } else {
            // When button released, snap to next angle.
            match self.snap_angle {
                Some(snap_angle) => {
                    transform.rotation = Quat::from_rotation_z(snap_angle);
                    self.snap_angle = None;
                }
                None => {}
            }
        }
    }
}

//struct GameEntities {
//    pub game_over_entity: Option<Entity>,
//}

struct FrameRateResource {
    pub display_frame_rate: bool,
    pub debug_sinusoidal_frame_rate: bool,
    delta_time: f64,
    fps_last: f64,
}

struct GameStateResource {
    level: u32,
    next_free_life_score: u64,
}

fn main() {
    println!("Hello, world!");

    let mut new_app = App::new();

    new_app.
    insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .insert_resource(WindowDescriptor {
            title: PROJECT.to_string(),
            width: WIDTH,
            height: HEIGHT,
            vsync: true,
            cursor_visible: false,
            // decorations: false, // Hide the white flashing window at atartup, but hides console.
            // mode: bevy_window::WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)

        //.add_plugin( WindowPlugin { ..Default::default()})q
        //.add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        //  .insert_resource(Scoreboard { score: 0 })
        //.insert_resource(GameEntities {
        //    game_over_entity: None,
        //})
        .insert_resource(FrameRateResource {
            delta_time: 0f64,
            display_frame_rate: true,
            debug_sinusoidal_frame_rate: false,
            fps_last: 0f64,
        })
        .insert_resource(GameStateResource {
            level: 0,
            next_free_life_score: 10000
        })
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                //.add_system(game_over_system)
                .with_system(player_system)
                .with_system(wrapped_2d)
                //.with_system(paddle_movement_system)
                //.with_system(ball_collision_system)
                //.with_system(ball_movement_system),
        )
        //.add_system(scoreboard_system)
        //.add_system( change_title)
        .add_system(frame_rate)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

pub fn new_camera_2d() -> OrthographicCameraBundle {
    let far = 1000.0 - 0.1;
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection = OrthographicProjection {
        far,
        depth_calculation: DepthCalculation::ZDifference,
        scaling_mode: ScalingMode::None,
        top: 1f32,
        left: 0f32,
        right: 1f32,
        bottom: 0f32,
        window_origin: WindowOrigin::BottomLeft,
        ..Default::default()
    };
    camera.transform.scale = Vec3::new(WIDTH, HEIGHT, 1.);
    //camera.transform.translation = Vec3::new( -400., -300., 0.);
    return camera;
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut windows: ResMut<Windows>, //,   mut game_entities: ResMut<GameEntities>,
) {
    // hot reloading of assets.
    asset_server.watch_for_changes().unwrap();

    //let window = windows.get_primary_mut().unwrap();
    //window.set_resolution(WIDTH, HEIGHT);
    //window.height(300.f32);

    //window.set_cursor_visibility(false);

    // Add the game's entities to our world

    // cameras
    commands.spawn_bundle(new_camera_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let texture_handle = asset_server.load("textures/Atlas.png");
    let mut texture_atlas = TextureAtlas::new_empty(texture_handle, Vec2::new(128.0, 128.0));
    TextureAtlas::add_texture(
        &mut texture_atlas,
        bevy::sprite::Rect {
            min: Vec2::new(2.0, 2.0),
            max: Vec2::new(27.0, 32.0),
        },
    );
    //let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(25.0,25.0),1,1);

    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let size = Vec2::new(27.0, 32.0);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(0),
            transform: Transform {
                scale: Vec3::splat(1.0),
                translation: Vec3::new(50.0, 50.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(PlayerComponent {
            thrust: 2.0f32,
            friction: 0.98f32,
            player_index: 0,
        })
        .insert(Wrapped2dComponent) 
        .insert(RotatorComponent {
            snap_angle: None,
            angle_increment: (3.141592654f32 / 16.0f32),
            rotate_speed: 4.0f32,
        })
        .insert(VelocityComponent {
            v: Vec3::new(0f32, 0f32, 0f32),
            max_speed: 300.0f32,
        });

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 10.0,
                            color: Color::rgb(0.5, 0.5, 1.0),
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                            font_size: 10.0,
                            color: Color::rgb(1.0, 0.5, 0.5),
                        },
                    },
                ],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FrameRateComponent);

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "GAME OVER".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/Hyperspace.otf"),
                        font_size: 30.0,
                        color: Color::rgb(1.0, 1.0, 1.0),
                    },
                }],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Percent(40.0f32),
                    left: Val::Px(200.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(GameOverComponent);
}

/// This system will then change the title during execution
// fn change_title(time: Res<Time>, mut windows: ResMut<Windows>, fr: Res<FrameRate>) {
//     let window = windows.get_primary_mut().unwrap();
//     window.set_title(format!(
//         "{} - Seconds since startup: {} FPS: {:.1}", PROJECT,
//         time.seconds_since_startup().round(),
//         fr.fps_last
//     ));
// }

fn wrapped_2d(mut query: Query<(&Wrapped2dComponent, &mut Transform)>) {
    let (_, mut transform) = query.single_mut();

    println!("wrapped2d translation={}", transform.translation);


    let cam_rect_right: f32 = WIDTH;
    let cam_rect_left: f32 = 0.0f32;
    let cam_rect_top = HEIGHT;
    let cam_rect_bottom = 0.0f32;
    
    if transform.translation.x > cam_rect_right {
        transform.translation.x = cam_rect_left;
    } else if transform.translation.x < cam_rect_left {
        transform.translation.x = cam_rect_right;
    }
    if transform.translation.y > cam_rect_top {
        transform.translation.y = cam_rect_bottom;
    } else if transform.translation.y < cam_rect_bottom {
        transform.translation.y = cam_rect_top;
    }
}

fn player_system(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(
        &PlayerComponent,
        &mut RotatorComponent,
        &mut Transform,
        &mut VelocityComponent,
    )>,
) {
    // println!("Player");

    let (player, mut rotator, mut transform, mut velocity) = query.single_mut();

    let mut dir = 0.0f32;
    if keyboard_input.pressed(KeyCode::Left) {
        dir += 1.0f32;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        dir += -1.0f32;
    }
    rotator.rotate_to_angle_with_snap(&mut transform, dir, &time);

    let mut vert = 0.0f32;

    if keyboard_input.pressed(KeyCode::Up) {
        vert = 1.0f32;
    }
    // Maybe a thruster component? Or maybe Rotator+Thruster=PlayerMover component.
    if vert > 0.0f32 {
        // TOo much trouble to implement rigid body like in Unity, so wrote my own.
        // Assume no friction while accelerating.
        println!("rotation: {}", transform.rotation);
        velocity.apply_thrust(player.thrust, &transform.rotation, &time);

        /*
        if (_exhaustParticleSystem.isStopped)
        {
            _exhaustParticleSystem.loop = true;
            _exhaustParticleSystem.Play();
        }
        if (!_thrustAudioSource.isPlaying)
        {
            _thrustAudioSource.loop = true;
            _thrustAudioSource.Play();
            Debug.Assert(_thrustAudioSource.isPlaying);
        }
        */
    } else {
        velocity.apply_friction(player.friction, &time);
        /* TODO
        if (_exhaustParticleSystem.isPlaying)
        {
            _exhaustParticleSystem.Stop();
        }
        if (_thrustAudioSource.isPlaying)
        {
            _thrustAudioSource.Stop();
            Debug.Assert(!_thrustAudioSource.isPlaying);
        }
        */
    }
    //  Move forward in direction of velocity.
    transform.translation += velocity.v * time.delta_seconds();

}

fn game_over_system(_: Query<(&Text, &GameOverComponent)>) {
    println!("Game over");
}

fn frame_rate(
    time: Res<Time>,
    mut fr: ResMut<FrameRateResource>,
    mut query: Query<(&mut Text, &FrameRateComponent)>,
) {
    fr.delta_time += (time.delta_seconds_f64() - fr.delta_time) * 0.1f64;
    fr.fps_last = 1.0f64 / fr.delta_time;

    if fr.debug_sinusoidal_frame_rate {
        // TODO
    }

    if fr.display_frame_rate || true {
        let (mut text, _) = query.single_mut();
        text.sections[1].value = format!("{:.1}", fr.fps_last);
    }
}

//struct Scoreboard {
//    score: usize,
//}
