//#![windows_subsystem = "windows"] // Remove comment to turn off console log output

use std::{ops::Mul, time::Duration};

use bevy::{
    core::FixedTimestep,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, //sprite::collide_aabb::{collide, Collision},
    prelude::*,
    //    math::Vec3,
};
use bevy_render::camera::{DepthCalculation, ScalingMode, WindowOrigin};
//use bevy_rng::*;
use bevy_kira_audio::{Audio, AudioPlugin};

//use bevy_window::*;
//use bevy_winit::*;
mod audio_helper;
mod math;

// TODO: Shooter not shooting straight.
// TODO: Cooler asset loader: https://www.nikl.me/blog/2021/asset-handling-in-bevy-apps/#:~:text=Most%20games%20have%20some%20sort%20of%20loading%20screen,later%20states%20can%20use%20them%20through%20the%20ECS.
// TODO: Inspector:  https://bevy-cheatbook.github.io/setup/bevy-tools.html

// Terminology differences from UNITY to BEVY:

// BEVY     UNITY
// Bundle = Prefab
// System = Behavior
// Component = Component
// Entity = Entity
// Spawn = Instantiate
// Despawn = Destroy
// Resource = Singleton

const TIME_STEP: f32 = 1.0 / 60.0;
const PROJECT: &'static str = "AST4!";
const WIDTH: f32 = 800.0f32;
const HEIGHT: f32 = 600.0f32;
const FREE_USER_AT: u32 = 10000;

fn from_now(t: &Time, delta_sec: f64) -> FutureTime {
    return FutureTime::from_now(t, delta_sec);
}

#[derive(Clone, Copy)]
struct FutureTime {
    seconds_since_startup: f64,
}

impl FutureTime {
    fn from_now(t: &Time, sec: f64) -> FutureTime {
        let now = t.seconds_since_startup();

        let ft = FutureTime {
            seconds_since_startup: now + sec,
        };
        return ft;
    }

    fn is_after(&self, t: &Time) -> bool {
        self.seconds_since_startup < t.seconds_since_startup()
    }
}

#[derive(Component)]
struct GameOverComponent;

#[derive(Component)]
struct Wrapped2dComponent;

#[derive(Component)]
struct FrameRateComponent;

#[derive(Component)]
struct MuzzleComponent;

#[derive(Component, Default)]
struct PlayerComponent {
    pub thrust: f32,
    // TODO: pub player_index: u8, // Or 1, for 2 players
    pub friction: f32,
    pub last_hyperspace_time: f64,
}

enum BulletSource {
    Player,
    Alien,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum State {
    Over,
    Playing,
}

impl Default for State {
    fn default() -> Self {
        State::Over
    }
}

#[derive(Default, Clone)]
struct GameManagerResource {
    state: State,
    score: u32,
    lives: u32,
    next_free_life_score: u32,
}

impl GameManagerResource {
    fn player_killed(
        &mut self,
        mut commands: Commands,
        player: &PlayerComponent,
        sceneController: &mut ResMut<SceneControllerResource>,
        textures_resource: Res<TexturesResource>,
        time: &Res<Time>,
    ) {
        if self.lives < 1 {
            self.state = State::Over;
            sceneController.game_over(time, player);
        } else {
            sceneController.respawn_player(
                commands,
                textures_resource,
                FutureTime::from_now(time, 0.5f64),
            );
        }
    }
}

impl SceneControllerResource {
    fn game_over(&mut self, time: &Res<Time>, player: &PlayerComponent) {
        // todo: stop all sounds.

        self.level = 0;

        //show_game_over( true);
        //show_instructions( true);

        self.disable_start_button_until_time = Some(from_now(time, 1.5f64));
    }

