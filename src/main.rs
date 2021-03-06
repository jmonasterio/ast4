                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                //#![windows_subsystem = "windows"] // Remove comment to turn off console log output

use std::ops::Mul;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    //ecs::system::SystemParam,
    prelude::*,
};
use bevy_prototype_lyon::prelude::*;
use bevy_render::camera::{DepthCalculation, ScalingMode, WindowOrigin, OrthographicCameraBundle,Camera2d};
use bevy::audio::{AudioSink, AudioPlugin};

mod audio_helper;

// TODO: Alient shooting.
// TODO: I want prefabs of aliens, and players, and asteroids. I clone a prefab to instantiate instead of code.
// See: https://github.com/bevyengine/bevy/issues/1515
// TODO: Exhaust
// TODO: Cooler asset loader: https://www.nikl.me/blog/2021/asset-handling-in-bevy-apps/#:~:text=Most%20games%20have%20some%20sort%20of%20loading%20screen,later%20states%20can%20use%20them%20through%20the%20ECS.
// TODO: Inspector:  https://bevy-cheatbook.github.io/setup/bevy-tools.html
// TODO: Investigate: MrGVSV/bevy_proto
// TODO: Why can't I have a sprite_renderer component like Unity has?

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
const PROJECT: &str = "AST4!";
const WIDTH: f32 = 800.0f32;
const HEIGHT: f32 = 600.0f32;
const FREE_USER_AT: u32 = 10000;
static DELETE_CLEANUP_STAGE: &str = "delete_cleanup_stage";
const MIN_HYPERSPACE_INTERVAL: f64 = 1.0f64;
const DELAY_BETWEEN_LEVELS: f64 = 1.0f64;
const ALIEN_INTERVAL: f64 = 10.0f64;
const MIN_ASTEROID_SPAWN_ALIEN: u16 = 5;
const JAWS_SOUND_INTERVAL: f64 = 1.0f64;
const MIN_ALIEN_INTERVAL: f64 = 1.0f64;
const ALIEN_INTERVAL_DECREASE: f64 = 1.0f64;

type Path2D = Vec<Vec3>;

#[derive(Default, Clone, Copy)]
struct FutureTime {
    seconds_since_startup_to_auto_destroy: f64,
}

// NOTE: I tried making an ADT with all the different event types in it, but then at the end all
//  the player_collision_system, asteroid_collion_system, etc collapse into a single method, and
//  then why have the events at all?

struct AsteroidCollisionEvent {
    asteroid: Entity,
}

struct PlayerCollisionEvent {
    player: Entity,
}

struct AlienCollisionEvent {
    alien: Entity,
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

    fn now(t: &Time) -> FutureTime {
        FutureTime::from_now(t, 0f64)
    }

    fn since(&self, t: &Time) -> f64 {
        t.seconds_since_startup() - self.seconds_since_startup_to_auto_destroy
    }

    fn is_expired(&self, t: &Time) -> bool {
        let now = t.seconds_since_startup();
        let future = self.seconds_since_startup_to_auto_destroy;
        now > future
    }
}

#[derive(Component)]
struct Particle {
    velocity: Vec3,
    lifetime: f32,
    spin: f32,
    fade: f32,
}

// These are for looped sounds.
struct ThrustSoundController(Handle<AudioSink>);
struct SaucerSoundController(Handle<AudioSink>);


// Simple particle system updater.
fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    //compute_task_pool: Res<ComputeTaskPool>,
    mut particles: Query<(&mut Particle, Entity, &mut Transform)>,
) {
    let dt = time.delta_seconds();
    //particles.par_for_each_mut(&compute_task_pool, 32, move |(mut particle,entity)| {
    particles.for_each_mut(move |(mut particle, entity, mut transform)| {
        let velocity = particle.velocity * dt;
        transform.translation += velocity;
        particle.lifetime -= dt;

        if particle.spin != 0.0f32 {
            rotate_by_angle(&mut transform, particle.spin * dt);
        }
        if particle.fade != 0.0f32 {
            transform.scale *= 1.0f32 - particle.fade * dt;
        }

        if particle.lifetime < 0.0f32 {
            commands.entity(entity).despawn_recursive();
        }
    });
}

struct ParticleEffect {
    count: u16,
    pos: Vec3,
    scale: Vec3,
    max_vel: f32,
    min_lifetime: f32,
    max_lifetime: f32,
    texture_index: usize,
    spin: f32,
    fade: f32,
    min_angle: f32,  // radians of arc to emit, like 0f32
    max_angle: f32,  // radian of arco to emit, like 2f32 * std::f32::consts::PI
}

fn create_particles(
    commands: &mut Commands,
    textures_resource: &Res<TexturesResource>,
    effect: &ParticleEffect,
) {
    for _ in 0..effect.count {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: textures_resource.texture_atlas_handle.clone(), // TODO: How to avoid clone
                sprite: TextureAtlasSprite::new(effect.texture_index),
                transform: Transform {
                    scale: effect.scale,
                    translation: effect.pos,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Particle {
                velocity: make_random_velocity(effect.max_vel / 3f32, effect.max_vel, effect.min_angle, effect.max_angle),

                lifetime: random_range(effect.min_lifetime, effect.max_lifetime),
                spin: random_sign(random_range(effect.spin / 2.0f32, effect.spin)),
                fade: effect.fade,
            });
    }
    //}
}

