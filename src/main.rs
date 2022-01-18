use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, //sprite::collide_aabb::{collide, Collision},
    core::FixedTimestep,
    prelude::*,
};

mod math;


// TODO: Cooler asset loader: https://www.nikl.me/blog/2021/asset-handling-in-bevy-apps/#:~:text=Most%20games%20have%20some%20sort%20of%20loading%20screen,later%20states%20can%20use%20them%20through%20the%20ECS.
// TODO: Inspector:  https://bevy-cheatbook.github.io/setup/bevy-tools.html

// Resource = Prefab
// System = Behavior
// Component =
// Entity = Entity

const TIME_STEP: f32 = 1.0 / 60.0;
const PROJECT: &'static str = "AST4!";

#[derive(Component)]
struct GameOverComponent;

#[derive(Component)]
struct FrameRateComponent;

#[derive(Component, Default)]
struct PlayerComponent {
    pub thrust: f32,
    pub max_speed: f32,
    pub player_index: u8, // Or 1, for 2 players
}

#[derive(Component, Default)]
struct RotatorComponent {
    pub snap_angle: Option<f32>,
    pub rotate_speed: f32,  //= 150f;
    pub angle_increment: f32, // = 5.0f;
}

impl RotatorComponent {

    pub fn snap_to_angle( &mut self, transform:  & mut Transform, horz: f32, time: Res<Time>) {


        let (_,_,cur_angle) = transform.rotation.to_euler( EulerRot::XYZ);
        if horz != 0.0f32 
        {
            // Assume horz is 1.0 or -1.0
            let angle_to_rotate = horz * self.rotate_speed * time.delta_seconds();
            let target_angle = cur_angle + angle_to_rotate;

            // create the change in rotation around the Z axis (pointing through the 2d plane of the screen)
            let rotation_delta = Quat::from_rotation_z(angle_to_rotate);
            // update the ship rotation with our rotation delta
            transform.rotation *= rotation_delta;

            // In case we have to stop, this will be the snap angle.
            let nearest = math::round_to_nearest_multiple(target_angle + horz * self.angle_increment,
                self.angle_increment);

            // Snap to this angle on next frame if button released.
            self.snap_angle = Some( nearest);


        }
        else
        {
            match self.snap_angle {
                Some(  snap_angle) => {
                
                    transform.rotation = Quat::from_rotation_z(snap_angle);
                    self.snap_angle = None;

                }
                None => {
                }
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
    next_free_life_score: u64
}

fn main() {
    println!("Hello, world!");

    let mut new_app = App::new();

    new_app.
        add_plugins(DefaultPlugins)
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
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                //.add_system(game_over_system)
                .with_system(player_system)
                //.with_system(paddle_movement_system)
                //.with_system(ball_collision_system)
                //.with_system(ball_movement_system),
        )
        //.add_system(scoreboard_system)
        //.add_system( change_title)
        .insert_resource(WindowDescriptor {
            title: PROJECT.to_string(),
            width: 500.,
            height: 300.,
            vsync: true,
            cursor_visible: false,
            ..Default::default()
        })
        .add_system(frame_rate)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
    //,   mut game_entities: ResMut<GameEntities>,
) {
    // hot reloading of assets.
    asset_server.watch_for_changes().unwrap();

    // Add the game's entities to our world

    // cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let texture_handle = asset_server.load("textures/Atlas.png");
    let mut texture_atlas = TextureAtlas::new_empty(texture_handle, Vec2::new (128.0, 128.0));
    TextureAtlas::add_texture(&mut texture_atlas, bevy::sprite::Rect { min: Vec2::new( 2.0, 2.0), max: Vec2::new ( 27.0, 32.0)});
    //let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(25.0,25.0),1,1);

    let texture_atlas_handle = texture_atlases.add(texture_atlas);

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
        ..Default::default()
    })
    .insert( RotatorComponent {
        snap_angle: None, angle_increment: (3.141592654f32/16.0f32), rotate_speed: 4.0f32
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

fn player_system(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query : Query<(&PlayerComponent,  &mut RotatorComponent, &mut Transform)>) {
    // println!("Player");

    let (player, mut rotator, mut transform) = query.single_mut();

    let mut dir = 0.0f32;
    if keyboard_input.pressed(KeyCode::Left) {
        dir += 1.0f32;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        dir += -1.0f32;
    } 
    rotator.snap_to_angle( &mut transform, dir, time)

}


fn game_over_system(
    _: Query<(&Text, &GameOverComponent)>) {
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

        //println!("FPS = {:.2}", fr.fps_last);
    }
}

//struct Scoreboard {
//    score: usize,
//}


#[test]
fn quat_test() {
    assert_eq!(2 + 2, 4);
}