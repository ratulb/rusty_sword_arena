use rodio::{Decoder, Device, Sink, source::Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor, Read};

/// A simple 4-channel audio system to play audio clips as sound effects.
pub struct Audio {
    endpoint: Device,
    clips: HashMap<&'static str, Vec<u8>>,
    channels: Vec<Sink>,
    current_channel: usize,
}

impl Audio {
    pub fn new() -> Self {
        let endpoint = rodio::default_output_device().unwrap();
        let clips = HashMap::new();
        let mut channels: Vec<Sink> = Vec::new();
        for _ in 0..8 {
            channels.push(Sink::new(&endpoint))
        }
        Self {
            endpoint,
            clips,
            channels,
            current_channel: 0,
        }
    }
    pub fn add(&mut self, name: &'static str, path: &str) {
        let mut file_vec: Vec<u8> = Vec::new();
        File::open(path)
            .expect("Couldn't find audio file to add.")
            .read_to_end(&mut file_vec)
            .expect("Failed reading in opened audio file.");
        self.clips.insert(name, file_vec);
    }
    pub fn play(&mut self, name: &str) {
        let clip = self
            .clips
            .get(name)
            .expect("We don't have that audio clip.")
            .clone();
        let cursor = Cursor::new(clip);
        let decoder = Decoder::new(BufReader::new(cursor)).unwrap();
        self.channels[self.current_channel].append(decoder);
        self.current_channel += 1;
        if self.current_channel >= self.channels.len() {
            self.current_channel = 0;
        }
    }
}
