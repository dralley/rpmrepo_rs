// <?xml version="1.0" encoding="UTF-8"?>
// <otherdata xmlns="http://linux.duke.edu/metadata/other" packages="1">
// <package pkgid="a2d3bce512f79b0bc840ca7912a86bbc0016cf06d5c363ffbb6fd5e1ef03de1b" name="deadbeef-devel" arch="x86_64">
//   <version epoch="0" ver="1.8.4" rel="2.fc33"/>
//   <changelog author="RPM Fusion Release Engineering &lt;leigh123linux@gmail.com&gt; - 0.7.3-0.2.20190209git373f556" date="1551700800">- Rebuilt for https://fedoraproject.org/wiki/Fedora_30_Mass_Rebuild</changelog>
//   <changelog author="Vasiliy N. Glazov &lt;vascom2@gmail.com&gt; - 1.8.0-1" date="1554724800">- Update to 1.8.0</changelog>
//   <changelog author="Vasiliy N. Glazov &lt;vascom2@gmail.com&gt; - 1.8.1-1" date="1561723200">- Update to 1.8.1</changelog>
//   <changelog author="Leigh Scott &lt;leigh123linux@gmail.com&gt; - 1.8.1-2" date="1565179200">- Rebuild for new ffmpeg version</changelog>
//   <changelog author="Vasiliy N. Glazov &lt;vascom2@gmail.com&gt; - 1.8.2-1" date="1565352000">- Update to 1.8.2</changelog>
// </package>
// </package>
// </otherdata>

use quick_xml::events::{BytesDecl, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::io::{BufRead, Write};

use super::common::EVR;
use super::metadata::{RpmMetadata, XML_NS_OTHER};
use super::MetadataError;

const TAG_OTHERDATA: &[u8] = b"otherdata";
const TAG_PACKAGE: &[u8] = b"package";
const TAG_VERSION: &[u8] = b"version";
const TAG_CHANGELOG: &[u8] = b"changelog";

#[derive(Debug, PartialEq, Default)]
pub struct Otherdata {
    pub packages: Vec<Package>,
}

#[derive(Debug, PartialEq, Default)]
pub struct Package {
    pkgid: String,
    name: String,
    arch: String,
    version: EVR,
    changelogs: Vec<Changelog>,
}

#[derive(Debug, PartialEq, Default)]
pub struct Changelog {
    author: String,
    date: u64,
    description: String,
}

impl RpmMetadata for Otherdata {
    fn deserialize<R: BufRead>(reader: &mut Reader<R>) -> Result<Otherdata, MetadataError> {
        let mut other = Otherdata::default();
        let mut buf = Vec::new();

        let mut found_metadata_tag = false;

        loop {
            match reader.read_event(&mut buf)? {
                Event::Start(e) => match e.name() {
                    TAG_OTHERDATA => {
                        found_metadata_tag = true;
                    }
                    TAG_PACKAGE => {
                        let package = parse_package(reader, &e)?;
                        other.packages.push(package);
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
        Ok(other)
    }

    fn serialize<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), MetadataError> {
        writer.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;

        let mut other_tag = BytesStart::borrowed_name(TAG_OTHERDATA);
        other_tag.push_attribute(("xmlns", XML_NS_OTHER));

        // <filelists>
        writer.write_event(Event::Start(other_tag.to_borrowed()))?;

        // <packages>
        for package in &self.packages {
            let mut package_tag = BytesStart::borrowed_name(TAG_PACKAGE);
            package_tag.push_attribute(("pkgid".as_bytes(), package.pkgid.as_bytes()));
            package_tag.push_attribute(("name".as_bytes(), package.name.as_bytes()));
            package_tag.push_attribute(("arch".as_bytes(), package.arch.as_bytes()));
            writer.write_event(Event::Start(package_tag.to_borrowed()))?;

            let (epoch, version, release) = package.version.values();
            // <version epoch="0" ver="2.8.0" rel="5.el6"/>
            let mut version_tag = BytesStart::borrowed_name(TAG_VERSION);
            version_tag.push_attribute(("epoch".as_bytes(), epoch.as_bytes()));
            version_tag.push_attribute(("ver".as_bytes(), version.as_bytes()));
            version_tag.push_attribute(("rel".as_bytes(), release.as_bytes()));
            writer.write_event(Event::Empty(version_tag))?;

            for changelog in &package.changelogs {
                //  <changelog author="dalley &lt;dalley@redhat.com&gt; - 2.7.2-1" date="1251720000">- Update to 2.7.2</changelog>
                writer
                    .create_element(TAG_CHANGELOG)
                    .with_attribute(("author".as_bytes(), changelog.author.as_str().as_bytes()))
                    .with_attribute((
                        "date".as_bytes(),
                        format!("{}", changelog.date).as_str().as_bytes(),
                    ))
                    .write_text_content(BytesText::from_plain_str(&changelog.description))?;
            }

            writer.write_event(Event::End(package_tag.to_end()))?;
        }
        writer.write_event(Event::End(other_tag.to_end()))?;
        Ok(())
    }
}

//   <package pkgid="6a915b6e1ad740994aa9688d70a67ff2b6b72e0ced668794aeb27b2d0f2e237b" name="fontconfig" arch="x86_64">
//     <version epoch="0" ver="2.8.0" rel="5.el6"/>
//     <changelog author="Behdad Esfahbod &lt;besfahbo@redhat.com&gt; - 2.7.2-1" date="1251720000">- Update to 2.7.2</changelog>
//     <changelog author="Behdad Esfahbod &lt;besfahbo@redhat.com&gt; - 2.7.3-1" date="1252411200">- Update to 2.7.3</changelog>
//     <changelog author="Behdad Esfahbod &lt;besfahbo@redhat.com&gt; - 2.8.0-1" date="1259841600">- Update to 2.8.0</changelog>
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
                TAG_CHANGELOG => {
                    let file = parse_changelog(reader, &e)?;
                    package.changelogs.push(file);
                }
                _ => (),
            },
            _ => (),
        }
    }

    Ok(package)
}

// <version epoch="0" ver="2.8.0" rel="5.el6"/>
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

pub fn parse_changelog<R: BufRead>(
    reader: &mut Reader<R>,
    open_tag: &BytesStart,
) -> Result<Changelog, MetadataError> {
    let mut changelog = Changelog::default();

    changelog.author = open_tag
        .try_get_attribute("author")?
        .unwrap()
        .unescape_and_decode_value(reader)?;
    changelog.date = open_tag
        .try_get_attribute("date")?
        .unwrap()
        .unescape_and_decode_value(reader)?
        .parse()?;

    changelog.description = reader.read_text(open_tag.name(), &mut Vec::new())?;

    Ok(changelog)
}
