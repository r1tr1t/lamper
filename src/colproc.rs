// todo:
// fix normalization
// two modes, one with hz -> color and another with hz -> brightness

use dft::{Operation, Plan};
use std::sync::{
    mpsc::{Receiver, RecvError, Sender},
    Arc, RwLock,
};

use crate::{LampErr, WINDOW};

// frequency range
const MAX_FREQUENCY: f32 = 20000.0;
const MIN_FREQUENCY: f32 = 20.0;
const BRIGHTNORMCAPACITY: usize = 50;

impl From<RecvError> for LampErr {
    fn from(_: RecvError) -> Self {
        LampErr::SendErr
    }
}

// brightness normalization
struct BrightNorm {
    vec: Vec<f32>,
    max: f32,
}
impl BrightNorm {
    fn new() -> Self {
        BrightNorm {
            vec: Vec::with_capacity(BRIGHTNORMCAPACITY),
            max: 0.0,
        }
    }
    fn norm(&mut self, vol: f32) -> u8 {
        if self.vec.len() < BRIGHTNORMCAPACITY {
            self.vec.push(vol);
        } else {
            self.max = *self.vec.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
            self.vec = Vec::with_capacity(BRIGHTNORMCAPACITY);
        }

        if vol >= self.max {
            100u8
        } else {
            (((vol / self.max) * 0.9) * 100.0) as u8
        }
    }
}

// main fn to process audio into brightness and rgb
pub fn process(
    rx: Receiver<Vec<f32>>,
    tx: Sender<(u8, [u8; 3])>,
    conn: Arc<RwLock<bool>>,
) -> Result<(), LampErr> {
    let mut bright_norm = BrightNorm::new();
    loop {
        if !*conn.read().unwrap() {
            return Ok(());
        }

        let data = rx.recv()?;
        let freqs = dft(data);
        let mut top_freq = 0.0;
        let mut top_freq_vol = 0.0;

        for (i, volume) in freqs.iter().enumerate() {
            if i > 0 && i < freqs.len() / 2 && volume >= &top_freq_vol {
                top_freq = i as f32 * 44100_f32 / freqs.len() as f32;
                top_freq_vol = *volume;
            }
        }

        let brightness = bright_norm.norm(top_freq_vol);
        let rgb = rgb(top_freq);

        tx.send((brightness, rgb))?;
        println!(
            "top freq: {}, top freq vol: {}, max: {}, brightness: {}",
            top_freq, top_freq_vol, bright_norm.max, brightness
        );
    }
}

// converts the dominant frequency of each frame to hsl then rgb
fn rgb(hz: f32) -> [u8; 3] {
    let hue = (hz - MIN_FREQUENCY) / (MAX_FREQUENCY - MIN_FREQUENCY) * 360.0;
    let saturation: f32 = 1.0;
    let lightness: f32 = 0.5;

    let hsl_to_rgb = |h: f32, s: f32, l: f32| {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        if h < 60.0 {
            [c + m, x + m, m]
        } else if h < 120.0 {
            [x + m, c + m, m]
        } else if h < 180.0 {
            [m, c + m, x + m]
        } else if h < 240.0 {
            [m, x + m, c + m]
        } else if h < 300.0 {
            [x + m, m, c + m]
        } else {
            [c + m, m, x + m]
        }
    };

    let rgb_f = hsl_to_rgb(hue, saturation, lightness);
    let mut rgb: [u8; 3] = [0u8; 3];
    let max = u8::MAX as f32;

    for (count, i) in rgb_f.iter().enumerate() {
        rgb[count] = (i * max) as u8;
    }

    rgb
}

// perform dft on each frame
fn dft(data: Vec<f32>) -> Vec<f32> {
    let plan = Plan::new(Operation::Forward, WINDOW);

    let mut input = Vec::with_capacity(WINDOW);
    for x in data {
        input.push(x as f64);
    }

    dft::transform(&mut input, &plan);
    let output = dft::unpack(&input);
    let mut result = Vec::with_capacity(WINDOW);
    for ref c in output {
        result.push(c.norm() as f32);
    }
    result
}