fn random_sign(input: f32) -> f32 {
    if ::fastrand::f32() > 0.5f32 {
        return -input;
    }
    input
}

fn random_range(min: f32, max: f32) -> f32 {
    let random = ::fastrand::f32();
    let range = max - min;
    let adjustment = range * random;
    min + adjustment
}

fn random_range_u32(min: u32, max: u32) -> u32 {
    ::fastrand::u32(std::ops::Range {
        start: min,
        end: max,
    })
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
    path: Path2D,
    path_step: usize,
    hit_radius: f32,
}

#[derive(Component)]
struct MuzzleComponent;

#[derive(Component)]
struct ThrusterComponent;


#[derive(Component)]
struct PlayerHitComponent;

#[derive(Component, Default)]
struct PlayerComponent {
    pub thrust: f32,
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

#[derive(Default)]
pub struct GameManagerResource {
    state: State,
    score: u32,
    lives: u32,
    next_free_life_score: u32,
    level: u32,
    next_jaws_sound_time: Option<FutureTime>,
    jaw_interval_seconds: f64,
    jaws_alternate: bool,
    next_alien_time: Option<FutureTime>,
    game_started_this_frame: bool,
    audio_state: audio_helper::AudioState,
    // -- Do some work at a future time.
    // TODO: Maybe make these a map, so I can have N of them. Or an enum or something.
    player_spawn_when: Option<FutureTime>,
    level_start_when: Option<FutureTime>,

    time_between_aliens: f64,
}

impl GameManagerResource {
    fn player_killed(&mut self, time: &Res<Time>) {
        if self.lives > 0 {
            self.lives -= 1;
        }
        if self.lives < 1 {
            self.state = State::Over;
        } else {
            self.respawn_player_later(FutureTime::from_now(time, 2.0f64));
        }
    }

    fn spawn_alien_later( &mut self, time: &Time) {
        self.next_alien_time = Some(FutureTime::from_now(time, self.time_between_aliens));
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

        let thruster_id = commands
            .spawn()            
            .insert(Transform {
                translation: Vec3::new(0f32, -9.0f32, 0f32),
                ..Default::default()
            })
            .insert(GlobalTransform {
                ..Default::default()
            })
            .insert(ThrusterComponent {})
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
                thrust: 100.0f32,
                friction: 0.999f32, /* 1-0.02 */
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

        commands.entity(player_id).push_children(&[muzzle_id, thruster_id]);
        
        let hit_points: [Entity; 5] = [
            commands
                .spawn()
                .insert(Transform {
                    translation: Vec3::new(0f32, 15f32, 0f32),
                    ..Default::default()
                })
                .insert(GlobalTransform {
                    ..Default::default()
                })
                .insert(PlayerHitComponent {})
                .id(),
            commands
                .spawn()
                .insert(Transform {
                    translation: Vec3::new(-12.5f32, -15f32, 0f32),
                    ..Default::default()
                })
                .insert(GlobalTransform {
                    ..Default::default()
                })
                .insert(PlayerHitComponent {})
                .id(),
            commands
                .spawn()
                .insert(Transform {
                    translation: Vec3::new(12.5f32, -15f32, 0f32),
                    ..Default::default()
                })
                .insert(GlobalTransform {
                    ..Default::default()
                })
                .insert(PlayerHitComponent {})
                .id(),
            commands
                .spawn()
                .insert(Transform {
                    translation: Vec3::new(6.25f32, 0f32, 0f32),
                    ..Default::default()
                })
                .insert(GlobalTransform {
                    ..Default::default()
                })
                .insert(PlayerHitComponent {})
                .id(),
            commands
                .spawn()
                .insert(Transform {
                    translation: Vec3::new(-6.25f32, 0f32, 0f32),
                    ..Default::default()
                })
                .insert(GlobalTransform {
                    ..Default::default()
                })
                .insert(PlayerHitComponent {})
                .id(),
        ];

        commands.entity(player_id).push_children(&hit_points);
    }

    fn start_game(
        &mut self,
        mut commands: Commands,
        textures_resource: Res<TexturesResource>,
        time: &Res<Time>,
    ) {
        seed_rng(time); // reseed again.
        self.lives = 4;
        self.score = 0;
        self.state = State::Playing;

        self.level = 0;
        self.next_free_life_score = FREE_USER_AT;

        self.game_started_this_frame = true;

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
        self.next_jaws_sound_time = Some(FutureTime::from_now(time, JAWS_SOUND_INTERVAL));
        self.add_asteroids(1 + self.level, commands, textures_resource); // 3.0 + Mathf.Log( (float) Level)));
        self.spawn_alien_later( time);

        self.time_between_aliens -= ALIEN_INTERVAL_DECREASE;
        if self.time_between_aliens < MIN_ALIEN_INTERVAL {
            self.time_between_aliens = MIN_ALIEN_INTERVAL;
        }

    }

    fn add_asteroids(
        &mut self,
        count: u32,
        commands: &mut Commands,
        textures_resource: &Res<TexturesResource>,
    ) {
        for _ in 0..count - 1 {
            let pos = GameManagerResource::make_safe_asteroid_pos();
            GameManagerResource::add_asteroid_with_size_at(
                commands,
                textures_resource,
                &AsteroidSize::Large,
                pos,
            )
        }
    }

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

