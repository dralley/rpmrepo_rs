use std::{
    fmt::format,
    io::{BufRead, Write},
};

use quick_xml::{events::Event, Writer};
use quick_xml::{
    events::{BytesDecl, BytesStart, BytesText},
    Reader,
};

use super::{
    common::Checksum,
    metadata::MetadataError,
    metadata::{RpmMetadata, XML_NS_COMMON, XML_NS_RPM},
};

const TAG_METADATA: &[u8] = b"metadata";
const TAG_NAME: &[u8] = b"name";
const TAG_VERSION: &[u8] = b"version";
const TAG_CHECKSUM: &[u8] = b"checksum";
const TAG_ARCH: &[u8] = b"arch";
const TAG_SUMMARY: &[u8] = b"summary";
const TAG_DESCRIPTION: &[u8] = b"description";
const TAG_PACKAGER: &[u8] = b"packager";
const TAG_URL: &[u8] = b"url";
const TAG_FORMAT: &[u8] = b"format";
const TAG_TIME: &[u8] = b"time";
const TAG_SIZE: &[u8] = b"size";
const TAG_LOCATION: &[u8] = b"location";

#[derive(Debug, PartialEq, Default)]
struct Entry {
    name: Option<String>,
    epoch: Option<String>,
    version: Option<String>, // ver
    release: Option<String>, // rel
    flags: Option<String>,
} // TODO: use EVR?

#[derive(Debug, Default, PartialEq)]
struct EVR {
    epoch: String,
    version: String, // ver
    release: String, // rel
}

#[derive(Debug, PartialEq, Default)]
struct File {
    file: String,
}

#[derive(Debug, PartialEq, Default)]
struct Time {
    file: u64,
    build: u64,
}

#[derive(Debug, PartialEq, Default)]
struct Size {
    package: String,
    installed: String,
    archive: String,
}

#[derive(Debug, PartialEq, Default)]
struct EntryList {
    entries: Vec<Entry>, // entry
}

#[derive(Debug, PartialEq, Default)]
struct Format {
    files: Option<Vec<File>>, // file

    rpm_provides: EntryList,          // rpm:provides
    rpm_requires: Option<EntryList>,  // rpm:requires
    rpm_conflicts: Option<EntryList>, // rpm:conflicts
    rpm_obsoletes: Option<EntryList>, // rpm:obsoletes

    rpm_license: String,           // rpm:license
    rpm_vendor: String,            // rpm:vendor
    rpm_group: String,             // rpm:group
    rpm_buildhost: String,         // rpm:buildhost
    rpm_sourcerpm: String,         // rpm:sourcerpm
    rpm_header_range: HeaderRange, // rpm:header-range
}

#[derive(Debug, PartialEq, Default)]
struct HeaderRange {
    start: u64,
    end: u64,
}

// Requirement (Provides, Conflicts, Obsoletes, Requires).
#[derive(Debug, PartialEq, Default)]
struct Requirement {
    name: String,
    epoch: String,
    version: String,
    release: String,
    preinstall: bool,
}

enum RequirementType {
    LT,
    GT,
    EQ,
    LE,
    GE,
}

