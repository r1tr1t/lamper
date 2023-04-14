use std::sync::mpsc::Sender;
use libpulse_binding::{self, sample::{Spec, Format}, stream::Direction};
use libpulse_simple_binding::{self, Simple};

use crate::WINDOW;

pub fn start(tx: Sender<Vec<f32>>) {
    let spec = Spec {
        format: Format::FLOAT32NE,
        channels: 1,
        rate: 44100 
    };
    assert!(spec.is_valid());

    let s = Simple::new(
        None,
        "lamper",
        Direction::Record,
        Some("alsa_output.usb-BurrBrown_from_Texas_Instruments_USB_AUDIO_CODEC-00.analog-stereo.monitor"),
        "Lamper",
        &spec,
        None,
        None
    ).expect("error connecting to server");
    

    loop {
        let mut data = Vec::with_capacity(WINDOW);
        for _ in 0..WINDOW {
            data.push(0.0)
        }

        s.read(&mut data).expect("error reading audio data");
        tx.send(data).expect("couldn't send data from audproc");
    }
}