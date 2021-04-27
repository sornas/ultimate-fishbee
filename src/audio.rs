use crate::asset::{self, audio::Samples};

use luminance_sdl2::sdl2::Sdl;
use luminance_sdl2::sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

pub const SAMPLE_RATE: i32 = 48000;

pub struct AudioSource {
    position: usize,
    samples: Samples,
}

impl AudioSource {
    pub fn new(samples: Samples) -> Self {
        Self {
            position: 0,
            samples,
        }
    }
}

pub struct Audio {
    sources: Vec<AudioSource>,
}

impl Audio {
    pub fn init(sdl: &Sdl) -> AudioDevice<Self> {
        let audio_subsystem = sdl.audio().unwrap();
        let desired = AudioSpecDesired {
            freq: Some(SAMPLE_RATE),
            channels: Some(2),
            samples: None,
        };

        audio_subsystem.open_playback(None, &desired, |spec| {
            assert_eq!(spec.freq, SAMPLE_RATE); //TODO handle differing sample rates gracefully
            Self {
                sources: Vec::new(),
            }
        }).unwrap()
    }

    pub fn play(&mut self, audio: &asset::Audio) {
        self.sources.push(AudioSource {
            position: 0,
            samples: audio.samples(),
        });
    }
}

impl AudioCallback for Audio {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            *x = 0.0;
        }

        for source in self.sources.iter_mut() {
            let samples = source.samples.read().unwrap();
            for x in out.iter_mut() {
                source.position += 1;
                if source.position >= samples.len() {
                    source.position = 0;
                }
                *x += samples[source.position];
            }
        }
    }
}
