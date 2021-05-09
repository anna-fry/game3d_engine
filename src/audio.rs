use std::{sync::mpsc::{self, Sender, Receiver}};
use std::thread;

use rodio::{SpatialSink};

use crate::geom::{Pos3};
use crate::music::Sound;

pub struct Audio {
    control_channel: Option<Sender<(bool, f32, f32, f32, Option<Sound>)>>,
    sink: Option<SpatialSink>,
}

impl Audio {
    pub fn new() -> Self {
        if cfg!(windows) {
            let (tx, rx): (Sender<(bool, f32, f32, f32, Option<Sound>)>, Receiver<(bool, f32, f32, f32, Option<Sound>)>) = mpsc::channel();
            thread::spawn(move || {
                let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
                let sink = Sound::spatial_sink(&stream_handle, [-2.0, 1.5, -3.0], [-19.0, 5.0, -20.0], [-21.0, 5.0, -20.0]);
                while let Ok((should_play, x, y, z, audio_source)) = rx.recv() {
                    if should_play {
                        sink.set_emitter_position([x, y, z]);
                        match audio_source {
                            Some(audio) => sink.append(audio.decoder()),
                            _ => ()
                        }
                    } 
                }
            });
            Self {
                control_channel: Some(tx),
                sink: None,
            }
        } else {
            let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
            let sink = Sound::spatial_sink(&stream_handle, [-2.0, 1.5, -3.0], [-19.0, 5.0, -20.0], [-21.0, 5.0, -20.0]);
            Self {
                control_channel: None,
                sink: Some(sink),
            }
        }
    }

    pub fn play(&self, pos: Pos3, audio_source: Sound) {
        if cfg!(windows) {
            if let Some(channel) = &self.control_channel {         
                channel.send((true, pos.x, pos.y, pos.z, Some(audio_source))).unwrap();
                channel.send((false, pos.x, pos.y, pos.z, None)).unwrap();
            }
        } else {
            match &self.sink {
                    Some(s) => {
                        s.set_emitter_position([pos.x, pos.y, pos.z]);
                        s.append(audio_source.decoder());
                    },
                    
                    None => {}
                }
        }
    }
}