use crate::GameManagerResource;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Sounds {
    Fire,
    BangLarge,
    BangMedium,
    BangSmall,
    Beat1,
    Beat2,
    ExtraShip,
    SaucerBig,
    SaucerSmall,
    Thrust,
}

#[derive(Default)]
pub struct AudioState {
    pub audio_loaded: bool,
    //loop_handle: Handle<AudioSource>,
    pub sound_handles: HashMap<Sounds, Handle<AudioSource>>,
}

pub fn prepare_audio(asset_server: &Res<AssetServer>) -> AudioState {
    let mut audio_state = AudioState {
        audio_loaded: false,
        //loop_handle,•        sound_handles: HashMap::new(),
        sound_handles: HashMap::new(),

        ..Default::default()
    };

    audio_state
        .sound_handles
        .insert(Sounds::Fire, asset_server.load("sounds/fire.wav"));
    audio_state
        .sound_handles
        .insert(Sounds::BangLarge, asset_server.load("sounds/bangLarge.wav"));
    audio_state.sound_handles.insert(
        Sounds::BangMedium,
        asset_server.load("sounds/bangMedium.wav"),
    );
    audio_state
        .sound_handles
        .insert(Sounds::BangSmall, asset_server.load("sounds/bangSmall.wav"));
    audio_state
        .sound_handles
        .insert(Sounds::Beat1, asset_server.load("sounds/beat1.wav"));
    audio_state
        .sound_handles
        .insert(Sounds::Beat2, asset_server.load("sounds/beat2.wav"));
    audio_state
        .sound_handles
        .insert(Sounds::ExtraShip, asset_server.load("sounds/extraShip.wav"));
    audio_state
        .sound_handles
        .insert(Sounds::SaucerBig, asset_server.load("sounds/saucerBig.wav"));
    audio_state.sound_handles.insert(
        Sounds::SaucerSmall,
        asset_server.load("sounds/saucerSmall.wav"),
    );
    audio_state
        .sound_handles
        .insert(Sounds::Thrust, asset_server.load("sounds/thrust.wav"));

    audio_state
}

use bevy::asset::LoadState;

// TODO: Seems stupid to check this every frame.
pub fn check_audio_loading_system(
    mut game_manager: ResMut<GameManagerResource>,
    asset_server: ResMut<AssetServer>,
) {
    if !game_manager.audio_state.audio_loaded {
        //|| LoadState::Loaded != asset_server.get_load_state(&audio_state.loop_handle)
        if game_manager
            .audio_state
            .sound_handles
            .iter()
            .any(|(_, handle)| LoadState::Loaded != asset_server.get_load_state(&handle.clone()))
        {
            return;
        }
        game_manager.audio_state.audio_loaded = true;
    }
}
