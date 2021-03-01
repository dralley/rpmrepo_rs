use niffler;
use quick_xml;
use quick_xml::{Reader, Writer};

use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Write};
use std::path::Path;
use thiserror::Error;

fn configure_reader<R: BufRead>(reader: &mut Reader<R>) {
    reader.expand_empty_elements(true).trim_text(true);
}

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error(transparent)]
    MetadataParseError(#[from] quick_xml::Error),
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    IntFieldParseError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    UnsupportedCompressionTypeError(#[from] niffler::Error),
    #[error("Checksum type {0} is not supported")]
    UnsupportedChecksumTypeError(String),
}

/// Default namespace for primary.xml
pub const XML_NS_COMMON: &str = "http://linux.duke.edu/metadata/common";
/// Default namespace for filelists.xml
pub const XML_NS_FILELISTS: &str = "http://linux.duke.edu/metadata/filelists";
/// Default namespace for other.xml
pub const XML_NS_OTHER: &str = "http://linux.duke.edu/metadata/other";
/// Default namespace for repomd.xml
pub const XML_NS_REPO: &str = "http://linux.duke.edu/metadata/repo";
/// Namespace for rpm (used in primary.xml and repomd.xml)
pub const XML_NS_RPM: &str = "http://linux.duke.edu/metadata/rpm";

pub trait MetadataIO {
    fn from_file(path: &Path) -> Result<Self, MetadataError>
    where
        Self: Sized;
    fn from_str(str: &str) -> Result<Self, MetadataError>
    where
        Self: Sized;
    fn from_bytes(bytes: &[u8]) -> Result<Self, MetadataError>
    where
        Self: Sized;

    fn to_file(&self, path: &Path) -> Result<(), MetadataError>;
    fn to_string(&self) -> Result<String, MetadataError>;
    fn to_bytes(&self) -> Result<Vec<u8>, MetadataError>;
}

impl<T: RpmMetadata> MetadataIO for T {
    fn from_file(path: &Path) -> Result<Self, MetadataError>
    where
        Self: Sized,
    {
        let file = File::open(path)?;
        let (reader, _compression) = niffler::get_reader(Box::new(&file))?;
        let mut reader = Reader::from_reader(BufReader::new(reader));
        configure_reader(&mut reader);

        Self::deserialize(&mut reader)
    }

    fn from_str(str: &str) -> Result<Self, MetadataError>
    where
        Self: Sized,
    {
        let mut reader = Reader::from_str(str);
        configure_reader(&mut reader);

        Self::deserialize(&mut reader)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MetadataError>
    where
        Self: Sized,
    {
        let mut reader = Reader::from_reader(bytes);
        configure_reader(&mut reader);

        Self::deserialize(&mut reader)
    }

    // fn from_bytes_encoded(bytes: &[u8], encoding: ???) -> Result<Self, MetadataParseError>
    // where
    //     Self: Sized,
    // {
    //     let mut reader = Reader::from_reader(bytes);
    //     configure_reader(&mut reader);

    //     Self::deserialize(&mut reader)
    // }

    fn to_file(&self, path: &Path) -> Result<(), MetadataError>
    where
        Self: Sized,
    {
        let file = File::create(path)?;
        let mut writer = Writer::new(file);
        self.serialize(&mut writer)?;
        Ok(())
    }

    fn to_string(&self) -> Result<String, MetadataError>
    where
        Self: Sized,
    {
        let bytes = self.to_bytes()?;
        Ok(String::from_utf8(bytes).map_err(|e| e.utf8_error())?)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, MetadataError>
    where
        Self: Sized,
    {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(Cursor::new(&mut buf), b' ', 2);
        self.serialize(&mut writer)?;
        Ok(writer.into_inner().into_inner().to_vec())
    }
}

pub trait RpmMetadata {
    fn deserialize<R: BufRead>(reader: &mut Reader<R>) -> Result<Self, MetadataError>
    where
        Self: Sized;

    fn serialize<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), MetadataError>;
}

// TODO: Trait impl tests https://github.com/rust-lang/rfcs/issues/616
