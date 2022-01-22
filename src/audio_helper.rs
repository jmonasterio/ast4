// TODO: I think this should be my own audio plugin that wraps kira.
use bevy::{
    prelude::*,
};
use std::collections::HashMap;
use bevy_kira_audio::{Audio, AudioChannel, AudioSource};

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
    Thrust
}
pub struct AudioState {
    audio_loaded: bool,
    //loop_handle: Handle<AudioSource>,
    sound_handles: HashMap<Sounds,Handle<AudioSource>>,
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
    sound: Sounds,
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
    audio.play_in_channel(audio_state.sound_handles.get(&sound).unwrap().clone(), &first_channel);
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
    let mut audio_state = AudioState {
        channels,
        audio_loaded: false,
        //loop_handle,
        sound_handles: HashMap::new(),
    };

    audio_state.sound_handles.insert( Sounds::Fire, asset_server.load("sounds/fire.wav"));
    audio_state.sound_handles.insert( Sounds::BangLarge, asset_server.load("sounds/bangLarge.wav"));
    audio_state.sound_handles.insert( Sounds::BangMedium, asset_server.load("sounds/bangMedium.wav"));
    audio_state.sound_handles.insert( Sounds::BangSmall,  asset_server.load("sounds/bangSmall.wav"));
    audio_state.sound_handles.insert( Sounds::Beat1,  asset_server.load("sounds/beat1.wav"));
    audio_state.sound_handles.insert( Sounds::Beat2,  asset_server.load("sounds/beat2.wav"));
    audio_state.sound_handles.insert( Sounds::ExtraShip,  asset_server.load("sounds/extraShip.wav"));
    audio_state.sound_handles.insert( Sounds::SaucerBig,  asset_server.load("sounds/saucerBig.wav"));
    audio_state.sound_handles.insert( Sounds::SaucerSmall,  asset_server.load("sounds/saucerSmall.wav"));
    audio_state.sound_handles.insert( Sounds::Thrust,  asset_server.load("sounds/thrust.wav"));
    

    commands.insert_resource(audio_state);
}

use bevy::asset::LoadState;

// TODO: Seems stupid to check this every frame.
pub fn check_audio_loading(mut audio_state: ResMut<AudioState>, asset_server: ResMut<AssetServer>) {
    if audio_state.audio_loaded == false {
        //|| LoadState::Loaded != asset_server.get_load_state(&audio_state.loop_handle)
        if audio_state.sound_handles.iter().any( |(_,handle)| LoadState::Loaded != asset_server.get_load_state(&handle.clone())) {
            return;
        }
        audio_state.audio_loaded = true;
    }
    
}