// TODO: I think this should be my own audio plugin that wraps kira.
use bevy::{
    prelude::*,
};
use std::collections::HashMap;
use bevy_kira_audio::{Audio, AudioChannel, AudioSource};


pub struct AudioState {
    audio_loaded: bool,
    //loop_handle: Handle<AudioSource>,
    sound_handle: Handle<AudioSource>,
    channels: HashMap<AudioChannel, ChannelAudioState>,
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

pub fn play_single_sound(
    audio: Res<Audio>,
    audio_state: Res<AudioState>)
    {
    if !audio_state.audio_loaded {
        return;
    }
    println!("channel");
    let first_channel =  audio_state.channels.keys().next().unwrap(); // Get first channel.
    //let channel_audio_state = audio_state.channels.get(&first_channel).unwrap();
    //channel_audio_state.paused = false;
    //channel_audio_state.stopped = false;
    audio.play_in_channel(audio_state.sound_handle.clone(), &first_channel);
}

pub fn prepare_audio(commands: &mut Commands, asset_server: &AssetServer) {
    let mut channels = HashMap::new();
    channels.insert(
        AudioChannel::new("first".to_owned()),
        ChannelAudioState::default(),
    );
    channels.insert(
        AudioChannel::new("second".to_owned()),
        ChannelAudioState::default(),
    );
    channels.insert(
        AudioChannel::new("third".to_owned()),
        ChannelAudioState::default(),
    );

    //let loop_handle = asset_server.load("sounds/f.ogg");
    let sound_handle = asset_server.load("sounds/fire.wav");
    let audio_state = AudioState {
        channels,
        audio_loaded: false,
        //loop_handle,
        sound_handle,
    };

    commands.insert_resource(audio_state);
}

use bevy::asset::LoadState;
pub fn check_audio_loading(mut audio_state: ResMut<AudioState>, asset_server: ResMut<AssetServer>) {
    if audio_state.audio_loaded
        //|| LoadState::Loaded != asset_server.get_load_state(&audio_state.loop_handle)
        || LoadState::Loaded != asset_server.get_load_state(&audio_state.sound_handle)
    {
        return;
    }
    audio_state.audio_loaded = true;
}