//#![windows_subsystem = "windows"] // Remove comment to turn off console log output

use std::ops::Mul;

use bevy::{
    core::FixedTimestep,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, //sprite::collide_aabb::{collide, Collision},
    ecs::system::SystemParam,
    //    math::Vec3,
    prelude::*,
};
//use bevy::tasks::ComputeTaskPool;
use bevy_kira_audio::{Audio, AudioPlugin};
use bevy_prototype_lyon::prelude::*;
use bevy_render::camera::{DepthCalculation, ScalingMode, WindowOrigin};

//use bevy_window::*;
//use bevy_winit::*;
mod audio_helper;
mod math;

// TODO: Shooter not shooting straight.
// TODO: Cooler asset loader: https://www.nikl.me/blog/2021/asset-handling-in-bevy-apps/#:~:text=Most%20games%20have%20some%20sort%20of%20loading%20screen,later%20states%20can%20use%20them%20through%20the%20ECS.
// TODO: Inspector:  https://bevy-cheatbook.github.io/setup/bevy-tools.html
// TODO: Investigate: MrGVSV/bevy_proto

// Terminology differences from UNITY to BEVY:

// BEVY     UNITY
// Bundle = Prefab
// System = Behavior
// Component = Component
// Entity = Entity
// Spawn = Instantiate
// Despawn = Destroy
// Resource = Singleton

const DEBUG: bool = false;
const TIME_STEP: f32 = 1.0 / 60.0;
const PROJECT: &str = "AST4!";
const WIDTH: f32 = 800.0f32;
const HEIGHT: f32 = 600.0f32;
const FREE_USER_AT: u32 = 10000;
static DELETE_CLEANUP_STAGE: &str = "delete_cleanup_stage";

#[derive(Default, Clone, Copy)]
struct FutureTime {
    seconds_since_startup_to_auto_destroy: f64,
}

struct AsteroidCollisionEvent {
    asteroid: Entity,
    hit_by: Entity,
}

struct PlayerCollisionEvent {
    player: Entity,
    hit_by: Entity,
}

impl FutureTime {
    fn from_now(t: &Time, sec: f64) -> FutureTime {
        assert!(sec >= 0f64);
        let now = t.seconds_since_startup();
        let future = now + sec;

        FutureTime {
            seconds_since_startup_to_auto_destroy: future,
        }
    }

    fn is_expired(&self, t: &Time) -> bool {
        let now = t.seconds_since_startup();
        let future = self.seconds_since_startup_to_auto_destroy;
        now > future
    }
}

#[derive(Component)]
struct Particle {
    position: Vec3,
    velocity: Vec3,
    lifetime: f32,
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    //compute_task_pool: Res<ComputeTaskPool>,
    mut particles: Query<(&mut Particle, Entity, &mut Transform)>,
) {
    let dt = time.delta_seconds_f64() as f32;
    //particles.par_for_each_mut(&compute_task_pool, 32, move |(mut particle,entity)| {
    particles.for_each_mut(move |(mut particle, entity, mut transform)| {
        let velocity = particle.velocity * dt;
        particle.position += velocity;
        particle.lifetime -= dt;
        transform.translation = particle.position;

        if particle.lifetime < 0.0f32 {
            commands.entity(entity).despawn_recursive();
        }
    });
}