impl From<RequirementType> for &str {
    fn from(rtype: RequirementType) -> Self {
        match rtype {
            LT => "LT",
            GT => "GT",
            EQ => "EQ",
            LE => "LE",
            GE => "GE",
        }
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct Package {
    name: String,
    arch: String,
    version: EVR,
    pub checksum: Checksum,
    summary: String,
    description: String,
    packager: String,
    url: String,
    time: Time,
    size: Size,
    pub location_href: String,
    format: Format,
    requires: Vec<Requirement>,
    provides: Vec<Requirement>,
    conflicts: Vec<Requirement>,
    obsoletes: Vec<Requirement>,
    suggests: Vec<Requirement>,
    enhances: Vec<Requirement>,
    recommends: Vec<Requirement>,
    supplements: Vec<Requirement>,
}

#[derive(Debug, PartialEq, Default)]
pub struct Primary {
    packages: Vec<Package>, // package
}

impl Primary {
    pub fn get_packages(&self) -> &[Package] {
        &self.packages
    }
}

impl RpmMetadata for Primary {
    fn deserialize<R: BufRead>(reader: &mut Reader<R>) -> Result<Self, MetadataError>
    where
        Self: Sized,
    {
        let mut primary = Primary::default();

        let mut buf = Vec::new();
        let mut found_metadata_tag = false;

        // TODO: less buffers, less allocation
        loop {
            match reader.read_event(&mut buf)? {
                Event::Start(e) => match e.name() {
                    b"metadata" => {
                        found_metadata_tag = true;
                    }
                    b"package" => {
                        let data = parse_package(reader, &e)?;
                        primary.packages.push(data);
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

        Ok(primary)
    }

    fn serialize<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), MetadataError> {
        // <?xml version="1.0" encoding="UTF-8"?>
        writer.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;

        // <metadata xmlns="http://linux.duke.edu/metadata/common" xmlns:rpm="http://linux.duke.edu/metadata/rpm" packages="35">
        let mut metadata_tag = BytesStart::borrowed_name(TAG_METADATA);
        metadata_tag.push_attribute(("xmlns", XML_NS_COMMON));
        metadata_tag.push_attribute(("xmlns:rpm", XML_NS_RPM));
        metadata_tag.push_attribute(("packages", 0.to_string().as_str())); // TODO: use real number
        writer.write_event(Event::Start(metadata_tag.to_borrowed()))?;

        for package in &self.packages {
            // <package type="rpm">
            let mut package_tag = BytesStart::borrowed_name(TAG_METADATA);
            package_tag.push_attribute(("type", "rpm"));
            writer.write_event(Event::Start(package_tag.to_borrowed()))?;

            // <name>bear</name>
            writer
                .create_element(TAG_NAME)
                .write_text_content(BytesText::from_plain_str(&package.name))?;

            // <arch>noarch</arch>
            writer
                .create_element(TAG_ARCH)
                .write_text_content(BytesText::from_plain_str(&package.arch))?;

            // <version epoch="0" ver="4.1" rel="1"/>
            writer
                .create_element(TAG_VERSION)
                .with_attribute(("epoch", package.version.epoch.as_str()))
                .with_attribute(("ver", package.version.version.as_str()))
                .with_attribute(("rel", package.version.release.as_str()))
                .write_empty()?;

            // <checksum type="sha256" pkgid="YES">afdc6dc379e58d097ed0b350536812bc6a604bbce50c5c109d8d98e28301dc4b</checksum>
            let (checksum_type, checksum_value) = package.checksum.to_values();
            writer
                .create_element(TAG_CHECKSUM)
                .with_attribute(("type", checksum_type))
                .with_attribute(("pkgId", "YES"))
                .write_text_content(BytesText::from_plain_str(checksum_value))?;

            // <summary>A dummy package of horse</summary>
            writer
                .create_element(TAG_SUMMARY)
                .write_text_content(BytesText::from_plain_str(&package.summary))?;

            // <description>A dummy package of horse</description>
            writer
                .create_element(TAG_DESCRIPTION)
                .write_text_content(BytesText::from_plain_str(&package.description))?;

            // <packager>Bojack Horseman</packager>
            writer
                .create_element(TAG_PACKAGER)
                .write_text_content(BytesText::from_plain_str(&package.packager))?;

            // <url>http://arandomaddress.com</url>
            writer
                .create_element(TAG_URL)
                .write_text_content(BytesText::from_plain_str(&package.url))?;

            // <time file="1617454165" build="1331871271"/>
            writer
                .create_element(TAG_TIME)
                .with_attribute(("file", package.time.file.to_string().as_str()))
                .with_attribute(("build", package.time.build.to_string().as_str()))
                .write_empty()?;

            // <size package="1921" installed="52" archive="246"/>
            writer
                .create_element(TAG_SIZE)
                .with_attribute(("package", package.size.package.as_str()))
                .with_attribute(("installed", package.size.installed.as_str()))
                .with_attribute(("archive", package.size.archive.as_str()))
                .write_empty()?;

            // <location href="horse-4.1-1.noarch.rpm"/>
            writer
                .create_element(TAG_LOCATION)
                .with_attribute(("href", package.location_href.as_str()))
                .write_empty()?;

            // <format>
            let format_tag = BytesStart::borrowed_name(TAG_FORMAT);
            writer.write_event(Event::Start(format_tag.to_borrowed()))?;

            // <rpm:license>GPLv2</rpm:license>
            writer
                .create_element("rpm:license")
                .write_text_content(BytesText::from_plain_str(
                    package.format.rpm_license.as_str(),
                ))?;

            // <rpm:vendor>Netflix</rpm:vendor>
            writer
                .create_element("rpm:vendor")
                .write_text_content(BytesText::from_plain_str(
                    package.format.rpm_vendor.as_str(),
                ))?;

            // <rpm:group>Internet/Applications</rpm:group>
            writer
                .create_element("rpm:group")
                .write_text_content(BytesText::from_plain_str(
                    &package.format.rpm_group.as_str(),
                ))?;

            // <rpm:buildhost>smqe-ws15</rpm:buildhost>
            writer.create_element("rpm:buildhost").write_text_content(
                BytesText::from_plain_str(&package.format.rpm_buildhost.as_str()),
            )?;

            // <rpm:sourcerpm>horse-4.1-1.src.rpm</rpm:sourcerpm>
            writer.create_element("rpm:sourcerpm").write_text_content(
                BytesText::from_plain_str(&package.format.rpm_sourcerpm.as_str()),
            )?;

            // <rpm:header-range start="280" end="1696"/>
            writer
                .create_element("rpm:header-range")
                .with_attribute((
                    "start",
                    package.format.rpm_header_range.start.to_string().as_str(),
                ))
                .with_attribute((
                    "end",
                    package.format.rpm_header_range.end.to_string().as_str(),
                ))
                .write_empty()?;

            // // <rpm:supplements>
            // //   <rpm:entry name="horse" flags="EQ" epoch="0" ver="4.1" rel="1"/>
            // // </rpm:supplements>
            // write_requirement_section(writer, "rpm:provides", package.provides)?;
            // write_requirement_section(writer, "rpm:requires", package.requires)?;
            // write_requirement_section(writer, "rpm:conflicts", package.conflicts)?;
            // write_requirement_section(writer, "rpm:obsoletes", package.obsoletes)?;
            // write_requirement_section(writer, "rpm:suggests", package.suggests)?;
            // write_requirement_section(writer, "rpm:enhances", package.enhances)?;
            // write_requirement_section(writer, "rpm:recommends", package.recommends)?;
            // write_requirement_section(writer, "rpm:supplements", package.supplements)?;

            // // <file type="dir">/etc/fonts/conf.avail</file>
            // for file in package.files {
            //     writer.create_element("file")
            // }

            // </format>
            writer.write_event(Event::End(format_tag.to_end()))?;

            // </package>
            writer.write_event(Event::End(package_tag.to_end()))?;
        }

        // </metadata>
        writer.write_event(Event::End(metadata_tag.to_end()))?;
        Ok(())
    }
}

// <rpm:supplements>
//   <rpm:entry name="horse" flags="EQ" epoch="0" ver="4.1" rel="1"/>
// </rpm:supplements>
fn write_requirement_section<W: Write>(
    writer: &mut Writer<W>,
    entry_list: &[Entry],
    section_name: &[u8],
) -> Result<(), quick_xml::Error> {
    let section_tag = BytesStart::borrowed_name(section_name);
    writer.write_event(Event::Start(section_tag.to_borrowed()))?;

    for entry in entry_list {
        writer
            .create_element("entry")
            .with_attribute(("name", entry.name.as_ref().unwrap().as_str()))
            .with_attribute(("flags", entry.name.as_ref().unwrap().as_str()))
            .with_attribute(("epoch", entry.name.as_ref().unwrap().as_str()))
            .with_attribute(("ver", entry.version.as_ref().unwrap().as_str()))
            .with_attribute(("rel", entry.release.as_ref().unwrap().as_str()))
            .write_empty()?;
    }

    writer.write_event(Event::End(section_tag.to_end()))?;

    Ok(())
}

pub fn parse_package<R: BufRead>(
    reader: &mut Reader<R>,
    open_tag: &BytesStart,
) -> Result<Package, MetadataError> {
    let ptype = open_tag
        .try_get_attribute(b"type")?
        .unwrap()
        .unescape_and_decode_value(reader)?;

    Ok(Package::default())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use once_cell::sync::OnceCell;

    use super::*;
    use crate::metadata::MetadataIO;

    const FIXTURE_PACKAGE_PATH: &str = "./tests/assets/complex_repo/repodata/primary.xml.gz";

    /// Fixture should cover all fields / tag types for repomd.xml
    /// Started w/ Fedora 33 updates repodata, added contenthash + repo, content, distro tags
    /// FilelistsDb covers standard fields + database_version, UpdateInfoZck covers header_size, header_checksum
    fn fixture_data() -> &'static Primary {
        static INSTANCE: OnceCell<Primary> = OnceCell::new();
        INSTANCE.get_or_init(|| {
            let mut primary = Primary::default();
            primary
        })
    }

    /// Test deserialization of repomd with full coverage of all fields of RepoMd and RepoMdRecord
    #[test]
    fn test_deserialization() -> Result<(), MetadataError> {
        let actual = &Primary::from_file(Path::new(FIXTURE_PACKAGE_PATH))?;
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
    //     File::open(FIXTURE_PACKAGE_PATH)?.read_to_string(&mut expected)?;

    //     assert_eq!(&expected, &actual);

    //     Ok(())
    // }

    /// Test roundtrip (serialize + deserialize) on a real repomd.xml (Fedora 33 x86_64 release "everything")
    #[test]
    fn test_roundtrip() -> Result<(), MetadataError> {
        let first_deserialize = Primary::from_file(Path::new(FIXTURE_PACKAGE_PATH))?;
        let first_serialize = first_deserialize.to_string()?;

        let second_deserialize = Primary::from_str(&first_serialize)?;
        let second_serialize = second_deserialize.to_string()?;

        assert_eq!(first_deserialize, second_deserialize);
        assert_eq!(first_serialize, second_serialize);

        Ok(())
    }
}
