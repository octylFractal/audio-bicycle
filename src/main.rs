use std::process::ExitCode;
use std::process::Termination;
use std::sync::Arc;

use clap::Parser;
use futures::{FutureExt, select};
use libpulse_binding::error::PAErr;
use log::LevelFilter;
use thiserror::Error;
use tokio::net::UdpSocket;

use crate::config::global::{ConfigError, load_config};
use crate::vban::receiver::ReceiverError;
use crate::vban::transmitter::TransmitterError;

mod asciistackstr;
mod config;
mod vban;
mod audio_engine;

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
    #[error("Couldn't receive audio: {0}")]
    Receiver(#[from] ReceiverError),
    #[error("Couldn't transmit audio: {0}")]
    Transmitter(#[from] TransmitterError),
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

    let (pa_out_send, pa_out_recv) = tokio::sync::mpsc::channel::<Vec<u8>>(10);
    let (pa_in_send, pa_in_recv) = tokio::sync::mpsc::channel::<Vec<u8>>(10);

    let mut pa_thread = tokio::task::spawn(audio_engine::run(pa_out_recv, pa_in_send)).fuse();
    let receiver = vban::receiver::Receiver {
        stream_name: config.stream_name.clone(),
        recv_address: config.dest_address.ip(),
        audio_out: pa_out_send,
        socket: Arc::clone(&socket),
    };
    let mut receiver_thread = tokio::task::spawn(receiver.run()).fuse();
    let transmitter = vban::transmitter::Transmitter {
        stream_name: config.stream_name.clone(),
        dest_address: config.dest_address,
        audio_in: pa_in_recv,
        socket,
    };
    let mut transmitter_thread = tokio::task::spawn(transmitter.run()).fuse();

    loop {
        select! {
            pa_result = pa_thread => pa_result.expect("task panicked")?,
            receiver_result = receiver_thread => receiver_result.expect("task panicked")?,
            transmitter_result = transmitter_thread => transmitter_result.expect("task panicked")?,
            complete => break,
        }
    }

    Ok(())
}
