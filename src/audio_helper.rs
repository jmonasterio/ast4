// TODO: I think this should be my own audio plugin that wraps kira.
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel, AudioSource};
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Tracks {
    Game, /* Fire, Bangs, Extraship */
    // TODO: Fire and bang may want to be on right/left/center tracks.
    Thrust,   /* Looping */
    Ambience, /* Looping, beat */
    Saucers,  /* Looping */
}

impl std::fmt::Display for Tracks {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

pub struct AudioState {
    audio_loaded: bool,
    //loop_handle: Handle<AudioSource>,
    sound_handles: HashMap<Sounds, Handle<AudioSource>>,

    // Tracks/channel. For now, we don't need to keep data about each channel (ChannelAudioState)
    audio_tracks: HashMap<Tracks, AudioChannel>,
}

struct ChannelAudioState {
    //stopped: bool,
//paused: bool,
//loop_started: bool,
//volume: f32,
}

impl Default for ChannelAudioState {
    fn default() -> Self {
        ChannelAudioState {
            //volume: 1.0,
            //stopped: true,
            //loop_started: false,
            //paused: false,
        }
    }
}

pub fn start_looped_sound(
    track: &Tracks,
    sound: &Sounds,
    audio: &Res<Audio>,
    audio_state: &Res<AudioState>,
) {
    if !audio_state.audio_loaded {
        return;
    }
    let channel_id = audio_state.audio_tracks.get(track).unwrap(); // Get first channel.
    audio.play_looped_in_channel(
        audio_state.sound_handles.get(sound).unwrap().clone(),
        channel_id,
    );
}

pub fn stop_looped_sound(track: &Tracks, audio: &Res<Audio>, audio_state: &Res<AudioState>) {
    let channel_id = audio_state.audio_tracks.get(track).unwrap(); // Get first channel.
    audio.stop_channel(channel_id);
}

pub fn play_single_sound(
    track: &Tracks,
    sound: &Sounds,
    audio: &Res<Audio>,
    audio_state: &Res<AudioState>,
) {
    if !audio_state.audio_loaded {
        return;
    }
    let channel_id = audio_state.audio_tracks.get(track).unwrap(); // Get first channel.
    audio.play_in_channel(
        audio_state.sound_handles.get(sound).unwrap().clone(),
        channel_id,
    );
}

pub fn prepare_audio(commands: &mut Commands, asset_server: &AssetServer) {
    let mut audio_state = AudioState {
        audio_loaded: false,
        //loop_handle,
        sound_handles: HashMap::new(),
        audio_tracks: HashMap::new(),
    };

    audio_state
        .audio_tracks
        .insert(Tracks::Game, AudioChannel::new(Tracks::Game.to_string()));
    audio_state.audio_tracks.insert(
        Tracks::Ambience,
        AudioChannel::new(Tracks::Ambience.to_string()),
    );
    audio_state.audio_tracks.insert(
        Tracks::Saucers,
        AudioChannel::new(Tracks::Saucers.to_string()),
    );
    audio_state.audio_tracks.insert(
        Tracks::Thrust,
        AudioChannel::new(Tracks::Thrust.to_string()),
    );

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

    commands.insert_resource(audio_state);
}

use bevy::asset::LoadState;

// TODO: Seems stupid to check this every frame.
pub fn check_audio_loading(mut audio_state: ResMut<AudioState>, asset_server: ResMut<AssetServer>) {
    if audio_state.audio_loaded == false {
        //|| LoadState::Loaded != asset_server.get_load_state(&audio_state.loop_handle)
        if audio_state
            .sound_handles
            .iter()
            .any(|(_, handle)| LoadState::Loaded != asset_server.get_load_state(&handle.clone()))
        {
            return;
        }
        audio_state.audio_loaded = true;
    }
}
