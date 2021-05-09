use std::{sync::mpsc::{self, Sender, Receiver}};
use std::thread;

use rodio::{SpatialSink};

use crate::geom::{Pos3};
use crate::music::Sound;

pub struct Audio {
    control_channel: Sender<(bool, f32, f32, f32, Option<Sound>)>
}

impl Audio {
    pub fn new() -> Self {
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
            control_channel: tx
        }
    }

    pub fn play(&self, pos: Pos3, audio_source: Sound) {
        self.control_channel.send((true, pos.x, pos.y, pos.z, Some(audio_source))).unwrap();
        self.control_channel.send((false, pos.x, pos.y, pos.z, None)).unwrap();
    }
}