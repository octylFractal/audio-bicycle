use binrw::binrw;
use thiserror::Error;

use crate::asciistackstr::AsciiStackString;

#[derive(Debug, Error)]
pub enum VbanPacketError {
    #[error("Unknown sample rate: {0}")]
    UnknownSampleRate(u32),
    #[error("Unknown sample rate index: {0}")]
    UnknownSampleRateIndex(u8),
    #[error("Stream name must be 16 or less ASCII characters")]
    InvalidStreamName(String),
    #[error("Too many samples per frame")]
    TooManySamplesPerFrame,
    #[error("Too many channels")]
    TooManyChannels,
}

/// A VB-Audio Network packet.
#[derive(Debug, Clone)]
#[binrw]
#[brw(little)]
pub struct VbanPacket {
    /// The packet's header.
    pub header: VbanHeader,
    /// The packet's data.
    #[br(parse_with = binrw::helpers::until_eof)]
    pub data: Vec<u8>,
}

/// A VB-Audio Network packet header.
/// This is biased towards only really working with [SubProtocol::Audio].
#[binrw]
#[brw(little, magic = b"VBAN")]
#[derive(Debug, Clone)]
pub struct VbanHeader {
    #[br(temp)]
    #[bw(calc = u8::from(*sample_rate) | u8::from(*sub_protocol))]
    sr_sub_protocol: u8,
    #[bw(ignore)]
    #[br(calc = SampleRate::from(sr_sub_protocol))]
    pub sample_rate: SampleRate,
    #[bw(ignore)]
    #[br(
        calc = SubProtocol::from(sr_sub_protocol),
        assert(matches!(sub_protocol, SubProtocol::Audio))
    )]
    pub sub_protocol: SubProtocol,
    pub samples_per_frame: u8,
    pub channels: u8,
    #[br(temp)]
    #[bw(calc = u8::from(*data_type) | u8::from(*codec))]
    data_type_codec: u8,
    #[bw(ignore)]
    #[br(calc = DataType::from(data_type_codec))]
    pub data_type: DataType,
    #[bw(ignore)]
    #[br(calc = Codec::from(data_type_codec))]
    pub codec: Codec,
    #[br(try_map = <AsciiStackString::<16> as TryFrom<[u8; 16]>>::try_from)]
    #[bw(map = <[u8; 16]>::from)]
    pub stream_name: AsciiStackString<16>,
    pub frame_counter: u32,
}

