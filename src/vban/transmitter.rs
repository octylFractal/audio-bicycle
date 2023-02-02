use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;

use binrw::BinWriterExt;
use thiserror::Error;
use tokio::net::UdpSocket;

use crate::asciistackstr::AsciiStackString;
use crate::vban::packet::{Codec, DataType, SampleRate, SubProtocol, VbanHeader, VbanPacket};

#[derive(Debug, Error)]
pub enum TransmitterError {
    #[error("Socket write error: {0}")]
    SocketWrite(#[from] std::io::Error),
}

pub struct Transmitter {
    pub stream_name: AsciiStackString<16>,
    pub dest_address: SocketAddr,
    pub audio_in: tokio::sync::mpsc::Receiver<Vec<u8>>,
    pub socket: Arc<UdpSocket>,
}


const SAMPLE_SIZE: u32 = 2 * 3;
const MAX_DATA_PACKET_SIZE: u32 = 1436;

const fn samples_per_packet() -> u32 {
    let samples_per_packet = MAX_DATA_PACKET_SIZE / SAMPLE_SIZE;
    if samples_per_packet <= 256 {
        samples_per_packet
    } else {
        256
    }
}

const SAMPLES_PER_PACKET: u32 = samples_per_packet();
pub const USABLE_DATA_PACKET_SIZE: u32 = SAMPLES_PER_PACKET * SAMPLE_SIZE;

impl Transmitter {
    pub async fn run(mut self) -> Result<(), TransmitterError> {
        let mut header = VbanHeader {
            sample_rate: SampleRate::Hz48000,
            sub_protocol: SubProtocol::Audio,
            samples_per_frame: (SAMPLES_PER_PACKET - 1) as u8,
            channels: 1, // meaning 2 ...
            data_type: DataType::I24,
            codec: Codec::PCM,
            stream_name: self.stream_name,
            frame_counter: 0,
        };
        let mut buf = Vec::new();
        while let Some(audio_packet) = self.audio_in.recv().await {
            let packet = VbanPacket {
                header: header.clone(),
                data: audio_packet,
            };
            buf.clear();
            Cursor::new(&mut buf).write_le(&packet)
                .expect("should always be able to write to a Vec");
            let sent = self.socket.send_to(&buf, self.dest_address).await?;
            assert_eq!(sent, buf.len(), "should always send the whole packet");
            header = header.next();
        }

        Ok(())
    }
}