    fn respawn_player(
        &mut self,
        mut commands: Commands,
        textures_resource: Res<TexturesResource>,
        ft: FutureTime,
    ) {
        // This is where we shoot from on player.
        let muzzle_id = commands
            .spawn()
            .insert(Transform {
                translation: Vec3::new(0f32, 12.5f32, 0f32),
                ..Default::default()
            })
            .insert(GlobalTransform {
                ..Default::default()
            })
            .insert(MuzzleComponent {})
            .id();

        let player_id = commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: textures_resource.texture_atlas_handle.clone(), // TODO: How to avoid clone
                sprite: TextureAtlasSprite::new(textures_resource.player_index),
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
                last_hyperspace_time: 0f64,
            })
            .insert(Wrapped2dComponent)
            .insert(RotatorComponent {
                snap_angle: None,
                angle_increment: (std::f32::consts::PI / 16.0f32),
                rotate_speed: 4.0f32,
            })
            .insert(VelocityComponent {
                v: Vec3::new(0f32, 0f32, 0f32),
                max_speed: 300.0f32,
            })
            .insert(ShooterComponent {
                max_bullets: 4,
                bullet_speed: 400.0f32,
            })
            .id();

        commands.entity(player_id).push_children(&[muzzle_id]);
    }

    fn start_game(
        &mut self,
        mut commands: Commands,
        textures_resource: Res<TexturesResource>,
        mut game_manager: ResMut<GameManagerResource>,
        time: &Res<Time>,
    ) {
        self.level = 0;
        game_manager.next_free_life_score = FREE_USER_AT;

        //self.show_game_over(false);
        //self.show_instructions(false);
        self.clear_bullets();

        self.clear_asteroids();
        self.clear_aliens();
        self.start_level(time);
        self.respawn_player(
            commands,
            textures_resource,
            FutureTime::from_now(time, 0.5f64),
        );
    }

    fn start_level(&mut self, time: &Res<Time>) {
        self.level += 1;
        self.jaw_interval_seconds = Duration::from_secs_f32(0.9f32);
        self.jaws_alternate = true;
        self.next_jaws_sound_time = Some(FutureTime::from_now(time, 0.0f64));
        self.add_asteroids(2 + self.level); // 3.0 + Mathf.Log( (float) Level)));
        self.last_asteroid_killed_at = Some(FutureTime::from_now(time, 15.0f64))
    }

    fn can_start_game(&mut self, time: &Res<Time>) -> bool {
        match self.disable_start_button_until_time {
            Some(dsbut) => dsbut.is_after(time),
            None => true,
        }
    }

    fn clear_bullets(&mut self) {
        // TODO
    }
    fn clear_asteroids(&mut self) {
        // TODO
    }
    fn clear_aliens(&mut self) {}
    fn add_asteroids(&mut self, count: u32) {}
}

#[derive(Component)]
struct BulletComponent {
    source: BulletSource,
}

#[derive(Component)]
struct AutoDestroyComponent {
    when: FutureTime,
    enabled: bool,
}

// TODO: will we really use this on the alien? Maybe max_bullets should just be on the shooter.
// Any entity that can shoot a bullet should have one of these to manage their bullets.
#[derive(Component)]
struct ShooterComponent {
    pub max_bullets: usize,
    pub bullet_speed: f32,
}

#[derive(Component, Default)]
struct VelocityComponent {
    pub v: Vec3,
    pub max_speed: f32, // magnitude.
}

impl VelocityComponent {
    // TODO: Should time be part of thrust?
    pub fn apply_thrust(&mut self, thrust: f32, direction: &Quat) {
        let (_, _, angle_radians) = direction.to_euler(EulerRot::XYZ);
        let thrust_vector =
            thrust * Vec3::new(-f32::sin(angle_radians), f32::cos(angle_radians), 0f32);
        self.v += thrust_vector; // * time.delta_seconds();
        self.v = self.v.clamp_length_max(self.max_speed);
    }

    pub fn apply_friction(&mut self, friction: f32) {
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
            if let Some(snap_angle) = self.snap_angle {
                transform.rotation = Quat::from_rotation_z(snap_angle);
                self.snap_angle = None;
            }
        }
    }
}

struct FrameRateResource {
    pub display_frame_rate: bool,
    pub debug_sinusoidal_frame_rate: bool,
    delta_time: f64,
    fps_last: f64,
}

#[derive(Default, Clone)]
struct GameStateResource {
    level: u32,
    next_free_life_score: u64,
}

