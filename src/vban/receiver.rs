use std::io::Cursor;
use std::net::IpAddr;
use std::sync::Arc;

use binrw::BinReaderExt;
use thiserror::Error;
use tokio::net::UdpSocket;
use crate::asciistackstr::AsciiStackString;

use crate::vban::packet::{DataType, VbanPacket,Codec,SampleRate};

#[derive(Debug, Error)]
pub enum ReceiverError {
    #[error("Socket read error: {0}")]
    SocketRead(#[from] std::io::Error),
    #[error("Audio channel could not receive")]
    AudioChannelBroken,
}

pub struct Receiver {
    pub stream_name: AsciiStackString<16>,
    pub recv_address: IpAddr,
    pub audio_out: tokio::sync::mpsc::Sender<Vec<u8>>,
    pub socket: Arc<UdpSocket>,
}

impl Receiver {
    pub async fn run(self) -> Result<(), ReceiverError> {
        let mut buf = [0u8; 1464];
        loop {
            let (len, addr) = self.socket.recv_from(&mut buf).await?;
            if addr.ip() != self.recv_address {
                continue;
            }
            if len < 4 || &buf[..4] != b"VBAN" {
                log::warn!("Received obviously invalid packet, discarding");
                continue;
            }
            let decoded: VbanPacket = match Cursor::new(&mut buf[..len]).read_le() {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Failed to decode packet: {}", e);
                    continue;
                }
            };
            if decoded.header.stream_name != self.stream_name {
                continue;
            }
            assert!(matches!(decoded.header.data_type, DataType::I24));
            assert!(matches!(decoded.header.codec, Codec::PCM));
            assert!(matches!(decoded.header.sample_rate, SampleRate::Hz48000));
            assert!(matches!(decoded.header.channels, 1)); // Meaning 2... :)
            self.audio_out.send(decoded.data).await.map_err(|_| ReceiverError::AudioChannelBroken)?;
        }
    }
}