        // Randomly flip the sprite so they are a little more different
        let mut sprite_at_index = TextureAtlasSprite::new(index);
        sprite_at_index.flip_x = fastrand::f32() > 0.5f32;

        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: textures_resource.texture_atlas_handle.clone(), // TODO: How to avoid clone
                sprite: sprite_at_index,
                transform: Transform {
                    scale: Vec3::splat(1.0f32),
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
                v: make_random_velocity(100f32, 200f32, 0f32, 2f32 * std::f32::consts::PI),
                spin: 1.0f32,
                max_speed: 200f32,
            })
            .insert(DeleteCleanupComponent {
                delete_after_frame: false,
                auto_destroy_enabled: false,
                ..Default::default()
            });
    }

    // TODO
    fn make_safe_asteroid_pos() -> Vec3 {

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
    pub fn apply_thrust(&mut self, thrust: f32, direction: &Quat, time: &Res<Time>) {
        let (_, _, angle_radians) = direction.to_euler(EulerRot::XYZ);
        let thrust_vector =
            thrust * Vec3::new(-f32::sin(angle_radians), f32::cos(angle_radians), 0f32);
        self.v += thrust_vector * time.delta_seconds();
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

// TODO: Would it help if this were part of the GameManagerResource???
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
    alien_small_index: usize,
    alien_large_index: usize,

    asteroid_large_hit_radius: f32,
    asteroid_medium_hit_radius: f32,
    asteroid_small_hit_radius: f32,

    alien_small_hit_radius: f32,
    alien_large_hit_radius: f32,
}

fn seed_rng(t: &Res<Time>) {
    let in_ms = t.seconds_since_startup();
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
            present_mode: bevy_window::PresentMode::Mailbox,
            cursor_visible: false,
            decorations: false, // Hide the white flashing window at atartup
            // mode: bevy_window::WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(ShapePlugin)
        .add_event::<AsteroidCollisionEvent>()
        .add_event::<PlayerCollisionEvent>()
        .add_event::<AlienCollisionEvent>()
        .insert_resource(GameManagerResource {
            state: State::Over,
            next_free_life_score: FREE_USER_AT,
            level: 0,
            jaw_interval_seconds: 0.9f64,
            jaws_alternate: false,
            next_jaws_sound_time: None,
            time_between_aliens: ALIEN_INTERVAL,
            ..Default::default()
        })
        .insert_resource(TexturesResource {
            ..Default::default()
        })
        .add_startup_system(setup)
        .add_system(audio_helper::check_audio_loading_system)
        .add_stage_after(
            CoreStage::Update,
            DELETE_CLEANUP_STAGE,
            SystemStage::single_threaded(),
        )
        .add_system_to_stage(DELETE_CLEANUP_STAGE, delete_cleanup_system)
        .add_system_to_stage(DELETE_CLEANUP_STAGE, clear_at_game_start_system)
        .add_system_set(
            SystemSet::new()
                .with_system(player_system)
                .with_system(alien_update_system)
                .with_system(velocity_system)
                .with_system(wrapped_2d_system)
                .with_system(update_ambience_sound_system)
                .with_system(collision_system)
                .with_system(asteroid_collision_system)
                .with_system(player_collision_system)
                .with_system(alien_collision_system)
                .with_system(update_particles)
                .with_system(player_spawn_system)
                .with_system(alien_spawn_system)
                .with_system(level_system)
                .with_system(score_system)
                .with_system(lives_system)
                .with_system(game_over_system)
                .with_system(start_game_system),
        );

    if built_info::CFG_OS == "windows" {
        new_app.add_system(bevy::input::system::exit_on_esc_system);
    }

    if DEBUG {
        new_app
            .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_system(debug_system)
            .add_system(frame_rate)
            .insert_resource(FrameRateResource {
                delta_time: 0f64,
                display_frame_rate: true,
                debug_sinusoidal_frame_rate: false,
                fps_last: 0f64,
            });
    }

    new_app.run();
}

pub fn new_camera_2d() -> OrthographicCameraBundle<Camera2d> {
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
    mut game_manager: ResMut<GameManagerResource>,
    time: Res<Time>,
) {
    seed_rng(&time);

    // Load audio assets.
    game_manager.audio_state = audio_helper::prepare_audio(&asset_server);

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

    let alien_small_rect = bevy::sprite::Rect {
        min: Vec2::new(6.0, 64.0),
        max: Vec2::new(34.0, 79.0),
    };
    textures_resource.alien_small_index =
        TextureAtlas::add_texture(&mut texture_atlas, alien_small_rect);
    textures_resource.alien_small_hit_radius = large_asteroid_rect.width() / 2.0f32;

    let alien_large_rect = bevy::sprite::Rect {
        min: Vec2::new(6.0, 64.0),
        max: Vec2::new(34.0, 79.0),
    };
    textures_resource.alien_large_index =
        TextureAtlas::add_texture(&mut texture_atlas, alien_large_rect);
    textures_resource.alien_large_hit_radius = large_asteroid_rect.width() / 2.0f32;

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

    let fps_label = if DEBUG { "FPS: " } else { "" };

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: fps_label.to_string(),
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

    // Create an explosion for player.
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

// TODO: maybe split into player_move and player_shoot system.s
// TODO: Maybe I wouldn't be sending so many params around, if I used
//  events to, e.g. eventStartSound(), eventSpawn
fn player_system(
    mut commands: Commands,
    game_manager: ResMut<GameManagerResource>,
    keyboard_input: Res<Input<KeyCode>>,
    textures_resource: Res<TexturesResource>,
    time: Res<Time>,
    audio: Res<Audio>,
    thrust_sound_controller: Option<Res<ThrustSoundController>>,
    audio_sinks: Res<Assets<AudioSink>>,
    mut query: Query<(
        &mut PlayerComponent,
        &mut Transform,
        &mut VelocityComponent,
        &mut ShooterComponent,
    )>,
    bullet_query: Query<&BulletComponent>,
    muzzle_query: Query<(&MuzzleComponent, &GlobalTransform)>,
    thruster_query: Query<(&ThrusterComponent, &GlobalTransform)>,
) {
    if query.is_empty() || game_manager.state == State::Over {
        return; // No player.
    }
    let (mut player, mut player_transform, mut velocity, shooter) = query.single_mut();

    let mut dir = 0.0f32;
    if keyboard_input.pressed(KeyCode::Left) {
        dir += 1.0f32;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        dir += -1.0f32;
    }
    player.rotate_to_angle_with_snap(&mut player_transform, dir, &time);

    if keyboard_input.just_pressed(KeyCode::Up) {

        // I probably had to duplicate this code bcause of ThrustSoundController
        if let Some(sound) = game_manager.audio_state.sound_handles.get( &audio_helper::Sounds::Thrust) {
            let sink = audio.play_with_settings(sound.clone(), PlaybackSettings { repeat: true, ..Default::default() });
            let sink_handle = audio_sinks.get_handle( sink);
            commands.insert_resource( ThrustSoundController(sink_handle));
        }
    } else if keyboard_input.just_released(KeyCode::Up) {
        if let Some(sink) = audio_sinks.get(&thrust_sound_controller.unwrap().0) {
            sink.stop();
        }
    }

    // Maybe a thruster component? Or maybe Rotator+Thruster=PlayerMover component.
    if keyboard_input.pressed(KeyCode::Up) {
        // Too much trouble to implement rigid body like in Unity, so wrote my own.
        // Assume no friction while accelerating.
        velocity.apply_thrust(player.thrust, &player_transform.rotation, &time);

        let (_, thruster_transform) = thruster_query.single();

        let angle = vec3_to_radians( player_transform.rotation) - std::f32::consts::PI;

        // Create thrust cone
        create_particles(
            &mut commands,
            &textures_resource,
            &ParticleEffect {
                count: (100.0f32 * time.delta_seconds()) as u16,
                pos: thruster_transform.translation,
                scale: bevy::prelude::Vec3::splat(1.25f32),
                max_vel: 100.0f32,
                min_lifetime: 0.01f32,
                max_lifetime: 0.1f32,
                texture_index: textures_resource.explosion_particle_index,
                spin: 0.0f32,
                fade: 1.0f32,
                min_angle: angle-1f32,
                max_angle: angle+1f32,
            },
        );

    } else {
        velocity.apply_friction( 1.0f32 - player.friction * time.delta_seconds() ); // TODO: This is weird.
    }

    if (keyboard_input.just_pressed(KeyCode::LControl)
        || keyboard_input.just_pressed(KeyCode::RControl))
        && bullet_query.iter().count() < shooter.max_bullets
    {
        let (_, muzzle_transform) = muzzle_query.single();

        if let Some(sound) = game_manager.audio_state.sound_handles.get( &audio_helper::Sounds::Fire) {
            audio.play(sound.clone());
        }

        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: textures_resource.texture_atlas_handle.clone(), // TODO: is this really good?
                sprite: TextureAtlasSprite::new(textures_resource.bullet_index),
                transform: Transform {
                    scale: Vec3::splat(1.0),
                    translation: muzzle_transform.translation,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(BulletComponent {
            })
            .insert(VelocityComponent {
                v: calc_player_normalized_pointing_dir(&player_transform).mul(shooter.bullet_speed),
                max_speed: 5000.0f32, // TODO: Speed should be a struct
                spin: 0.0f32,
            })
            .insert(Visibility { is_visible: true })
            .insert(Wrapped2dComponent {})
            .insert(DeleteCleanupComponent {
                delete_after_frame: false,
                auto_destroy_enabled: true,
                auto_destroy_when: FutureTime::from_now(&time, 1.2f64),
            });
    }

    if keyboard_input.pressed(KeyCode::Return)
        && time.seconds_since_startup() - player.last_hyperspace_time > MIN_HYPERSPACE_INTERVAL
    {
        player_transform.translation = make_random_pos(); // Not safe on purpose
        player.last_hyperspace_time = time.seconds_since_startup();
    }
}

pub fn round_to_nearest_multiple(f: f32, multiple: f32) -> f32 {
    f32::round(f / multiple) * multiple
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
            let nearest = round_to_nearest_multiple(
                target_angle + horz * self.angle_increment, // TODO: this may be laggy.
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

fn calc_player_normalized_pointing_dir(p: &Transform) -> Vec3 {
    radians_to_vec3(vec3_to_radians(p.rotation))
}

fn radians_to_vec3(angle_radians: f32) -> Vec3 {
    Vec3::new(-f32::sin(angle_radians), f32::cos(angle_radians), 0f32)
}

fn vec3_to_radians( rot: Quat) -> f32 {
    let (_, _, angle_radians) = rot.to_euler(EulerRot::XYZ);
    angle_radians
}

fn make_random_pos() -> Vec3 {
    let x = fastrand::f32();
    let y = fastrand::f32();
    Vec3::new(x * WIDTH, y * HEIGHT, 0f32)
}

fn make_random_velocity(min_speed: f32, max_speed: f32, min_angle: f32, max_angle: f32) -> Vec3 {
    let angle_radians = random_range(min_angle, max_angle);
    let vector = radians_to_vec3(angle_radians);
    let speed = random_range(min_speed, max_speed);
    speed * vector
}

// Show or hide instructions based on game state.
fn game_over_system(
    mut commands: Commands,
    time: Res<Time>,
    textures_resource: Res<TexturesResource>,
    game_manager: Res<GameManagerResource>,
    mut query: Query<(&mut Visibility, &GameOverComponent)>,
) {
    let is_over = game_manager.state == State::Over;
    for (mut vis, _) in query.iter_mut() {
        vis.is_visible = is_over;
    }

    if is_over {
        create_particles(
            &mut commands,
            &textures_resource,
            &ParticleEffect {
                count: (500.0f32 * time.delta_seconds()) as u16,
                pos: Vec3::new(0.53f32 * WIDTH, 0.6f32 * HEIGHT, 0.0f32),
                scale: bevy::prelude::Vec3::splat(0.5f32),
                max_vel: 300.0f32,
                min_lifetime: 1.5f32,
                max_lifetime: 4.0f32,
                texture_index: textures_resource.explosion_particle_index,
                spin: 0.0f32,
                fade: 0.80f32,
                min_angle: 0f32,
                max_angle: 2f32 * std::f32::consts::PI,
            },
        );
    }
}

fn score_system(
    mut game_manager: ResMut<GameManagerResource>,
    audio: Res<Audio>,
    mut query: Query<(&mut Visibility, &ScoreComponent, &mut Text)>,
) {
    let is_playing = game_manager.state == State::Playing;
    let (mut vis, _, mut text) = query.single_mut();
    vis.is_visible = is_playing;

    text.sections[0].value = game_manager.score.to_string();

    if game_manager.score > game_manager.next_free_life_score {
        game_manager.next_free_life_score += FREE_USER_AT;
        game_manager.lives += 1;

        if let Some(sound) = game_manager.audio_state.sound_handles.get( &audio_helper::Sounds::ExtraShip) {
            audio.play(sound.clone());
        }
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

// TODO: Reduce number of params :https://github.com/bevyengine/bevy/issues/3267
//#[derive(SystemParam)]
//struct MySystemParam<'w, 's> {
//    audio: Res<'w, Audio>,
//    textures_resource: Res<'w, TexturesResource>,
//    _query: Query<'w, 's, ()>,
//}

fn start_game_system(
    commands: Commands,
    mut game_manager: ResMut<GameManagerResource>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    textures_resource: Res<TexturesResource>,
) {
    if game_manager.state == State::Over && keyboard_input.pressed(KeyCode::Space) {
        game_manager.start_game(commands, textures_resource, &time);
    }
}

fn update_ambience_sound_system(
    time: Res<Time>,
    mut game_manager: ResMut<GameManagerResource>,
    audio: Res<Audio>,
) {
    if game_manager.state == State::Playing
        && game_manager.next_jaws_sound_time.unwrap().is_expired(&time)
    // if in level
    {
        if game_manager.jaw_interval_seconds > 0.1800f64 {
            game_manager.jaw_interval_seconds -= 0.005f64
        }
        game_manager.next_jaws_sound_time = Some(FutureTime::from_now(
            &time,
            game_manager.jaw_interval_seconds,
        ));
        let sound_handle = (if game_manager.jaws_alternate {
            &audio_helper::Sounds::Beat1 } else {
                &audio_helper::Sounds::Beat2
            });

        if let Some(sound) = game_manager.audio_state.sound_handles.get( sound_handle ) {
            audio.play_with_settings(sound.clone(), PlaybackSettings::ONCE.with_speed(0.25f32));
        }
        game_manager.jaws_alternate = !game_manager.jaws_alternate;
    }
}

// Not easy-to-use physics like unity, so had to implement my own

fn collision_system(
    mut commands: Commands,
    mut ev_asteroid_collision: EventWriter<AsteroidCollisionEvent>, // TODO: Can I have two type params, like ASTERID?
    mut ev_player_collision: EventWriter<PlayerCollisionEvent>,
    mut ev_alien_collision: EventWriter<AlienCollisionEvent>,
    bullet_query: Query<(Entity, &BulletComponent, &Transform)>, // Todo, ColliderComponent in each bullet (etc), would contain info about collision size.
    player_query: Query<(Entity, &PlayerComponent, &Transform)>,
    asteroid_query: Query<(Entity, &AsteroidComponent, &Transform)>,
    alien_query: Query<(Entity, &AlienComponent, &Transform)>,
    hit_point_query: Query<(Entity, &PlayerHitComponent, &GlobalTransform)>,
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
    let alien_array: Vec<(Entity, &AlienComponent, &Transform)> = alien_query.iter().collect();
    let hitpoint_array: Vec<(Entity, &PlayerHitComponent, &GlobalTransform)> =
        hit_point_query.iter().collect();

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
        let near_bullet_area = make_area_around(bul_trans, 30.0f32);

        // TODO: Is the first returned asteroid, really the closest?

        if let Some(entry) = asteroid_qt.query(near_bullet_area).next() {
            let idx = entry.value_ref();
            let (ast_ent, ast_comp, ast_trans) = asteroid_array[*idx];

            // So we're close. Let's calculate actual distance based on size.
            let d = bul_trans.translation.distance(ast_trans.translation);
            if d < ast_comp.hit_radius {
                commands.entity(*bul_ent).despawn_recursive();
                ev_asteroid_collision.send(AsteroidCollisionEvent { asteroid: ast_ent });
                break;
            }
        }

        if !alien_array.is_empty() {
            let (alien_ent, alien_comp, alien_trans) = alien_array[0];

            // TODO: SImilar code in next for loop
            // So we're close. Let's calculate actual distance based on size.
            let d = bul_trans.translation.distance(alien_trans.translation);
            if d < alien_comp.hit_radius {
                commands.entity(*bul_ent).despawn_recursive(); // TODO: DO after frame.
                ev_alien_collision.send(AlienCollisionEvent { alien: alien_ent });
                break;
            }
        }
    }

    for (player_ent, _, play_trans) in &player_array {
        let near_player_area = make_area_around(play_trans, 100.0f32); // TOD: hardcode
        for entry in asteroid_qt.query(near_player_area) {
            let idx = entry.value_ref();
            let (_, ast_comp, ast_trans) = asteroid_array[*idx];

            // We really want to see whether the hit points of the ship is inside
            //  the radius of the asteroid. Ignore shape, of asteroid -- assume circle
            for (_, _, glob_trns) in &hitpoint_array {
                let d = glob_trns.translation.distance(ast_trans.translation); // TODO do after frame.
                if d < ast_comp.hit_radius {
                    ev_player_collision.send(PlayerCollisionEvent {
                        player: *player_ent,
                    });
                    break;
                }
            }
        }
        if !alien_array.is_empty() {
            let (_, alien_comp, alien_trans) = alien_array[0]; // TODO: There should only be one.

            let d = play_trans.translation.distance(alien_trans.translation);
            if d < alien_comp.hit_radius {
                ev_player_collision.send(PlayerCollisionEvent {
                    player: *player_ent,
                });
                break;
            }
        }
    }
}

fn spawn_asteroid_or_alien_explosion(
    commands: &mut Commands,
    textures_resource: &Res<TexturesResource>,
    trans: &Transform,
) {
    create_particles(
        commands,
        textures_resource,
        &ParticleEffect {
            count: 20,
            pos: trans.translation,
            scale: bevy::prelude::Vec3::splat(1.0f32),
            max_vel: 50.0f32,
            min_lifetime: 0.9f32,
            max_lifetime: 1.1f32,
            texture_index: textures_resource.explosion_particle_index,
            spin: 0.0f32,
            fade: 0.8f32,
            min_angle: 0f32,
            max_angle: 2f32 * std::f32::consts::PI,
        },
    );
}

fn alien_collision_system(
    mut commands: Commands,
    mut ev_collision: EventReader<AlienCollisionEvent>,
    mut query: Query<(Entity, &mut DeleteCleanupComponent, &Transform)>,
    mut game_manager: ResMut<GameManagerResource>,
    textures_resource: Res<TexturesResource>,
    audio: Res<Audio>,
    time: Res<Time>,
    saucer_sound_controller: Option<Res<SaucerSoundController>>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    for ev in ev_collision.iter() {
        // This is pretty inefficient.
        if let Ok((_, mut dcc, trans)) = query.get_mut(ev.alien) {
            dcc.delete_after_frame = true;

            if let Some(sound) = game_manager.audio_state.sound_handles.get( &audio_helper::Sounds::BangSmall) {
                audio.play(sound.clone());
            }

            spawn_asteroid_or_alien_explosion( &mut commands, &textures_resource, &trans);

            // Stop alien sound
            println!("Stop looped sound");
            if let Some(ssc) = &saucer_sound_controller {
                if let Some(sink) = audio_sinks.get(&ssc.0) {
                    sink.stop();
                }
            }

            // Treat killing an alien, like killing an asteroid.
            game_manager.spawn_alien_later(&time);

        }
    }
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
    time: Res<Time>,
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

                if let Some(sound) = game_manager.audio_state.sound_handles.get( &audio_helper::Sounds::BangLarge) {
                    audio.play(sound.clone());
                }
    
                let mut replace_size: Option<AsteroidSize> = None;
                match ast.size {
                    AsteroidSize::Large => {
                        game_manager.score += 20;

                        replace_size = Some(AsteroidSize::Medium)
                    }
                    AsteroidSize::Medium => {
                        game_manager.score += 50;

                        replace_size = Some(AsteroidSize::Small);
                    }
                    AsteroidSize::Small => {
                        game_manager.score += 100;
                    }
                }
                spawn_asteroid_or_alien_explosion(&mut commands, &textures_resource, trans);
                game_manager.spawn_alien_later(&time);

                if let Some(size) = replace_size {
                    // Replace with 2 smaller asteroids
                    for _ in 0..2 {
                        GameManagerResource::add_asteroid_with_size_at(
                            &mut commands,
                            &textures_resource,
                            &size,
                            trans.translation,
                        );
                    }
                }
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

fn player_collision_system(
    mut commands: Commands,
    mut ev_collision: EventReader<PlayerCollisionEvent>,
    mut query: Query<(Entity, &mut DeleteCleanupComponent, &Transform)>,
    mut game_manager: ResMut<GameManagerResource>,
    textures_resource: Res<TexturesResource>,
    time: Res<Time>,
    audio: Res<Audio>,
    thrust_sound_controller: Option<Res<ThrustSoundController>>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    for ev in ev_collision.iter() {
        // This is pretty inefficient.
        if let Ok((_, mut dcc, trans)) = query.get_mut(ev.player) {
            dcc.delete_after_frame = true;

            if let Some(sound) = game_manager.audio_state.sound_handles.get( &audio_helper::Sounds::BangSmall) {
                audio.play(sound.clone());
            }

            // Create an explosion for player.
            create_particles(
                &mut commands,
                &textures_resource,
                &ParticleEffect {
                    count: 5,
                    pos: trans.translation,
                    scale: bevy::prelude::Vec3::splat(1.0f32),
                    max_vel: 100.0f32,
                    min_lifetime: 1.5f32,
                    max_lifetime: 2.0f32,
                    texture_index: textures_resource.ship_particle_index,
                    spin: 2.0f32,
                    fade: 0.95f32,
                    min_angle: 0f32,
                    max_angle: 2f32 * std::f32::consts::PI,
                },
            );

            // STOP thrust sound when player_killed
            // Need to do this better.
            if let Some(tsc) = &thrust_sound_controller {

                if let Some(sink) = audio_sinks.get(&tsc.0) {
                    sink.stop();
                }
            }
            
            // Delete lifes
            game_manager.player_killed(&time);
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

fn clear_at_game_start_system(
    mut commands: Commands,
    query: Query<(Entity, &AsteroidComponent)>,
    mut game_manager: ResMut<GameManagerResource>,
) {
    if game_manager.game_started_this_frame {
        for (ent, _) in query.iter() {
            commands.entity(ent).despawn_recursive();
        }
        game_manager.game_started_this_frame = false;
    }

    // todo: clear bullets and aliens?
}

fn debug_system(
    mut commands: Commands,
    query: Query<(Entity, &DebugComponent)>,
    //bullet_query: Query<(Entity, &BulletComponent, &Transform)>, // Todo, ColliderComponent in each bullet (etc), would contain info about collision size.
    //player_query: Query<(Entity, &PlayerComponent, &Transform)>,
    asteroid_query: Query<(Entity, &AsteroidComponent, &Transform)>,
    //alien_query: Query<(Entity, &AlienComponent, &Transform)>,
    hit_points: Query<(Entity, &PlayerHitComponent, &GlobalTransform)>,
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

    for (_, ast, tr) in asteroid_array {
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

    let hit_point_array: Vec<(Entity, &PlayerHitComponent, &GlobalTransform)> =
        hit_points.iter().collect();

    for (_, _, tr) in hit_point_array {
        // Shape test
        let shape = shapes::Circle {
            radius: 2.0f32,
            ..shapes::Circle::default()
        };

        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shape,
                DrawMode::Stroke(StrokeMode::new(Color::GREEN, 0.5f32)),
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
fn level_system(
    mut commands: Commands,
    mut game_manager: ResMut<GameManagerResource>,
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

    // Delay a bit before starting the next level
    // TODO: This is cumbersome. Can we make it nicer?
    match game_manager.level_start_when {
        None => {
            game_manager.level_start_when = Some(FutureTime::from_now(&time, DELAY_BETWEEN_LEVELS));
        }
        Some(when) => {
            if when.is_expired(&time) {
                game_manager.start_level(&mut commands, &textures_resource, &time);
                game_manager.level_start_when = None;
            }
        }
    }
}

fn player_spawn_system(
    mut commands: Commands,
    textures_resource: Res<TexturesResource>,
    mut game_manager: ResMut<GameManagerResource>,
    time: Res<Time>,
) {
    if let Some(when) = game_manager.player_spawn_when {
        if when.is_expired(&time) {
            game_manager.player_spawn_when = None;
            game_manager.respawn_player(&mut commands, &textures_resource);
            game_manager.spawn_alien_later(&time);
        }
    }
}

fn make_random_path() -> Path2D {
    let mut path = vec![Vec3::new(
        0f32,
        random_range(HEIGHT * 0.2f32, HEIGHT * 0.8f32),
        0f32,
    ),
    Vec3::new(
        0.25f32 * WIDTH,
        random_range(HEIGHT * 0.2f32, HEIGHT * 0.8f32),
        0f32,
    ),
    Vec3::new(
        0.75f32 * WIDTH,
        random_range(HEIGHT * 0.2f32, HEIGHT * 0.8f32),
        0f32,
    ),
    Vec3::new(
        WIDTH,
        random_range(HEIGHT * 0.2f32, HEIGHT * 0.8f32),
        0f32,
    )];

    if random_range_u32(0, 2) == 0 {
        path.reverse();
    }
    path
}

fn alien_update_system(
    mut game_manager: ResMut<GameManagerResource>, // Seems like sound should attach to entity and be killed with it.
    time: Res<Time>,
    saucer_sound_controller: Option<Res<SaucerSoundController>>,
    audio_sinks: Res<Assets<AudioSink>>,
    mut aliens_query: Query<(
        &mut AlienComponent,
        &Transform,
        &mut VelocityComponent,
        &mut DeleteCleanupComponent,
    )>
) {
    if aliens_query.is_empty() {
        return;
    };

    let (mut alien, trans, mut vel, mut dcc) = aliens_query.single_mut();

    if dcc.delete_after_frame {
        return;
    }

    assert!( alien.path_step+1 < alien.path.len());

    let target = alien.path[alien.path_step + 1];
    let cur_pos = trans.translation;

    if cur_pos.distance(target) <= 5f32 {
        alien.path_step += 1;
        println!("path_step: {}    path_len: {}", alien.path_step, alien.path.len());
        if alien.path_step >= alien.path.len()-1 {
            //If end of path, we're done.
            dcc.delete_after_frame = true;

            // Stop alien sound
            println!("Stop looped sound");
            if let Some(sink) = audio_sinks.get(&saucer_sound_controller.unwrap().0) {
                sink.stop();
            }
            game_manager.spawn_alien_later(&time);
        }
    } else {
        // Go towards
        let dir = target - cur_pos;
        vel.v = dir.normalize() * vel.max_speed;
    }

    //var hit = Physics2D.Raycast(transform.position, rigidBody.velocity, distance:10.0f, layerMask:9 /* Asteroid */);
    //if (hit.collider != null)
    //{
    //    float distance = Vector2.Distance(hit.point, transform.position); //n + Vector2.Up From2D(rigidBody.velocity.normalized*transform.localScale.magnitude)); // Extra math to not hit self.
    //    if (distance > 0 && distance < 4.0f)
    //    {
    //        print("distance: " + distance);
    //        distance = 0;
    //    }
    //
    //}

    //if (_bullet == null)
    //{
    //    FireBullet();
    //}
}

fn alien_spawn_system(
    mut commands: Commands,
    game_manager: ResMut<GameManagerResource>,
    other_aliens_query: Query<&AlienComponent>,
    asteroids_query: Query<&AsteroidComponent>,
    time: Res<Time>,
    audio: Res<Audio>,
    textures_resource: Res<TexturesResource>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    if game_manager.state == State::Playing && other_aliens_query.iter().count() == 0 {
        match game_manager.next_alien_time {
            None => {}
            Some(laka) => {
                let diff = laka.since( &time); //.seconds_since_startup() - laka.seconds_since_startup_to_auto_destroy;

                let mut ast_count = 0;
                for ast in asteroids_query.iter() {
                    match ast.size {
                        AsteroidSize::Large => { ast_count +=4; }
                        AsteroidSize::Medium => { ast_count +=2; }
                        AsteroidSize::Small => { ast_count +=1; }
                    }
                }

                if (diff > 0.0f64)
                    || diff > MIN_ALIEN_INTERVAL && ast_count < MIN_ASTEROID_SPAWN_ALIEN // MAGIC
                        && random_range_u32(0, 1000) > (996 - game_manager.level * 2)
                {
                    let alien_size = if random_range_u32(0, 3) == 0 {
                        AlienSize::Small
                    } else {
                        AlienSize::Large
                    };

                    let alien_hit_radius = if alien_size == AlienSize::Small {
                        textures_resource.alien_small_hit_radius
                    } else {
                        textures_resource.alien_large_hit_radius
                    };

                    let alien_scale_factor = if alien_size == AlienSize::Small {
                        0.6f32
                    } else {
                        1.0f32
                    };

                    let random_path = make_random_path();
                    let start_pos = random_path[0];

                    commands
                        .spawn_bundle(SpriteSheetBundle {
                            texture_atlas: textures_resource.texture_atlas_handle.clone(), // TODO: How to avoid clone
                            sprite: TextureAtlasSprite::new(if alien_size == AlienSize::Small {
                                textures_resource.alien_small_index
                            } else {
                                textures_resource.alien_large_index
                            }),
                            transform: Transform {
                                scale: Vec3::splat(alien_scale_factor),
                                translation: start_pos,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(AlienComponent {
                            size: alien_size.clone(),
                            path: random_path,
                            path_step: 0,
                            hit_radius: alien_hit_radius,
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
                        });

                    // TBD: Wouldn't it be cool if the alien knew it was spawned and played its own sound.

                    // I probably had to duplicate this code bcause of ThrustSoundController
                    let snd = &(if alien_size == AlienSize::Small {
                        audio_helper::Sounds::SaucerSmall
                    } else {
                        audio_helper::Sounds::SaucerBig
                    });

                    if let Some(sound) = game_manager.audio_state.sound_handles.get( snd) {
                        let sink = audio.play_with_settings(sound.clone(), PlaybackSettings { repeat: true, ..Default::default() });
                        let sink_handle = audio_sinks.get_handle( sink);
                        commands.insert_resource( SaucerSoundController(sink_handle));
                    }
                }
            }
        }
    }
}