// todo: not sure why this isn't part of gamestate.
#[derive(Default, Clone)]
struct SceneControllerResource {
    level: u32,
    next_jaws_sound_time: Option<FutureTime>,
    jaw_interval_seconds: Duration,
    jaws_alternate: bool,
    last_asteroid_killed_at: Option<FutureTime>,
    disable_start_button_until_time: Option<FutureTime>,
}

#[derive(Default, Clone)]
struct TexturesResource {
    texture_atlas_handle: Handle<TextureAtlas>,
    player_index: usize,
    bullet_index: usize,
}

fn seed_rng() {
    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let in_ms = since_the_epoch.as_secs();
    fastrand::seed(in_ms as u64);
}
fn main() {
    seed_rng();

    let mut new_app = App::new();

    new_app.
    insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .insert_resource(WindowDescriptor {
            title: PROJECT.to_string(),
            width: WIDTH,
            height: HEIGHT,
            vsync: true,
            cursor_visible: false,
            decorations: false, // Hide the white flashing window at atartup
            // mode: bevy_window::WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        //.add_plugin( RngPlugin)

        //.add_plugin( WindowPlugin { ..Default::default()})q
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(AudioPlugin)
        //  .insert_resource(Scoreboard { score: 0 })
        //.insert_resource(GameEntities {
        //    game_over_entity: None,
        //})
        .insert_resource( SceneControllerResource {
            ..Default::default()
        })
        .insert_resource( GameManagerResource {
            state: State::Over,
            ..Default::default()

        })
        .insert_resource(SceneControllerResource {
            jaw_interval_seconds: Duration::from_secs_f32(0.9f32),
            jaws_alternate: false,
            next_jaws_sound_time: None,
            ..Default::default()
        })
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
        .insert_resource( TexturesResource {
            ..Default::default()
        })
        .add_startup_system(setup)
        .add_system(audio_helper::check_audio_loading)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(game_over_system)
                .with_system(player_system)
                .with_system(wrapped_2d)
                .with_system(auto_destroy_system)
                .with_system(velocity_system)
                .with_system(scene_system)
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

fn setup<'a>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures_resource: ResMut<TexturesResource>,
    scene_controller_resource: ResMut<SceneControllerResource>,
    game_manager: ResMut<GameManagerResource>,
) {
    audio_helper::prepare_audio(&mut commands, asset_server.as_ref());

    // hot reloading of assets.
    asset_server.watch_for_changes().unwrap();

    // Add the game's entities to our world

    // cameras
    commands.spawn_bundle(new_camera_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let texture_handle = asset_server.load("textures/Atlas.png");
    let mut texture_atlas = TextureAtlas::new_empty(texture_handle, Vec2::new(128.0, 128.0));
    textures_resource.player_index = TextureAtlas::add_texture(
        &mut texture_atlas,
        bevy::sprite::Rect {
            min: Vec2::new(2.0, 2.0),
            max: Vec2::new(27.0, 32.0),
        },
    );
    textures_resource.bullet_index = TextureAtlas::add_texture(
        &mut texture_atlas,
        bevy::sprite::Rect {
            min: Vec2::new(9.0, 40.0),
            max: Vec2::new(15.0, 46.0), // TODO
        },
    );
    //let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(25.0,25.0),1,1);

    // Save for later.
    let ttad = texture_atlases.add(texture_atlas);
    textures_resource.texture_atlas_handle = ttad.clone();

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

    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "Game Over\n\n\nPress space to start\nCTRL to shoot\nL/R arrow keys to rotate\nup arrow for thrust\nenter for hyperspace".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/Hyperspace.otf"),
                    font_size: 30.0,
                    color: Color::rgb(1.0, 1.0, 1.0),
                },
            }],
            alignment: TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            },
        },
        style: Style {
            align_self: AlignSelf::Center,
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Percent(30.0f32),
                left: Val::Percent(30.0f32),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(GameOverComponent)
    .insert(Visibility { is_visible: false });
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
    let cam_rect_right: f32 = WIDTH;
    let cam_rect_left: f32 = 0.0f32;
    let cam_rect_top = HEIGHT;
    let cam_rect_bottom = 0.0f32;

    for (_, mut transform) in query.iter_mut() {
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
}

// TODO: Make this work in debugger by actually count time.
fn auto_destroy_system(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &mut AutoDestroyComponent)>,
) {
    let now = time;
    let mut iter = query.iter();
    for (ee, ad) in &mut iter {
        if ad.enabled {
            if ad.when.is_after(&now) {
                println!("happened too soon.");
                commands.entity(ee).despawn_recursive();
            }
        }
    }
}

