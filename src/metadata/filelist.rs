use quick_xml::events::{BytesDecl, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::io::{BufRead, Write};

use super::common::EVR;
use super::metadata::{RpmMetadata, XML_NS_FILELISTS};
use super::MetadataError;

const TAG_FILELISTS: &[u8] = b"filelists";
const TAG_PACKAGE: &[u8] = b"package";
const TAG_VERSION: &[u8] = b"version";
const TAG_FILE: &[u8] = b"file";

#[derive(Debug, PartialEq, Default)]
pub struct Filelist {
    pub packages: Vec<Package>,
}

#[derive(Debug, PartialEq, Default)]
pub struct Package {
    pkgid: String,
    name: String,
    arch: String,
    version: EVR,
    files: Vec<PackageFile>,
}

#[derive(Debug, PartialEq)]
pub enum FileType {
    File,
    Dir,
    Ghost,
}

impl FileType {
    fn try_create(val: &[u8]) -> Result<Self, MetadataError> {
        let ftype = match val {
            b"dir" => FileType::Dir,
            b"ghost" => FileType::Ghost,
            b"file" => FileType::File,
            _ => panic!(),
        };
        Ok(ftype)
    }

    pub fn to_values(&self) -> &[u8] {
        match self {
            FileType::File => b"file",
            FileType::Dir => b"dir",
            FileType::Ghost => b"ghost",
        }
    }
}

impl Default for FileType {
    fn default() -> Self {
        FileType::File
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct PackageFile {
    filetype: Option<FileType>,
    path: String,
}

impl RpmMetadata for Filelist {
    // <?xml version="1.0" encoding="UTF-8"?>
    // <filelists xmlns="http://linux.duke.edu/metadata/filelists" packages="1">
    //   <package pkgid="a2d3bce512f79b0bc840ca7912a86bbc0016cf06d5c363ffbb6fd5e1ef03de1b" name="fontconfig" arch="x86_64">
    //     <version epoch="0" ver="2.8.0" rel="5.fc33"/>
    //     <file type="dir">/etc/fonts/conf.avail</file>
    //     ...
    //     <file>/etc/fonts/conf.avail/10-autohint.conf</file>
    //   </package>
    // </filelists>

    fn deserialize<R: BufRead>(reader: &mut Reader<R>) -> Result<Filelist, MetadataError> {
        let mut filelist = Filelist::default();
        let mut buf = Vec::new();

        let mut found_metadata_tag = false;

        loop {
            match reader.read_event(&mut buf)? {
                Event::Start(e) => match e.name() {
                    TAG_FILELISTS => {
                        found_metadata_tag = true;
                    }
                    TAG_PACKAGE => {
                        let package = parse_package(reader, &e)?;
                        filelist.packages.push(package);
                    }
                    _ => (),
                },
                Event::Eof => break,
                Event::Decl(_) => (), // TOOD
                _ => (),
            }
        }
        if !found_metadata_tag {
            // TODO
        }
        Ok(filelist)
    }

    fn serialize<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), MetadataError> {
        // <?xml version="1.0" encoding="UTF-8"?>
        writer.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;

        // <filelists xmlns="http://linux.duke.edu/metadata/filelists" packages="210">
        let mut filelists_tag = BytesStart::borrowed_name(TAG_FILELISTS);
        filelists_tag.push_attribute(("xmlns", XML_NS_FILELISTS));
        writer.write_event(Event::Start(filelists_tag.to_borrowed()))?;

        for package in &self.packages {
            // <package pkgid="a2d3bce512f79b0bc840ca7912a86bbc0016cf06d5c363ffbb6fd5e1ef03de1b" name="fontconfig" arch="x86_64">
            let mut package_tag = BytesStart::borrowed_name(TAG_PACKAGE);
            package_tag.push_attribute(("pkgid", package.pkgid.as_str()));
            package_tag.push_attribute(("name", package.name.as_str()));
            package_tag.push_attribute(("arch", package.arch.as_str()));
            writer.write_event(Event::Start(package_tag.to_borrowed()))?;

            // <version epoch="0" ver="2.8.0" rel="5.fc33"/>
            let (epoch, version, release) = package.version.values();
            let mut version_tag = BytesStart::borrowed_name(TAG_VERSION);
            version_tag.push_attribute(("epoch", epoch));
            version_tag.push_attribute(("ver", version));
            version_tag.push_attribute(("rel", release));
            writer.write_event(Event::Empty(version_tag))?;

            // <file type="dir">/etc/fonts/conf.avail</file>
            for file in &package.files {
                let mut file_tag = BytesStart::borrowed_name(TAG_FILE);
                if let Some(filetype) = &file.filetype {
                    file_tag.push_attribute(("type".as_bytes(), filetype.to_values()));
                }
                writer.write_event(Event::Start(file_tag.to_borrowed()))?;
                writer.write_event(Event::Text(BytesText::from_plain_str(&file.path)))?;
                writer.write_event(Event::End(file_tag.to_end()))?;
            }

            // </package>
            writer.write_event(Event::End(package_tag.to_end()))?;
        }

        // </filelists>
        writer.write_event(Event::End(filelists_tag.to_end()))?;
        Ok(())
    }
}

//   <package pkgid="a2d3bce512f79b0bc840ca7912a86bbc0016cf06d5c363ffbb6fd5e1ef03de1b" name="fontconfig" arch="x86_64">
//     <version epoch="0" ver="2.8.0" rel="5.fc33"/>
//     <file type="dir">/etc/fonts/conf.avail</file>
//     ...
//     <file>/etc/fonts/conf.avail/10-autohint.conf</file>
//   </package>
pub fn parse_package<R: BufRead>(
    reader: &mut Reader<R>,
    open_tag: &BytesStart,
) -> Result<Package, MetadataError> {
    let mut package = Package::default();
    let mut buf = Vec::new();

    package.pkgid = open_tag
        .try_get_attribute("pkgid")?
        .unwrap()
        .unescape_and_decode_value(reader)?;
    package.name = open_tag
        .try_get_attribute("name")?
        .unwrap()
        .unescape_and_decode_value(reader)?;
    package.arch = open_tag
        .try_get_attribute("arch")?
        .unwrap()
        .unescape_and_decode_value(reader)?;

    loop {
        match reader.read_event(&mut buf)? {
            Event::End(e) if e.name() == TAG_PACKAGE => break,

            Event::Start(e) => match e.name() {
                TAG_VERSION => {
                    package.version = parse_version(reader, &e)?;
                }
                TAG_FILE => {
                    let file = parse_file(reader, &e)?;
                    package.files.push(file);
                }
                _ => (),
            },
            _ => (),
        }
    }

    Ok(package)
}

// <version epoch="0" ver="2.8.0" rel="5.fc33"/>
pub fn parse_version<R: BufRead>(
    reader: &mut Reader<R>,
    open_tag: &BytesStart,
) -> Result<EVR, MetadataError> {
    let epoch = open_tag
        .try_get_attribute("epoch")?
        .unwrap()
        .unescape_and_decode_value(reader)?;
    let version = open_tag
        .try_get_attribute("ver")?
        .unwrap()
        .unescape_and_decode_value(reader)?;
    let release = open_tag
        .try_get_attribute("rel")?
        .unwrap()
        .unescape_and_decode_value(reader)?;

    // TODO: double-allocations
    Ok(EVR::new(&epoch, &version, &release))
}

// <file type="dir">/etc/fonts/conf.avail</file>
pub fn parse_file<R: BufRead>(
    reader: &mut Reader<R>,
    open_tag: &BytesStart,
) -> Result<PackageFile, MetadataError> {
    let mut file = PackageFile::default();
    file.path = reader.read_text(open_tag.name(), &mut Vec::new())?;

    if let Some(filetype) = open_tag.try_get_attribute("type")? {
        file.filetype = Some(FileType::try_create(filetype.value.as_ref())?);
    }

    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::MetadataIO;
    use once_cell::sync::OnceCell;
    use pretty_assertions::assert_eq;
    use std::path::Path;

    const FIXTURE_FILELIST_PATH: &str = "./tests/assets/complex_repo/repodata/filelists.xml.gz";

    /// Fixture should cover all fields / tag types for repomd.xml
    /// Started w/ Fedora 33 updates repodata, added contenthash + repo, content, distro tags
    /// FilelistsDb covers standard fields + database_version, UpdateInfoZck covers header_size, header_checksum
    fn fixture_data() -> &'static Filelist {
        static INSTANCE: OnceCell<Filelist> = OnceCell::new();
        INSTANCE.get_or_init(|| {
            let mut filelist = Filelist::default();
            filelist.packages = vec![
                Package {
                    pkgid: "90fbba546300f507473547f33e229ee7bad94bbbe6e84b21d485e8e43b5f1132"
                        .to_owned(),
                    name: "rpm-empty".to_owned(),
                    arch: "x86_64".to_owned(),
                    version: EVR::new("0", "0", "0"),
                    files: vec![],
                },
                Package {
                    pkgid: "957de8a966af8fe8e55102489099d8b20bbecc23954c8c2bd88fb59625260393"
                        .to_owned(),
                    name: "rpm-with-non-ascii".to_owned(),
                    arch: "noarch".to_owned(),
                    version: EVR::new("0", "1", "1.fc33"),
                    files: vec![],
                },
            ];

            filelist
        })
    }

    /// Test deserialization of repomd with full coverage of all fields of RepoMd and RepoMdRecord
    #[test]
    fn test_deserialization() -> Result<(), MetadataError> {
        let actual = &Filelist::from_file(Path::new(FIXTURE_FILELIST_PATH))?;
        let expected = fixture_data();

        assert_eq!(actual, expected);
        // assert_eq!(actual.contenthash(), expected.contenthash());
        // assert_eq!(actual.repo_tags(), expected.repo_tags());
        // assert_eq!(actual.content_tags(), expected.content_tags());
        // assert_eq!(actual.distro_tags(), expected.distro_tags());

        Ok(())
    }

    // /// Test Serialization on a real repomd.xml (Fedora 33 x86_64 release "everything")
    // #[test]
    // fn test_serialization() -> Result<(), MetadataError> {
    //     let actual = fixture_data().to_string()?;

    //     let mut expected = String::new();
    //     File::open(FIXTURE_FILELIST_PATH)?.read_to_string(&mut expected)?;

    //     assert_eq!(&expected, &actual);

    //     Ok(())
    // }

    /// Test roundtrip (serialize + deserialize) on a real repomd.xml (Fedora 33 x86_64 release "everything")
    #[test]
    fn test_roundtrip() -> Result<(), MetadataError> {
        let first_deserialize = Filelist::from_file(Path::new(FIXTURE_FILELIST_PATH))?;
        let first_serialize = first_deserialize.to_string()?;

        let second_deserialize = Filelist::from_str(&first_serialize)?;
        let second_serialize = second_deserialize.to_string()?;

        assert_eq!(first_deserialize, second_deserialize);
        assert_eq!(first_serialize, second_serialize);

        Ok(())
    }
}
