use super::metadata::MetadataError;

#[derive(Debug, PartialEq)]
pub enum Checksum {
    SHA224(String),
    SHA256(String),
    SHA384(String),
    SHA512(String),
    None,
}

impl Checksum {
    pub fn try_create(checksum_type: &[u8], checksum: &[u8]) -> Result<Self, MetadataError> {
        let get_checksum_value = |value| std::str::from_utf8(value).unwrap().to_owned();

        let checksum = match checksum_type {
            b"sha224" => Checksum::SHA224(get_checksum_value(checksum)),
            b"sha256" => Checksum::SHA256(get_checksum_value(checksum)),
            b"sha384" => Checksum::SHA384(get_checksum_value(checksum)),
            b"sha512" => Checksum::SHA512(get_checksum_value(checksum)),
            _ => {
                return Err(MetadataError::UnsupportedChecksumTypeError(
                    get_checksum_value(checksum_type),
                ))
            }
        };
        Ok(checksum)
    }

    pub fn to_values<'a>(&'a self) -> (&str, &'a str) {
        match self {
            Checksum::SHA224(c) => ("sha224", c.as_str()),
            Checksum::SHA256(c) => ("sha256", c.as_str()),
            Checksum::SHA384(c) => ("sha384", c.as_str()),
            Checksum::SHA512(c) => ("sha512", c.as_str()),
            _ => unreachable!(),
        }
    }
}

impl Default for Checksum {
    fn default() -> Self {
        Checksum::None
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct EVR {
    epoch: String,
    version: String, // ver
    release: String, //rel
}

impl EVR {
    pub fn new(epoch: &str, version: &str, release: &str) -> EVR {
        EVR {
            epoch: epoch.to_owned(),
            version: version.to_owned(),
            release: release.to_owned(),
        }
    }

    pub fn values(&self) -> (&str, &str, &str) {
        (&self.epoch, &self.version, &self.release)
    }
}