fn player_system(
    mut commands: Commands,
    game_manager: Res<GameManagerResource>,
    keyboard_input: Res<Input<KeyCode>>,
    textures: Res<TexturesResource>,
    time: Res<Time>,
    audio: Res<Audio>,
    audio_state: Res<audio_helper::AudioState>,
    mut query: Query<(
        &mut PlayerComponent,
        &mut RotatorComponent,
        &mut Transform,
        &mut VelocityComponent,
        &mut ShooterComponent,
        //        &mut Rng,
    )>,
    bullet_query: Query<&BulletComponent>,
    muzzle_query: Query<(&MuzzleComponent, &GlobalTransform)>,
) {
    // println!("Player");
    if query.is_empty() || game_manager.state == State::Over {
        return; // No player.
    }
    let (mut player, mut rotator, mut transform, mut velocity, shooter) = query.single_mut();

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
        // Can't stop looped sounds individually, so one per track.
        audio_helper::start_looped_sound(
            &audio_helper::Tracks::Thrust,
            &audio_helper::Sounds::Thrust,
            &audio,
            &audio_state,
        );

        // Too much trouble to implement rigid body like in Unity, so wrote my own.
        // Assume no friction while accelerating.
        velocity.apply_thrust(player.thrust, &transform.rotation);

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
        velocity.apply_friction(player.friction);

        audio_helper::stop_looped_sound(&audio_helper::Tracks::Thrust, &audio, &audio_state);

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

    if keyboard_input.just_pressed(KeyCode::LControl)
        || keyboard_input.just_pressed(KeyCode::RControl)
    {
        if bullet_query.iter().count() < shooter.max_bullets {
            let (_, muzzle_transform) = muzzle_query.single();

            fire_bullet_from_player(
                textures,
                transform.as_ref(),
                &mut commands,
                &shooter,
                &muzzle_transform,
                audio,
                audio_state,
                &time,
            );
        }
    }

    if time.seconds_since_startup() - player.last_hyperspace_time > 1.0f64 {
        // TBD: make a constant.
        if keyboard_input.pressed(KeyCode::Return) {
            transform.translation = make_random_pos(); // Not safe on purpose
            player.last_hyperspace_time = time.seconds_since_startup();
        }
    }
}

fn velocity_system(time: Res<Time>, mut query: Query<(&mut Transform, &VelocityComponent)>) {
    for (mut transform, velocity) in query.iter_mut() {
        //  Move forward in direction of velocity.
        transform.as_mut().translation += velocity.v * time.delta_seconds();
    }
}