impl VbanHeader {
    pub fn next(self) -> VbanHeader {
        VbanHeader {
            frame_counter: self.frame_counter + 1,
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SampleRate {
    Hz6000,
    Hz12000,
    Hz24000,
    Hz48000,
    Hz96000,
    Hz192000,
    Hz384000,
    Hz8000,
    Hz16000,
    Hz32000,
    Hz64000,
    Hz128000,
    Hz256000,
    Hz512000,
    Hz11025,
    Hz22050,
    Hz44100,
    Hz88200,
    Hz176400,
    Hz352800,
    Hz705600,
    Undefined(u8),
}

impl SampleRate {
    pub fn get_rate_if_known(&self) -> Option<u32> {
        match self {
            SampleRate::Hz6000 => Some(6000),
            SampleRate::Hz12000 => Some(12000),
            SampleRate::Hz24000 => Some(24000),
            SampleRate::Hz48000 => Some(48000),
            SampleRate::Hz96000 => Some(96000),
            SampleRate::Hz192000 => Some(192000),
            SampleRate::Hz384000 => Some(384000),
            SampleRate::Hz8000 => Some(8000),
            SampleRate::Hz16000 => Some(16000),
            SampleRate::Hz32000 => Some(32000),
            SampleRate::Hz64000 => Some(64000),
            SampleRate::Hz128000 => Some(128000),
            SampleRate::Hz256000 => Some(256000),
            SampleRate::Hz512000 => Some(512000),
            SampleRate::Hz11025 => Some(11025),
            SampleRate::Hz22050 => Some(22050),
            SampleRate::Hz44100 => Some(44100),
            SampleRate::Hz88200 => Some(88200),
            SampleRate::Hz176400 => Some(176400),
            SampleRate::Hz352800 => Some(352800),
            SampleRate::Hz705600 => Some(705600),
            SampleRate::Undefined(_) => None,
        }
    }
}

impl From<SampleRate> for u8 {
    fn from(sample_rate: SampleRate) -> Self {
        match sample_rate {
            SampleRate::Hz6000 => 0,
            SampleRate::Hz12000 => 1,
            SampleRate::Hz24000 => 2,
            SampleRate::Hz48000 => 3,
            SampleRate::Hz96000 => 4,
            SampleRate::Hz192000 => 5,
            SampleRate::Hz384000 => 6,
            SampleRate::Hz8000 => 7,
            SampleRate::Hz16000 => 8,
            SampleRate::Hz32000 => 9,
            SampleRate::Hz64000 => 10,
            SampleRate::Hz128000 => 11,
            SampleRate::Hz256000 => 12,
            SampleRate::Hz512000 => 13,
            SampleRate::Hz11025 => 14,
            SampleRate::Hz22050 => 15,
            SampleRate::Hz44100 => 16,
            SampleRate::Hz88200 => 17,
            SampleRate::Hz176400 => 18,
            SampleRate::Hz352800 => 19,
            SampleRate::Hz705600 => 20,
            SampleRate::Undefined(v) => v,
        }
    }
}

impl From<u8> for SampleRate {
    fn from(v: u8) -> Self {
        match v & 0b0001_1111 {
            0 => SampleRate::Hz6000,
            1 => SampleRate::Hz12000,
            2 => SampleRate::Hz24000,
            3 => SampleRate::Hz48000,
            4 => SampleRate::Hz96000,
            5 => SampleRate::Hz192000,
            6 => SampleRate::Hz384000,
            7 => SampleRate::Hz8000,
            8 => SampleRate::Hz16000,
            9 => SampleRate::Hz32000,
            10 => SampleRate::Hz64000,
            11 => SampleRate::Hz128000,
            12 => SampleRate::Hz256000,
            13 => SampleRate::Hz512000,
            14 => SampleRate::Hz11025,
            15 => SampleRate::Hz22050,
            16 => SampleRate::Hz44100,
            17 => SampleRate::Hz88200,
            18 => SampleRate::Hz176400,
            19 => SampleRate::Hz352800,
            20 => SampleRate::Hz705600,
            _ => SampleRate::Undefined(v),
        }
    }
}

impl TryFrom<u32> for SampleRate {
    type Error = VbanPacketError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            6000 => Ok(SampleRate::Hz6000),
            12000 => Ok(SampleRate::Hz12000),
            24000 => Ok(SampleRate::Hz24000),
            48000 => Ok(SampleRate::Hz48000),
            96000 => Ok(SampleRate::Hz96000),
            192000 => Ok(SampleRate::Hz192000),
            384000 => Ok(SampleRate::Hz384000),
            8000 => Ok(SampleRate::Hz8000),
            16000 => Ok(SampleRate::Hz16000),
            32000 => Ok(SampleRate::Hz32000),
            64000 => Ok(SampleRate::Hz64000),
            128000 => Ok(SampleRate::Hz128000),
            256000 => Ok(SampleRate::Hz256000),
            512000 => Ok(SampleRate::Hz512000),
            11025 => Ok(SampleRate::Hz11025),
            22050 => Ok(SampleRate::Hz22050),
            44100 => Ok(SampleRate::Hz44100),
            88200 => Ok(SampleRate::Hz88200),
            176400 => Ok(SampleRate::Hz176400),
            352800 => Ok(SampleRate::Hz352800),
            705600 => Ok(SampleRate::Hz705600),
            _ => Err(VbanPacketError::UnknownSampleRate(value)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SubProtocol {
    Audio = 0x00,
    Serial = 0x20,
    Txt = 0x40,
    Service = 0x60,
    Undefined1 = 0x80,
    Undefined2 = 0xA0,
    Undefined3 = 0xC0,
    User = 0xE0,
}

impl From<SubProtocol> for u8 {
    fn from(sub_protocol: SubProtocol) -> Self {
        sub_protocol as u8
    }
}

impl From<u8> for SubProtocol {
    fn from(v: u8) -> Self {
        match v & 0b1110_0000 {
            0x00 => SubProtocol::Audio,
            0x20 => SubProtocol::Serial,
            0x40 => SubProtocol::Txt,
            0x60 => SubProtocol::Service,
            0x80 => SubProtocol::Undefined1,
            0xA0 => SubProtocol::Undefined2,
            0xC0 => SubProtocol::Undefined3,
            0xE0 => SubProtocol::User,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DataType {
    U8,
    I16,
    I24,
    I32,
    F32,
    F64,
    I12,
    I10,
}

impl From<DataType> for u8 {
    fn from(data_type: DataType) -> Self {
        match data_type {
            DataType::U8 => 0,
            DataType::I16 => 1,
            DataType::I24 => 2,
            DataType::I32 => 3,
            DataType::F32 => 4,
            DataType::F64 => 5,
            DataType::I12 => 6,
            DataType::I10 => 7,
        }
    }
}

impl From<u8> for DataType {
    fn from(v: u8) -> Self {
        match v & 0b0000_0111 {
            0 => DataType::U8,
            1 => DataType::I16,
            2 => DataType::I24,
            3 => DataType::I32,
            4 => DataType::F32,
            5 => DataType::F64,
            6 => DataType::I12,
            7 => DataType::I10,
            _ => unreachable!(),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum Codec {
    PCM = 0x00,
    VBCA = 0x10,
    VBCV = 0x20,
    Undefined1 = 0x30,
    Undefined2 = 0x40,
    Undefined3 = 0x50,
    Undefined4 = 0x60,
    Undefined5 = 0x70,
    Undefined6 = 0x80,
    Undefined7 = 0x90,
    Undefined8 = 0xA0,
    Undefined9 = 0xB0,
    Undefined10 = 0xC0,
    Undefined11 = 0xD0,
    Undefined12 = 0xE0,
    User = 0xF0,
}

impl From<Codec> for u8 {
    fn from(codec: Codec) -> Self {
        codec as u8
    }
}

impl From<u8> for Codec {
    fn from(v: u8) -> Self {
        match v & 0b1111_0000 {
            0x00 => Codec::PCM,
            0x10 => Codec::VBCA,
            0x20 => Codec::VBCV,
            0x30 => Codec::Undefined1,
            0x40 => Codec::Undefined2,
            0x50 => Codec::Undefined3,
            0x60 => Codec::Undefined4,
            0x70 => Codec::Undefined5,
            0x80 => Codec::Undefined6,
            0x90 => Codec::Undefined7,
            0xA0 => Codec::Undefined8,
            0xB0 => Codec::Undefined9,
            0xC0 => Codec::Undefined10,
            0xD0 => Codec::Undefined11,
            0xE0 => Codec::Undefined12,
            0xF0 => Codec::User,
            _ => unreachable!(),
        }
    }
}