fn create_particles(
    commands: &mut Commands,
    textures_resource: &Res<TexturesResource>,
    count: u16,
    pos: Vec3,
) {
    for _ in 0..count {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: textures_resource.texture_atlas_handle.clone(), // TODO: How to avoid clone
                sprite: TextureAtlasSprite::new(textures_resource.explosion_particle_index),
                transform: Transform {
                    scale: Vec3::splat(1.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Particle {
                position: pos,
                velocity: make_random_velocity(100.0f32),

                lifetime: random_range(1.0f32, 1.0f32),
            });
    }
    //}
}

fn random_range(min: f32, max: f32) -> f32 {
    let random = ::fastrand::f32();
    let range = max - min;
    let adjustment = range * random;
    min + adjustment
}

#[derive(Component)]
struct GameOverComponent;

#[derive(Component)]
struct DebugComponent;

#[derive(Component)]
struct Wrapped2dComponent;

#[derive(Component)]
struct FrameRateComponent;

#[derive(Component, Default)]
struct DeleteCleanupComponent {
    delete_after_frame: bool,
    auto_destroy_enabled: bool,
    auto_destroy_when: FutureTime,
}

#[derive(Component)]
struct AsteroidComponent {
    size: AsteroidSize,
    hit_radius: f32,
}

#[derive(Component)]
struct AlienComponent {
    size: AlienSize,
}

#[derive(Component)]
struct MuzzleComponent;

#[derive(Component, Default)]
struct PlayerComponent {
    pub thrust: f32,
    // TODO: pub player_index: u8, // Or 1, for 2 players
    pub friction: f32,
    pub last_hyperspace_time: f64,
    pub snap_angle: Option<f32>,
    pub rotate_speed: f32,    //= 150f;
    pub angle_increment: f32, // = 5.0f;
}

#[derive(Component)]
struct ScoreComponent;

#[derive(Component)]
struct LivesComponent;

enum BulletSource {
    Player,
    Alien,
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
enum AsteroidSize {
    Large,
    Medium,
    Small,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AlienSize {
    Large,
    Small,
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

// Combine with scene controller.
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
        scene_controller: &mut ResMut<SceneControllerResource>,
        time: &Res<Time>,
    ) {
        if self.lives > 0 {
            self.lives -= 1;
        }
        if self.lives < 1 {
            self.state = State::Over;
            scene_controller.game_over();
        } else {
            scene_controller.respawn_player_later(FutureTime::from_now(time, 0.5f64));
        }
    }
}

impl SceneControllerResource {
    fn game_over(&mut self) {
        // todo: stop all sounds.

        self.level = 0;

        //show_game_over( true);
        //show_instructions( true);
    }

    // We need to respawn player "later", so there:
    //  1) aren't two players in one frame (dead and spawned)
    //  2) Multiple hits with same asteroid after respawn
    fn respawn_player_later(&mut self, ft: FutureTime) {
        self.player_spawn_when = Some(ft);
    }

    fn respawn_player(
        &mut self,
        commands: &mut Commands,
        textures_resource: &Res<TexturesResource>,
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
                    translation: Vec3::new(WIDTH / 2.0f32, HEIGHT / 2.0f32, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(PlayerComponent {
                thrust: 2.0f32,
                friction: 0.98f32,
                last_hyperspace_time: 0f64,
                snap_angle: None,
                angle_increment: (std::f32::consts::PI / 16.0f32),
                rotate_speed: 4.0f32,
            })
            .insert(Wrapped2dComponent)
            .insert(VelocityComponent {
                v: Vec3::new(0f32, 0f32, 0f32),
                max_speed: 300.0f32,
                spin: 0.0f32,
            })
            .insert(ShooterComponent {
                max_bullets: 4,
                bullet_speed: 400.0f32,
            })
            .insert(DeleteCleanupComponent {
                delete_after_frame: false,
                auto_destroy_enabled: false,
                ..Default::default()
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
        self.start_level(&mut commands, &textures_resource, time);
        self.respawn_player_later(FutureTime::from_now(time, 0.5f64));
    }

    fn start_level(
        &mut self,
        commands: &mut Commands,
        textures_resource: &Res<TexturesResource>,
        time: &Res<Time>,
    ) {
        self.level += 1;
        self.jaw_interval_seconds = 0.9f64;
        self.jaws_alternate = true;
        self.next_jaws_sound_time = Some(FutureTime::from_now(time, 1.0f64));
        self.add_asteroids(2 + self.level, commands, textures_resource); // 3.0 + Mathf.Log( (float) Level)));
        self.last_asteroid_killed_at = Some(FutureTime::from_now(time, 15.0f64))
    }

    fn clear_bullets(&mut self) {
        // TODO
    }
    fn clear_asteroids(&mut self) {
        // TODO
    }
    fn clear_aliens(&mut self) {}

    fn add_asteroids(
        &mut self,
        count: u32,
        commands: &mut Commands,
        textures_resource: &Res<TexturesResource>,
    ) {
        for _ in 0..count - 1 {
            let pos = SceneControllerResource::make_safe_asteroid_pos();
            SceneControllerResource::add_asteroid_with_size_at(
                commands,
                textures_resource,
                &AsteroidSize::Large,
                pos,
            )
        }
    }

    // TODO: Load a mirrored verion of asteroid, so not all the same.
    fn add_asteroid_with_size_at(
        commands: &mut Commands,
        textures_resource: &Res<TexturesResource>,
        size: &AsteroidSize,
        p: Vec3,
    ) {
        // TODO: Maybe all this should be in a map.
        let hit_radius = match size {
            AsteroidSize::Large => textures_resource.asteroid_large_hit_radius,
            AsteroidSize::Medium => textures_resource.asteroid_medium_hit_radius,
            AsteroidSize::Small => textures_resource.asteroid_small_hit_radius,
        };
        let index = match size {
            AsteroidSize::Large => textures_resource.asteroid_large_index,
            AsteroidSize::Medium => textures_resource.asteroid_medium_index,
            AsteroidSize::Small => textures_resource.asteroid_small_index,
        };

        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: textures_resource.texture_atlas_handle.clone(), // TODO: How to avoid clone
                sprite: TextureAtlasSprite::new(index),
                transform: Transform {
                    scale: Vec3::splat(1.0),
                    translation: p,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(AsteroidComponent {
                size: *size,
                hit_radius,
            })
            .insert(Wrapped2dComponent)
            .insert(VelocityComponent {
                v: make_random_velocity(300f32),
                max_speed: 300.0f32,
                spin: 1.0f32,
            })
            .insert(DeleteCleanupComponent {
                delete_after_frame: false,
                auto_destroy_enabled: false,
                ..Default::default()
            });
    }

    // TODO
    fn make_safe_asteroid_pos() -> Vec3 {
        // Todo: Implement

        //if (_player1 != null)
        //{
        //    var playerPos = _player1.transform.position;
        //    for (int ii = 1; ii < 1000; ii++)
        //    {
        //        var astPos = MakeRandomPos();
        //        if (Vector3.Distance(astPos, playerPos) > 2.0)
        //        {
        //            return astPos;
        //        }
        //    }
        //}
        make_random_pos()
    }
}

#[derive(Component)]
struct BulletComponent {
    source: BulletSource,
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
    pub spin: f32,      // Spin/sec in radians.
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

fn rotate_by_angle(t: &mut Transform, angle_to_rotate: f32) -> f32 {
    let (_, _, cur_angle) = t.rotation.to_euler(EulerRot::XYZ);
    let target_angle = cur_angle + angle_to_rotate;

    // create the change in rotation around the Z axis (pointing through the 2d plane of the screen)
    let rotation_delta = Quat::from_rotation_z(angle_to_rotate);
    // update the ship rotation with our rotation delta
    t.rotation *= rotation_delta;
    target_angle
}

struct FrameRateResource {
    pub display_frame_rate: bool,
    pub debug_sinusoidal_frame_rate: bool,
    delta_time: f64,
    fps_last: f64,
}

// todo: not sure why this isn't part of gamestate.
#[derive(Default, Clone)]
struct SceneControllerResource {
    level: u32,
    next_jaws_sound_time: Option<FutureTime>,
    jaw_interval_seconds: f64,
    jaws_alternate: bool,
    last_asteroid_killed_at: Option<FutureTime>,
    next_free_life_score: u32,

    player_spawn_when: Option<FutureTime>,
}

#[derive(Default, Clone)]
struct TexturesResource {
    texture_atlas_handle: Handle<TextureAtlas>,
    player_index: usize,
    bullet_index: usize,
    asteroid_large_index: usize,
    asteroid_medium_index: usize,
    asteroid_small_index: usize,
    explosion_particle_index: usize,
    ship_particle_index: usize,

    asteroid_large_hit_radius: f32,
    asteroid_medium_hit_radius: f32,
    asteroid_small_hit_radius: f32,
}

fn seed_rng(t: &Res<Time>) {
    let in_ms = t.seconds_since_startup();
    println!("seed: {}", in_ms);
    fastrand::seed(in_ms as u64);
}

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() {
    println!(
        "VERSION: {}  GIT_VERSION: {}",
        built_info::PKG_VERSION,
        built_info::GIT_VERSION.unwrap_or("?????")
    );

    let mut new_app = App::new();

    new_app
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
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
        //.add_plugin(LogDiagnosticsPlugin::default())  // TODO - put behind a flag
        //.add_plugin(FrameTimeDiagnosticsPlugin::default()) // TODO - put behind a flag
        .add_plugin(AudioPlugin)
        .add_plugin(ShapePlugin)
        .add_event::<AsteroidCollisionEvent>()
        .add_event::<PlayerCollisionEvent>()
        .insert_resource(SceneControllerResource {
            ..Default::default()
        })
        .insert_resource(GameManagerResource {
            state: State::Over,
            ..Default::default()
        })
        .insert_resource(SceneControllerResource {
            level: 0,
            jaw_interval_seconds: 0.9f64,
            jaws_alternate: false,
            next_jaws_sound_time: None,
            next_free_life_score: FREE_USER_AT,
            ..Default::default()
        })
        .insert_resource(FrameRateResource {
            delta_time: 0f64,
            display_frame_rate: true,
            debug_sinusoidal_frame_rate: false,
            fps_last: 0f64,
        })
        .insert_resource(TexturesResource {
            ..Default::default()
        })
        .add_startup_system(setup)
        .add_system(audio_helper::check_audio_loading)
        .add_stage_after(
            CoreStage::Update,
            DELETE_CLEANUP_STAGE,
            SystemStage::single_threaded(),
        )
        .add_system_to_stage(DELETE_CLEANUP_STAGE, delete_cleanup_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(game_over_system)
                .with_system(player_system)
                .with_system(wrapped_2d_system)
                .with_system(velocity_system)
                .with_system(scene_system)
                .with_system(score_system)
                .with_system(lives_system)
                .with_system(collision_system)
                .with_system(asteroid_collision_system)
                .with_system(player_collision_system)
                .with_system(level_system)
                .with_system(player_spawn_system)
                .with_system(update_particles)
                // TODO: Figure out.
                //.with_run_criteria(IntoRunCriteria::into( if DEBUG  {bevy::ecs::schedule::ShouldRun::Yes} else { bevy::ecs::schedule::ShouldRun::No}) )
                .with_system(debug_system),
        )
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
    camera
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures_resource: ResMut<TexturesResource>,
    time: Res<Time>,
    //scene_controller_resource: ResMut<SceneControllerResource>,
    //game_manager: ResMut<GameManagerResource>,
) {
    seed_rng(&time);

    audio_helper::prepare_audio(&mut commands, asset_server.as_ref());

    // hot reloading of assets.
    //asset_server.watch_for_changes().unwrap();

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
            max: Vec2::new(15.0, 46.0),
        },
    );

    let large_asteroid_rect = bevy::sprite::Rect {
        min: Vec2::new(82.0, 4.0),
        max: Vec2::new(126.0, 42.0),
    };
    textures_resource.asteroid_large_index =
        TextureAtlas::add_texture(&mut texture_atlas, large_asteroid_rect);
    textures_resource.asteroid_large_hit_radius = large_asteroid_rect.width() / 2.0f32;

    let medium_asteroid_rect = bevy::sprite::Rect {
        min: Vec2::new(47.0, 4.0),
        max: Vec2::new(71.0, 24.0),
    };
    textures_resource.asteroid_medium_index =
        TextureAtlas::add_texture(&mut texture_atlas, medium_asteroid_rect);
    textures_resource.asteroid_medium_hit_radius = medium_asteroid_rect.width() / 2.0f32;

    let small_asteroid_rect = bevy::sprite::Rect {
        min: Vec2::new(29.0, 2.0),
        max: Vec2::new(43.0, 15.0),
    };
    textures_resource.asteroid_small_index =
        TextureAtlas::add_texture(&mut texture_atlas, small_asteroid_rect);
    textures_resource.asteroid_small_hit_radius = small_asteroid_rect.width() / 2.0f32;

    textures_resource.explosion_particle_index = TextureAtlas::add_texture(
        &mut texture_atlas,
        bevy::sprite::Rect {
            min: Vec2::new(49.0, 65.0),
            max: Vec2::new(52.0, 68.0),
        },
    );

    textures_resource.ship_particle_index = TextureAtlas::add_texture(
        &mut texture_atlas,
        bevy::sprite::Rect {
            min: Vec2::new(40.0, 34.0),
            max: Vec2::new(47.0, 49.0),
        },
    );

    //let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(25.0,25.0),1,1);

    // Save for later.
    let ttad = texture_atlases.add(texture_atlas);
    textures_resource.texture_atlas_handle = ttad;

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
                    TextSection {
                        value: " VERSION: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 10.0,
                            color: Color::rgb(0.5, 0.5, 1.0),
                        },
                    },
                    TextSection {
                        value: built_info::PKG_VERSION.to_string()
                            + "."
                            + built_info::GIT_VERSION.unwrap_or("?????"),
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
                    left: Val::Px(650.0),
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
            align_self: AlignSelf::Auto,
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Percent(30.0f32),
                left: Val::Percent(31.0f32), // TODO: Why do I have to guess?
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(GameOverComponent)
    .insert(Visibility { is_visible: false });

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "0".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/Hyperspace.otf"),
                        font_size: 30.0,
                        color: Color::rgb(1.0, 1.0, 1.0),
                    },
                }],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Left,
                },
            },
            style: Style {
                align_self: AlignSelf::Auto,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Percent(1.0f32),
                    left: Val::Percent(4.0f32),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ScoreComponent)
        .insert(Visibility { is_visible: true });

    // Lives
    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "^".to_string().repeat(3),
                    style: TextStyle {
                        font: asset_server.load("fonts/Hyperspace.otf"),
                        font_size: 30.0,
                        color: Color::rgb(1.0, 1.0, 1.0),
                    },
                }],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Left,
                },
            },
            style: Style {
                align_self: AlignSelf::Auto,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Percent(2.0f32),
                    left: Val::Percent(16.0f32),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(LivesComponent)
        .insert(Visibility { is_visible: true });
}