// TBD: If this were inside
fn fire_bullet_from_player(
    textures: Res<TexturesResource>,
    player_transform: &Transform,
    commands: &mut Commands,
    shooter: &ShooterComponent,
    muzzle_transform: &GlobalTransform,
    audio: Res<Audio>,
    audio_state: Res<audio_helper::AudioState>,
    time: &Res<Time>,
) {
    audio_helper::play_single_sound(
        &audio_helper::Tracks::Game,
        &audio_helper::Sounds::Fire,
        &audio,
        &audio_state,
    );

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: textures.texture_atlas_handle.clone(), // TODO: is this really good?
            sprite: TextureAtlasSprite::new(textures.bullet_index),
            transform: Transform {
                scale: Vec3::splat(1.0),
                translation: muzzle_transform.translation.clone(), // TODO: This needs to be muzzle-child position.
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(BulletComponent {
            source: BulletSource::Player,
        })
        .insert(VelocityComponent {
            v: calc_player_normalized_pointing_dir(player_transform).mul(shooter.bullet_speed),
            max_speed: 5000.0f32, // TBD: Speed should be a struct
        })
        .insert(Wrapped2dComponent {})
        .insert(AutoDestroyComponent {
            enabled: true,
            when: FutureTime::from_now(time, 1.0f64),
        });

    //TODO:

    //newBullet.transform.position = MuzzleChild.transform.position;
    //newBullet.transform.rotation = this.transform.rotation;
    //newBullet.GetComponent<Rigidbody2D>().AddRelativeForce(Vector2.up*1.4f, ForceMode2D.Impulse);
    //newBullet.gameObject.SetActive(true);

    // GameManager.Instance.PlayClip(ShootSound);
    // Destroy(newBullet.gameObject, 1.4f);
}

fn calc_player_normalized_pointing_dir(p: &Transform) -> Vec3 {
    let (_, _, angle_radians) = p.rotation.to_euler(EulerRot::XYZ);
    let dir_vector = Vec3::new(-f32::sin(angle_radians), f32::cos(angle_radians), 0f32);

    return dir_vector;
}

fn make_random_pos() -> Vec3 {
    let x = fastrand::f32();
    let y = fastrand::f32();
    Vec3::new(x * WIDTH, y * HEIGHT, 0f32)
}

// Show or hide instructions based on game state.
fn game_over_system(
    game_manager: Res<GameManagerResource>,
    mut query: Query<(&mut Visibility, &GameOverComponent)>,
) {
    for (mut vis, _) in query.iter_mut() {
        vis.is_visible = game_manager.state == State::Over;
    }
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

    if fr.display_frame_rate {
        let (mut text, _) = query.single_mut();
        text.sections[1].value = format!("{:.1}", fr.fps_last);
    }
}

fn scene_system(
    mut commands: Commands,
    mut scene_controller: ResMut<SceneControllerResource>,
    mut game_manager: ResMut<GameManagerResource>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    audio: Res<Audio>,
    audio_state: Res<audio_helper::AudioState>,
    textures_resource: Res<TexturesResource>,
) {
    match game_manager.state {
        State::Playing => {
            update_ambience_sound(&time, scene_controller, &audio, &audio_state);
        }
        State::Over => {
            // TODO: Turn off jaws sounds.
            audio_helper::stop_looped_sound(&audio_helper::Tracks::Ambience, &audio, &audio_state);

            if keyboard_input.pressed(KeyCode::Space) {
                // Try to prevent game starting right after previous if you keep firing.
                if scene_controller.can_start_game(&time) {
                    game_manager.lives = 4;
                    game_manager.score = 0;
                    game_manager.state = State::Playing;
                    scene_controller.start_game(commands, textures_resource, game_manager, &time);
                }
            }
        }
    }

    // TODO: Make impl method on the scene controller.
    fn update_ambience_sound(
        time: &Res<Time>,
        mut scene_controller: ResMut<SceneControllerResource>,
        audio: &Res<Audio>,
        audio_state: &Res<audio_helper::AudioState>,
    ) {
        // TODO: Lame that Time doesn't have a mehthod for this.

        if scene_controller
            .next_jaws_sound_time
            .unwrap()
            .is_after(time)
        // if in level
        {
            if scene_controller.jaw_interval_seconds.as_secs_f32() > 0.1800f32 {
                scene_controller.jaw_interval_seconds = Duration::from_secs_f32(
                    scene_controller.jaw_interval_seconds.as_secs_f32() - 0.005f32,
                );
            }
            scene_controller.next_jaws_sound_time = Some(FutureTime::from_now(
                time,
                scene_controller.jaw_interval_seconds.as_secs_f64(),
            ));
            if scene_controller.jaws_alternate {
                audio_helper::play_single_sound(
                    &audio_helper::Tracks::Ambience,
                    &audio_helper::Sounds::Beat1,
                    audio,
                    audio_state,
                );
            } else {
                audio_helper::play_single_sound(
                    &audio_helper::Tracks::Ambience,
                    &audio_helper::Sounds::Beat2,
                    audio,
                    audio_state,
                );
            }
            scene_controller.jaws_alternate = !scene_controller.jaws_alternate;
        }
    }
}

//struct Scoreboard {
//    score: usize,
//}
