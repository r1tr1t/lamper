// todo:
// default device/selection

use libpulse_binding::{
    self,
    error::PAErr,
    sample::{Format, Spec},
    stream::Direction,
};
use libpulse_simple_binding::{self, Simple};
use std::sync::{
    mpsc::{SendError, Sender},
    Arc,
};

use crate::{LampErr, WINDOW};

impl From<PAErr> for LampErr {
    fn from(_: PAErr) -> Self {
        LampErr::PAErr
    }
}

impl<T> From<SendError<T>> for LampErr {
    fn from(_: SendError<T>) -> Self {
        LampErr::SendErr
    }
}

pub fn start(tx: Sender<Vec<f32>>, conn: Arc<bool>) -> Result<(), LampErr> {
    // interface specs
    let spec = Spec {
        format: Format::FLOAT32NE,
        channels: 1,
        rate: 44100,
    };
    assert!(spec.is_valid());

    // create libpulse-simple interface
    let s = Simple::new(
        None,
        "lamper",
        Direction::Record,
        Some("alsa_output.usb-BurrBrown_from_Texas_Instruments_USB_AUDIO_CODEC-00.analog-stereo.monitor"),
        "Lamper",
        &spec,
        None,
        None
    )?;

    // send data to colproc thread
    loop {
        if !*conn {
            return Ok(());
        }
        let mut data = Vec::with_capacity(WINDOW);
        for _ in 0..WINDOW {
            data.push(0.0)
        }

        s.read(&mut data)?;
        tx.send(data)?;
    }
}
