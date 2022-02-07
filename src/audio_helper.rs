// TODO: I think this should be my own audio plugin that wraps kira.
use crate::GameManagerResource;
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

#[derive(Default)]
pub struct AudioState {
    audio_loaded: bool,
    //loop_handle: Handle<AudioSource>,
    sound_handles: HashMap<Sounds, Handle<AudioSource>>,

    // Tracks/channel. For now, we don't need to keep data about each channel (ChannelAudioState)
    audio_tracks: HashMap<Tracks, ChannelAudioState>,
}
#[derive(Default, Clone)]
struct ChannelAudioState {
    channel: AudioChannel,
    loop_started: bool,
}

pub fn start_looped_sound(
    track: &Tracks,
    sound: &Sounds,
    audio: &Res<Audio>,
    audio_state: &mut AudioState,
) {
    if !audio_state.audio_loaded {
        return;
    }

    // TODO: This seems so sketch. Do I have to clone handle, even if not going to use?
    let maybe_sound = audio_state.sound_handles.get(sound);

    if let Some(sound) = maybe_sound {
        let handle = sound.clone();
        let cas = audio_state.audio_tracks.get_mut(track).unwrap();
        if !cas.loop_started {
            audio.play_looped_in_channel(handle, &cas.channel);
            cas.loop_started = true;
        }
    }
}

pub fn stop_looped_sound(track: &Tracks, audio: &Res<Audio>, audio_state: &AudioState) {
    let maybe_cas = audio_state.audio_tracks.get(track); // Get first channel.
    if let Some(cas) = maybe_cas { audio.stop_channel(&cas.channel); }
}

pub fn play_single_sound(
    track: &Tracks,
    sound: &Sounds,
    audio: &Res<Audio>,
    audio_state: &AudioState,
) {
    if !audio_state.audio_loaded {
        return;
    }
    let cas = audio_state.audio_tracks.get(track).unwrap(); // Get first channel.
    audio.play_in_channel(
        audio_state.sound_handles.get(sound).unwrap().clone(),
        &cas.channel,
    );
}

pub fn prepare_audio( asset_server: &Res<AssetServer>) -> AudioState {
    let mut audio_state = AudioState {
        audio_loaded: false,
        //loop_handle,•        sound_handles: HashMap::new(),
        audio_tracks: HashMap::new(),

        ..Default::default()
    };

    audio_state.audio_tracks.insert(
        Tracks::Game,
        ChannelAudioState {
            channel: AudioChannel::new(Tracks::Game.to_string()),
            loop_started: false,
        },
    );

    audio_state.audio_tracks.insert(
        Tracks::Ambience,
        ChannelAudioState {
            channel: AudioChannel::new(Tracks::Ambience.to_string()),
            loop_started: false,
        },
    );

    audio_state.audio_tracks.insert(
        Tracks::Saucers,
        ChannelAudioState {
            channel: AudioChannel::new(Tracks::Saucers.to_string()),
            loop_started: false,
        },
    );
    audio_state.audio_tracks.insert(
        Tracks::Thrust,
        ChannelAudioState {
            channel: AudioChannel::new(Tracks::Thrust.to_string()),
            loop_started: false,
        },
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

    audio_state
}

use bevy::asset::LoadState;

// TODO: Seems stupid to check this every frame.
pub fn check_audio_loading_system(
    mut game_manager: ResMut<GameManagerResource>, 
    asset_server: ResMut<AssetServer>) 
    {
    if !game_manager.audio_state.audio_loaded {
        //|| LoadState::Loaded != asset_server.get_load_state(&audio_state.loop_handle)
        if game_manager.audio_state
            .sound_handles
            .iter()
            .any(|(_, handle)| LoadState::Loaded != asset_server.get_load_state(&handle.clone()))
        {
            return;
        }
        game_manager.audio_state.audio_loaded = true;
    }
}
