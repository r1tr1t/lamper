// todo:
// fix normalization
// two modes, one with hz -> color and another with hz -> brightness

use dft::{Operation, Plan};
use rand::{self, Rng};
use std::sync::{
    mpsc::{Receiver, RecvError, Sender},
    Arc, RwLock,
};

use crate::{LampErr, WINDOW};

// frequency range
const MAX_FREQUENCY: f32 = 20000.0;
const MIN_FREQUENCY: f32 = 20.0;

impl From<RecvError> for LampErr {
    fn from(_: RecvError) -> Self {
        LampErr::SendErr
    }
}

// brightness normalization
struct BrightNorm {
    default: f32,
    max: f32,
    reset: u8,
    reset_end: u8,
}
impl BrightNorm {
    fn new() -> Self {
        BrightNorm {
            default: 800.0,
            max: 0.0,
            reset: 0,
            reset_end: 50,
        }
    }
    // add conditions for reset_end
    fn norm(&mut self, vol: f32) -> u8 {
        if self.max != 0.0 && self.reset >= self.reset_end && vol < self.default {
            self.max = 0.0;
            self.reset = 0;
            ((vol / self.default) * 100.0) as u8
        } else if self.max != 0.0 && (vol > self.max || vol > self.default) {
            self.max = vol;
            self.reset = 0;
            100u8
        } else if self.max != 0.0 {
            self.reset += 1;
            ((vol / self.max) * 100.0) as u8
        } else if vol > self.default {
            self.max = vol;
            self.reset = 0;
            100u8
        } else {
            self.reset = 0;
            ((vol / self.default) * 100.0) as u8
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

pub enum Cycle {
    Brightness(u8),
    Color([u8; 3]),
}

pub fn process_cycle(
    rx: Receiver<Vec<f32>>,
    tx: Sender<Cycle>,
    conn: Arc<RwLock<bool>>,
) -> Result<(), LampErr> {
    let (mut color, mut color_index) = cycle_color(None);
    let mut cycle_count: u8 = 0;
    let cycle_end: u8 = 255;
    let mut bright_norm = BrightNorm::new();
    loop {
        if !*conn.read().unwrap() {
            return Ok(());
        }
        match cycle_count {
            0 => {
                rx.recv()?;
                tx.send(Cycle::Color(color))?;
                cycle_count += 1
            }
            v if v < cycle_end => {
                let data = rx.recv()?;
                let vol = data.iter().sum::<f32>() / data.len() as f32;
                let brightness = bright_norm.norm(vol);
                tx.send(Cycle::Brightness(brightness))?;
            }
            _ => {
                rx.recv()?;
                (color, color_index) = cycle_color(Some(color_index));
                tx.send(Cycle::Color(color))?;
            }
        }
    }
}

fn cycle_color(prev: Option<usize>) -> ([u8; 3], usize) {
    let cycle: [[u8; 3]; 7] = [
        [255, 0, 0],
        [255, 0, 213],
        [94, 0, 255],
        [0, 26, 255],
        [0, 213, 255],
        [26, 255, 0],
        [255, 111, 0],
    ];
    let index = match prev {
        Some(val) => {
            let mut rand = rand::thread_rng().gen_range(0..cycle.len());
            while val == rand {
                rand = rand::thread_rng().gen_range(0..cycle.len());
            }
            rand
        }
        None => rand::thread_rng().gen_range(0..cycle.len()),
    };
    (cycle[index], index)
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