fn wrapped_2d_system(mut query: Query<(&Wrapped2dComponent, &mut Transform)>) {
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

fn player_system(
    mut commands: Commands,
    game_manager: Res<GameManagerResource>,
    keyboard_input: Res<Input<KeyCode>>,
    textures: Res<TexturesResource>,
    time: Res<Time>,
    audio: Res<Audio>,
    mut audio_state: ResMut<audio_helper::AudioState>,
    mut query: Query<(
        &mut PlayerComponent,
        &mut Transform,
        &mut VelocityComponent,
        &mut ShooterComponent,
    )>,
    bullet_query: Query<&BulletComponent>,
    muzzle_query: Query<(&MuzzleComponent, &GlobalTransform)>,
) {
    // println!("Player");
    if query.is_empty() || game_manager.state == State::Over {
        return; // No player.
    }
    let (mut player, mut transform, mut velocity, shooter) = query.single_mut();

    let mut dir = 0.0f32;
    if keyboard_input.pressed(KeyCode::Left) {
        dir += 1.0f32;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        dir += -1.0f32;
    }
    player.rotate_to_angle_with_snap(&mut transform, dir, &time);

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
            &mut audio_state,
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
        */
    } else {
        velocity.apply_friction(player.friction);

        audio_helper::stop_looped_sound(&audio_helper::Tracks::Thrust, &audio, &audio_state);

        /* TODO
        if (_exhaustParticleSystem.isPlaying)
        {
            _exhaustParticleSystem.Stop();
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
                muzzle_transform,
                audio,
                &audio_state,
                &time,
            );
        }
    }

    // TBD: make a constant.
    if keyboard_input.pressed(KeyCode::Return) {
        if time.seconds_since_startup() - player.last_hyperspace_time > 1.0f64 {
            transform.translation = make_random_pos(); // Not safe on purpose
            player.last_hyperspace_time = time.seconds_since_startup();
        }
    }
}

impl PlayerComponent {
    pub fn rotate_to_angle_with_snap(
        &mut self,
        transform: &mut Transform,
        horz: f32,
        time: &Res<Time>,
    ) {
        if horz != 0.0f32 {
            let target_angle =
                rotate_by_angle(transform, horz * self.rotate_speed * time.delta_seconds());

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

fn velocity_system(time: Res<Time>, mut query: Query<(&mut Transform, &VelocityComponent)>) {
    for (mut transform, velocity) in query.iter_mut() {
        //  Move forward in direction of velocity.
        transform.as_mut().translation += velocity.v * time.delta_seconds();

        let _ = rotate_by_angle(&mut transform, velocity.spin * time.delta_seconds());
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
    audio_state: &ResMut<audio_helper::AudioState>,
    time: &Res<Time>,
) {
    audio_helper::play_single_sound(
        &audio_helper::Tracks::Game,
        &audio_helper::Sounds::Fire,
        &audio,
        audio_state,
    );

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: textures.texture_atlas_handle.clone(), // TODO: is this really good?
            sprite: TextureAtlasSprite::new(textures.bullet_index),
            transform: Transform {
                scale: Vec3::splat(1.0),
                translation: muzzle_transform.translation,
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
            spin: 0.0f32,
        })
        .insert(Visibility { is_visible: true })
        .insert(Wrapped2dComponent {})
        .insert(DeleteCleanupComponent {
            delete_after_frame: false,
            auto_destroy_enabled: true,
            auto_destroy_when: FutureTime::from_now(time, 1.2f64),
        });
}

fn calc_player_normalized_pointing_dir(p: &Transform) -> Vec3 {
    let (_, _, angle_radians) = p.rotation.to_euler(EulerRot::XYZ);
    Vec3::new(-f32::sin(angle_radians), f32::cos(angle_radians), 0f32)
}

fn make_random_pos() -> Vec3 {
    let x = fastrand::f32();
    let y = fastrand::f32();
    Vec3::new(x * WIDTH, y * HEIGHT, 0f32)
}

// TODO: This makes some very slow speeds and need to fix that.
fn make_random_velocity(max_speed: f32) -> Vec3 {
    let x = fastrand::f32() - 0.5f32;
    let y = fastrand::f32() - 0.5f32;
    let speed = fastrand::f32() * max_speed;
    speed * Vec3::new(x, y, 0f32)
}

// Show or hide instructions based on game state.
fn game_over_system(
    game_manager: Res<GameManagerResource>,
    mut query: Query<(&mut Visibility, &GameOverComponent)>,
) {
    let is_over = game_manager.state == State::Over;
    for (mut vis, _) in query.iter_mut() {
        vis.is_visible = is_over;
    }
}

fn score_system(
    mut game_manager: ResMut<GameManagerResource>,
    mut query: Query<(&mut Visibility, &ScoreComponent, &mut Text)>,
) {
    let is_playing = game_manager.state == State::Playing;
    let (mut vis, _, mut text) = query.single_mut();
    vis.is_visible = is_playing;

    text.sections[0].value = game_manager.score.to_string();

    if game_manager.score > game_manager.next_free_life_score {
        game_manager.next_free_life_score += FREE_USER_AT;
        game_manager.lives += 1;
    }
}

fn lives_system(
    game_manager: Res<GameManagerResource>,
    mut query: Query<(&mut Visibility, &LivesComponent, &mut Text)>,
) {
    let is_playing = game_manager.state == State::Playing;
    let (mut vis, _, mut text) = query.single_mut();
    vis.is_visible = is_playing;

    text.sections[0].value = "^".to_string().repeat(game_manager.lives as usize);
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

// Reduce number of params :https://github.com/bevyengine/bevy/issues/3267
#[derive(SystemParam)]
struct MySystemParam<'w, 's> {
    audio: Res<'w, Audio>,
    audio_state: ResMut<'w, audio_helper::AudioState>,
    textures_resource: Res<'w, TexturesResource>,
    _query: Query<'w, 's, ()>,
}

fn scene_system(
    commands: Commands,
    mut scene_controller: ResMut<SceneControllerResource>,
    mut game_manager: ResMut<GameManagerResource>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    common: MySystemParam,
) {
    match game_manager.state {
        State::Playing => {
            update_ambience_sound(&time, scene_controller, &common.audio, &common.audio_state);
        }
        State::Over => {
            // TODO: Turn off jaws sounds.
            audio_helper::stop_looped_sound(
                &audio_helper::Tracks::Ambience,
                &common.audio,
                &common.audio_state,
            );

            if keyboard_input.pressed(KeyCode::Space) {
                seed_rng(&time); // reseed again.
                game_manager.lives = 4;
                game_manager.score = 0;
                game_manager.state = State::Playing;
                scene_controller.start_game(
                    commands,
                    common.textures_resource,
                    game_manager,
                    &time,
                );
            }
        }
    }

    // TODO: Make impl method on the scene controller.
    fn update_ambience_sound(
        time: &Res<Time>,
        mut scene_controller: ResMut<SceneControllerResource>,
        audio: &Res<Audio>,
        audio_state: &ResMut<audio_helper::AudioState>,
    ) {
        if scene_controller
            .next_jaws_sound_time
            .unwrap()
            .is_expired(time)
        // if in level
        {
            if scene_controller.jaw_interval_seconds > 0.1800f64 {
                scene_controller.jaw_interval_seconds -= 0.005f64
            }
            scene_controller.next_jaws_sound_time = Some(FutureTime::from_now(
                time,
                scene_controller.jaw_interval_seconds,
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

// Not easy-to-use physics like unity, so had to implement my own

fn collision_system(
    mut commands: Commands,
    mut ev_asteroid_collision: EventWriter<AsteroidCollisionEvent>, // TODO: Can I have two type params, like ASTERID?
    mut ev_player_collision: EventWriter<PlayerCollisionEvent>,
    bullet_query: Query<(Entity, &BulletComponent, &Transform)>, // Todo, ColliderComponent in each bullet (etc), would contain info about collision size.
    player_query: Query<(Entity, &PlayerComponent, &Transform)>,
    asteroid_query: Query<(Entity, &AsteroidComponent, &Transform)>,
    alient_query: Query<(Entity, &AlienComponent, &Transform)>,
) {
    // Detect:
    //  player -> bullet
    //  alien -> bullet

    //  player -> asteroid
    //  alien -> asteroid
    //  bullet -> asteroid

    // So if I had a quadtree of Bullets and quadtree of Asteroids.
    use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
    let mut bullet_qt = Quadtree::<i16, usize>::new(10); /*Depth, allows coordinates up to 1024);*/
    assert!(bullet_qt.width() > WIDTH as usize);
    assert!(bullet_qt.height() > HEIGHT as usize);

    let mut asteroid_qt = Quadtree::<i16, usize>::new(10); /*Depth, allows coordinates up to 1024);*/
    assert!(asteroid_qt.width() > WIDTH as usize);
    assert!(asteroid_qt.height() > HEIGHT as usize);

    // Copy iterators into arrays for indexable access
    let bullet_array: Vec<(Entity, &BulletComponent, &Transform)> = bullet_query.iter().collect();
    let player_array: Vec<(Entity, &PlayerComponent, &Transform)> = player_query.iter().collect();
    assert!(player_array.len() <= 1);
    let asteroid_array: Vec<(Entity, &AsteroidComponent, &Transform)> =
        asteroid_query.iter().collect();
    let _alien_array: Vec<(Entity, &AlienComponent, &Transform)> = alient_query.iter().collect();

    fn make_area_around(t: &Transform, radius: f32) -> quadtree_rs::area::Area<i16> {
        let r = radius as i16;
        let one_half_r = r / 2;
        AreaBuilder::default()
            .anchor(Point {
                x: t.translation.x as i16 - one_half_r,
                y: t.translation.y as i16 - one_half_r,
            })
            .dimensions((r, r))
            .build()
            .unwrap()
    }

    for (idx, (_, _, t)) in bullet_array.iter().enumerate() {
        bullet_qt.insert_pt(
            Point {
                x: t.translation.x as i16,
                y: t.translation.y as i16,
            },
            idx,
        );
    }
    for (idx, (_, _, t)) in asteroid_array.iter().enumerate() {
        asteroid_qt.insert_pt(
            Point {
                x: t.translation.x as i16,
                y: t.translation.y as i16,
            },
            idx,
        );
    }

    for (bul_ent, _, bul_trans) in &bullet_array {
        let near_bullet_area = make_area_around(bul_trans, 25.0f32);

        // TODO: Is the first returned asteroid, really the closest?

        if let Some(entry) = asteroid_qt.query(near_bullet_area).next() {
            let idx = entry.value_ref();
            let (ast_ent, ast_comp, ast_trans) = asteroid_array[*idx];

            // So we're close. Let's calculate actual distance based on size.
            let d = bul_trans.translation.distance(ast_trans.translation);
            if d < ast_comp.hit_radius {
                commands.entity(*bul_ent).despawn_recursive();
                ev_asteroid_collision.send(AsteroidCollisionEvent {
                    asteroid: ast_ent,
                    hit_by: *bul_ent,
                });
            }
        }
    }

    for (player_ent, _, play_trans) in &player_array {
        let near_bullet_area = make_area_around(play_trans, 25.0f32); // TOD: hardcode
        for entry in asteroid_qt.query(near_bullet_area) {
            let idx = entry.value_ref();
            let (ast_ent, ast_comp, ast_trans) = asteroid_array[*idx];

            let d = play_trans.translation.distance(ast_trans.translation);

            // TODO: We really want to see whether the triangle of the ship is inside
            //  the radius of the asteroid. Ignore shape, of asteroid.
            if d < ast_comp.hit_radius {
                ev_player_collision.send(PlayerCollisionEvent {
                    player: *player_ent,
                    hit_by: ast_ent,
                });
            }
        }
    }
}

fn replace_asteroid_with(
    commands: &mut Commands,
    textures_resource: &Res<TexturesResource>,
    trans: &Transform,
    count: u8,
    size: AsteroidSize,
) {
    for _ in 0..count {
        SceneControllerResource::add_asteroid_with_size_at(
            commands,
            textures_resource,
            &size,
            trans.translation,
        );
    }

    // TODO: Give momentum from the bullet.
}

fn spawn_asteroid_or_alien_explosion(
    mut commands: &mut Commands,
    textures_resource: &Res<TexturesResource>,
    trans: &Transform,
) {
    create_particles(commands, &textures_resource, 20, trans.translation);
}

// TODO: Can this be part of the regular asteroid_system?
// TODO: Split asteroid into smaller parts, or destroy it. Show explosions.
fn asteroid_collision_system(
    mut commands: Commands,
    mut ev_collision: EventReader<AsteroidCollisionEvent>,
    mut query: Query<(
        Entity,
        &AsteroidComponent,
        &Transform,
        &mut DeleteCleanupComponent,
    )>,
    mut game_manager: ResMut<GameManagerResource>,
    textures_resource: Res<TexturesResource>,
    audio: Res<Audio>,
    audio_state: ResMut<audio_helper::AudioState>,
) {
    for ev in ev_collision.iter() {
        {
            // This is pretty inefficient.
            if let Ok((_, ast, trans, mut dcc)) = query.get_mut(ev.asteroid) {
                if dcc.delete_after_frame {
                    // I guess it's possible this asteroid already deleted
                    //  elsewhere, so don't increase score, etc.
                    continue;
                }
                dcc.delete_after_frame = true;

                audio_helper::play_single_sound(
                    &audio_helper::Tracks::Ambience,
                    &audio_helper::Sounds::BangLarge, // TBD: Write one?
                    &audio,
                    &audio_state,
                );

                match ast.size {
                    AsteroidSize::Large => {
                        game_manager.score += 20;
                        replace_asteroid_with(
                            &mut commands,
                            &textures_resource,
                            trans,
                            2,
                            AsteroidSize::Medium,
                        );
                    }
                    AsteroidSize::Medium => {
                        game_manager.score += 50;
                        replace_asteroid_with(
                            &mut commands,
                            &textures_resource,
                            trans,
                            2,
                            AsteroidSize::Small,
                        );
                    }
                    AsteroidSize::Small => {
                        game_manager.score += 100;
                    }
                }
                spawn_asteroid_or_alien_explosion( &mut commands, &textures_resource, trans);
            }
        }

        /* TODO: Awful:


        if (_asteroids.Count == 0)
        {
            StartLevel();
        }
        else
        {
            _lastAsteroidKilled = Time.time;
        }
        */
    }
}

// IDEAS: Prblem... we have double deletes.
//  System to delete at end frame.
//  Game manager tracks deletions for a specific frame, so you can avoid doing it twice.

// TODO: Lifes,etc.
fn player_collision_system(
    mut ev_collision: EventReader<PlayerCollisionEvent>,
    mut query: Query<(Entity, &mut DeleteCleanupComponent)>,
    mut game_manager: ResMut<GameManagerResource>,
    mut scene_controller: ResMut<SceneControllerResource>,
    time: Res<Time>,
) {
    for ev in ev_collision.iter() {
        // This is pretty inefficient.
        if let Ok((_, mut dcc)) = query.get_mut(ev.player) {
            dcc.delete_after_frame = true;

            // TODO: Create an explosion for player.

            // Delete lifes
            game_manager.player_killed(&mut scene_controller, &time);
        }
    }
}

// The last thing we do is look for components that need to be deleted at the end of this frame.
//
// We had trouble double deleting asteroids after getting hi
//  by bullets or players. So now we just mark delete, and cleanup later.
fn delete_cleanup_system(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &DeleteCleanupComponent)>,
) {
    let now = time;
    for (ent, dcc) in query.iter() {
        if dcc.delete_after_frame
            || (dcc.auto_destroy_enabled && dcc.auto_destroy_when.is_expired(&now))
        {
            commands.entity(ent).despawn_recursive();
        }
    }
}

