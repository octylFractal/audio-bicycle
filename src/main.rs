use std::process::ExitCode;
use std::process::Termination;
use std::sync::Arc;

use clap::Parser;
use futures::{select, FutureExt};
use libpulse_binding::error::PAErr;
use libpulse_binding::sample::{Format, Spec};
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;
use log::LevelFilter;
use thiserror::Error;
use tokio::net::UdpSocket;

use crate::config::global::{load_config, ConfigError};
use crate::vban::receiver::ReceiverError;

mod asciistackstr;
mod config;
mod vban;

/// Service designed to run on systemd to connect to a VBAN stream pair for mic and sound output.
#[derive(Parser)]
pub struct AudioBicycle {
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, Error)]
enum AudioBicycleError {
    #[error("Couldn't load config: {0}")]
    Config(#[from] ConfigError),
    #[error("PulseAudio error: {0}")]
    PulseAudio(#[from] PAErr),
    #[error("Couldn't create socket: {0}")]
    Socket(#[from] std::io::Error),
    #[error("Couldn't receive from socket: {0}")]
    SocketReceive(#[from] ReceiverError),
}

impl Termination for AudioBicycleError {
    fn report(self) -> ExitCode {
        // Might split this up later.
        ExitCode::FAILURE
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: AudioBicycle = AudioBicycle::parse();
    env_logger::Builder::new()
        .filter_level(match args.verbose {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        })
        .init();

    match main_for_result(args).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            log::error!("{:#}", e);
            e.report()
        }
    }
}

async fn main_for_result(_: AudioBicycle) -> Result<(), AudioBicycleError> {
    let config = load_config()?;

    let socket = UdpSocket::bind(config.local_address).await?;
    let socket = Arc::new(socket);

    let (pa_send, mut pa_recv) = tokio::sync::mpsc::channel::<Vec<u8>>(10);

    let spec = Spec {
        format: Format::S24le,
        channels: 2,
        rate: 48000,
    };
    assert!(spec.is_valid());

    let s = Simple::new(
        None,                // Use the default server
        "Audio Bicycle",     // Our application’s name
        Direction::Playback, // We want a playback stream
        None,                // Use the default device
        "VBAN Output",       // Description of our stream
        &spec,               // Our sample format
        None,                // Use default channel map
        None,                // Use default buffering attributes
    )
    .unwrap();

    let mut pa_thread = tokio::task::spawn(async move {
        while let Some(buffer) = pa_recv.recv().await {
            s.write(&buffer)?;
        }
        Ok::<_, AudioBicycleError>(())
    })
    .fuse();
    let receiver = vban::receiver::Receiver {
        stream_name: config.stream_name.clone(),
        recv_address: config.dest_address.ip(),
        pw_out: pa_send,
        socket,
    };
    let mut receiver_thread = tokio::task::spawn(receiver.run()).fuse();

    select! {
        pa_result = pa_thread => pa_result.expect("task panicked")?,
        receiver_result = receiver_thread => receiver_result.expect("task panicked")?,
    }

    Ok(())
}
