use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};

use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AsciiError {
    #[error("String must be ASCII characters")]
    NotAscii,
    #[error("Capacity exceeded")]
    CapacityExceeded(#[from] arrayvec::CapacityError),
}

#[derive( Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct AsciiStackString<const CAP: usize> {
    storage: ArrayVec<u8, CAP>,
}

impl<const CAP: usize> AsciiStackString<CAP> {
    pub fn new() -> Self {
        Self {
            storage: ArrayVec::new(),
        }
    }

    pub fn as_str(&self) -> &str {
        self
    }

    pub fn as_mut_str(&mut self) -> &mut str {
        self
    }
}

impl Display for AsciiStackString<16> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Debug for AsciiStackString<16> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const CAP: usize> TryFrom<String> for AsciiStackString<CAP> {
    type Error = AsciiError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut storage: ArrayVec<u8, CAP> = value.as_bytes().try_into()?;
        if storage.iter().all(|c| c.is_ascii()) {
            storage.truncate(
                storage.iter().position(|v| *v == b'\0').unwrap_or(CAP)
            );
            Ok(Self {
                storage,
            })
        } else {
            Err(AsciiError::NotAscii)
        }
    }
}

impl<const CAP: usize> From<AsciiStackString<CAP>> for String {
    fn from(value: AsciiStackString<CAP>) -> Self {
        value.as_str().to_string()
    }
}

impl<const CAP: usize> TryFrom<[u8; CAP]> for AsciiStackString<CAP> {
    type Error = AsciiError;

    fn try_from(value: [u8; CAP]) -> Result<Self, Self::Error> {
        if value.iter().all(|&b| b.is_ascii()) {
            let mut string = Self {
                storage: ArrayVec::from(value),
            };
            string.storage.truncate(
                string.storage.iter().position(|v| *v == b'\0').unwrap_or(CAP)
            );
            Ok(string)
        } else {
            Err(AsciiError::NotAscii)
        }
    }
}

impl<'a, const CAP: usize> From<&'a AsciiStackString<CAP>> for [u8; CAP] {
    fn from(value: &'a  AsciiStackString<CAP>) -> Self {
        let mut index = 0;
        [(); CAP].map(|_| {
            let byte = value.storage.get(index).copied().unwrap_or(0);
            index += 1;
            byte
        })
    }
}

impl<const CAP: usize> Deref for AsciiStackString<CAP> {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.storage.as_slice()) }
    }
}

impl<const CAP: usize> DerefMut for AsciiStackString<CAP> {
    fn deref_mut(&mut self) -> &mut str {
        unsafe { std::str::from_utf8_unchecked_mut(self.storage.as_mut_slice()) }
    }
}