fn debug_system(
    mut commands: Commands,
    query: Query<(Entity, &DebugComponent)>,
    bullet_query: Query<(Entity, &BulletComponent, &Transform)>, // Todo, ColliderComponent in each bullet (etc), would contain info about collision size.
    player_query: Query<(Entity, &PlayerComponent, &Transform)>,
    asteroid_query: Query<(Entity, &AsteroidComponent, &Transform)>,
    alient_query: Query<(Entity, &AlienComponent, &Transform)>,
) {
    if !DEBUG {
        return;
    };

    // Delete all debug lines from last frame.
    for (ent, _) in query.iter() {
        commands.entity(ent).despawn_recursive();
    }

    let asteroid_array: Vec<(Entity, &AsteroidComponent, &Transform)> =
        asteroid_query.iter().collect();

    for (ee, ast, tr) in asteroid_array {
        // Shape test
        let shape = shapes::Circle {
            radius: ast.hit_radius,
            ..shapes::Circle::default()
        };

        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shape,
                DrawMode::Stroke(StrokeMode::new(Color::YELLOW, 0.5f32)),
                //DrawMode::Outlined {
                //    fill_mode: FillMode::color(Color::CYAN),
                //    outline_mode: StrokeMode::new(Color::WHITE, 1.0),
                //},
                Transform::from_translation(tr.translation),
            ))
            .insert(DebugComponent);
    }
}

// Start the next level when all asteroids gone.TexturesResource
// TODO: Maybe a little delay here?
fn level_system(
    mut commands: Commands,
    mut scene_controller_resource: ResMut<SceneControllerResource>,
    game_manager: Res<GameManagerResource>,
    time: Res<Time>,
    textures_resource: Res<TexturesResource>,
    query: Query<&AsteroidComponent>,
) {
    if game_manager.state == State::Over {
        return;
    }
    if query.iter().count() > 0 {
        return;
    }

    scene_controller_resource.start_level(&mut commands, &textures_resource, &time);
}

fn player_spawn_system(
    mut commands: Commands,
    textures_resource: Res<TexturesResource>,
    mut scene_controller_resource: ResMut<SceneControllerResource>,
    time: Res<Time>,
) {
    if let Some(when) = scene_controller_resource.player_spawn_when {
        if when.is_expired(&time) {
            scene_controller_resource.player_spawn_when = None;
            scene_controller_resource.respawn_player(&mut commands, &textures_resource);
        }
    }
}
