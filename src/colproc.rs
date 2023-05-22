// todo: fix normalization

use std::{sync::mpsc::{Sender, Receiver}, collections::VecDeque};
use dft::{Operation, Plan};

use crate::WINDOW;

// frequency range
const MAX_FREQUENCY: f32 = 20000.0;
const MIN_FREQUENCY: f32 = 20.0;


// main fn to process audio into brightness and rgb
pub fn process(rx: Receiver<Vec<f32>>, tx: Sender<(u8, [u8; 3])>, conn: &bool) {
    let norm_buf_size: usize = 1000;
    let norm_damper: f32 = 0.9;
    let mut norm_buf = VecDeque::with_capacity(norm_buf_size);
    let mut normalize = |vol: &f32| {
        if norm_buf.len() < norm_buf_size {
            norm_buf.push_back(*vol);
        } else {
            norm_buf.pop_front();
            norm_buf.push_back(*vol);
        }
        let average: f32 = norm_buf.iter().sum::<f32>() / norm_buf_size as f32;
        average * norm_damper
    };
    while *conn{
        let data = rx.recv().expect("failed to recieve from audproc");

        let freqs =dft(data);
        let mut top_freq = 0.0;
        let mut top_freq_vol = 0.0;

        for (i, volume) in freqs.iter().enumerate() {
            if i > 0 && i < freqs.len() / 2 && volume >= &top_freq_vol {
                top_freq = i as f32 * 44100 as f32 / freqs.len() as f32;
                top_freq_vol = *volume;
            }
        }
        
        let norm = normalize(&top_freq_vol);
        let brightness = brightness(top_freq_vol, norm);

        let rgb = rgb(top_freq);

        tx.send((brightness, rgb)).expect("couldn't send from colproc");
        println!("top freq: {}, top freq vol: {}, norm: {} brightness: {}", top_freq, top_freq_vol, norm, brightness);
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

        if h < 60.0 {
            [c, x, 0.0]
        } else if h < 120.0 {
            [x, c, 0.0]
        } else if h < 180.0 {
            [0.0, c, x]
        } else if h < 240.0 {
            [0.0, x, c]
        } else if h < 300.0 {
            [x, 0.0, c]
        } else {
            [c, 0.0, x]
        }
    };

    let rgb_f = hsl_to_rgb(hue, saturation, lightness);
    let mut rgb: [u8; 3] = [0u8; 3];
    let max = u8::MAX as f32;
    let mut count = 0;
    for i in rgb_f {
        rgb[count] = (i * max) as u8;
        count += 1;
    }

    rgb

}

// convert volume of each frame to 1-100 brightness range
fn brightness(vol: f32, norm: f32) -> u8 {
    if vol >= norm {
        return 100
    } else {
        return (vol/norm) as u8 * 100
    }
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