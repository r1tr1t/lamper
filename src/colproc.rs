use std::sync::mpsc::{Sender, Receiver};
use dft::{Operation, Plan};

use crate::WINDOW;


pub fn process(rx: Receiver<Vec<f32>>) {
    loop{
        let mut rgb: [u8; 3] = [0; 3];
        let data = rx.recv().expect("failed to recieve from audproc");

        let mut freqs =dft(data);
        let mut top_freq = 0.0;
        let mut top_freq_vol = 0.0;

        for (i, volume) in freqs.iter().enumerate() {
            if i > 0 && i < freqs.len() / 2 && volume >= &top_freq_vol {
                top_freq = i as f32 * 44100 as f32 / freqs.len() as f32;
                top_freq_vol = *volume;
            }
        }
        println!("top: {} Hz at volume {}", top_freq, top_freq_vol);

        // rgb[0] = 
        // tx.send(rgb).expect("failed to send from colproc");
    }
}
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