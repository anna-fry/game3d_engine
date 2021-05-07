// GitHub User sinesc
// https://github.com/RustAudio/rodio/issues/141#issuecomment-383371609

use rodio::{self, OutputStreamHandle, Sink};
use std::convert::AsRef;
use std::{
    io::{self, Read},
    sync::Arc,
};
#[derive(Clone)]
pub struct Sound(Arc<Vec<u8>>);

impl AsRef<[u8]> for Sound {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Sound {
    pub fn load(filename: &str) -> io::Result<Sound> {
        use std::fs::File;
        let mut buf = Vec::new();
        let mut file = File::open(filename)?;
        file.read_to_end(&mut buf)?;
        Ok(Sound(Arc::new(buf)))
    }
    pub fn cursor(self: &Self) -> io::Cursor<Sound> {
        io::Cursor::new(Sound(self.0.clone()))
    }
    // Decoder implements Source!
    pub fn decoder(self: &Self) -> rodio::Decoder<io::Cursor<Sound>> {
        rodio::Decoder::new(self.cursor()).unwrap()
    }

    pub fn sink(stream_handle: &OutputStreamHandle) -> Sink {
        rodio::Sink::try_new(&stream_handle).unwrap()
    }
}
