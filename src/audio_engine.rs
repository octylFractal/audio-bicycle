use crate::vban::transmitter::USABLE_DATA_PACKET_SIZE;
use futures::future::FutureExt;
use futures::select;
use libpulse_binding::def::BufferAttr;
use libpulse_binding::error::PAErr;
use libpulse_binding::sample::{Format, Spec};
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;
use once_cell::sync::Lazy;

static SPEC: Lazy<Spec> = Lazy::new(|| {
    let spec = Spec {
        format: Format::S24le,
        channels: 2,
        rate: 48000,
    };
    assert!(spec.is_valid());
    spec
});

pub async fn run(
    mut pa_recv: tokio::sync::mpsc::Receiver<Vec<u8>>,
    pa_send: tokio::sync::mpsc::Sender<Vec<u8>>,
) -> Result<(), PAErr> {
    let mut output_task = tokio::spawn(async move {
        let s = Simple::new(
            None,                // Use the default server
            "Audio Bicycle",     // Our application’s name
            Direction::Playback, // We want a playback stream
            None,                // Use the default device
            "VBAN Output",       // Description of our stream
            &SPEC,               // Our sample format
            None,                // Use default channel map
            None,                // Use default buffering attributes
        )?;

        while let Some(buffer) = pa_recv.recv().await {
            tokio::task::block_in_place(|| s.write(&buffer))?;
        }
        Ok::<_, PAErr>(())
    })
    .fuse();
    let mut input_task = tokio::spawn(async move {
        let s = Simple::new(
            None,
            "Audio Bicycle",
            Direction::Record,
            None,
            "VBAN Input",
            &SPEC,
            None,
            Some(&BufferAttr {
                maxlength: (USABLE_DATA_PACKET_SIZE * 4),
                fragsize: (USABLE_DATA_PACKET_SIZE),
                ..Default::default()
            }),
        )?;

        let mut buffer = vec![0u8; USABLE_DATA_PACKET_SIZE as usize];
        loop {
            tokio::task::block_in_place(|| s.read(&mut buffer))?;
            if (pa_send.send(buffer.clone()).await).is_err() {
                break;
            }
        }
        Ok::<_, PAErr>(())
    })
    .fuse();

    loop {
        (select! {
            output_result = output_task => output_result,
            input_result = input_task => input_result,
            complete => break,
        })
        .expect("task panicked")?;
    }

    Ok(())
}
