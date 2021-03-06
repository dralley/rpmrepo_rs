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

use std::io::{BufRead, Write};

use quick_xml::escape::partial_escape;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use super::metadata::{Changelog, OtherXml, Package, RpmMetadata, XML_NS_OTHER};
use super::{MetadataError, Repository, EVR};

const TAG_OTHERDATA: &[u8] = b"otherdata";
const TAG_PACKAGE: &[u8] = b"package";
const TAG_VERSION: &[u8] = b"version";
const TAG_CHANGELOG: &[u8] = b"changelog";

impl RpmMetadata for OtherXml {
    fn filename() -> &'static str {
        "other.xml"
    }

    fn load_metadata<R: BufRead>(
        repository: &mut Repository,
        reader: &mut Reader<R>,
    ) -> Result<(), MetadataError> {
        read_other_xml(repository, reader)
    }

    fn write_metadata<W: Write>(
        repository: &Repository,
        writer: Writer<W>,
    ) -> Result<(), MetadataError> {
        let mut writer = OtherXml::new_writer(writer);
        writer.write_header(repository.packages().len())?;
        for package in repository.packages().values() {
            writer.write_package(package)?;
        }
        writer.finish()
    }
}

fn read_other_xml<R: BufRead>(
    repository: &mut Repository,
    reader: &mut Reader<R>,
) -> Result<(), MetadataError> {
    let mut buf = Vec::new();

    let mut found_metadata_tag = false;

    loop {
        match reader.read_event(&mut buf)? {
            Event::Start(e) => match e.name() {
                TAG_OTHERDATA => {
                    found_metadata_tag = true;
                }
                TAG_PACKAGE => {
                    parse_package(repository, reader, &e)?;
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
    Ok(())
}

impl OtherXml {
    pub fn new_writer<W: Write>(writer: Writer<W>) -> OtherXmlWriter<W> {
        OtherXmlWriter {
            writer,
            num_packages: 0,
            packages_written: 0,
        }
    }

    pub fn new_reader<'a, R: BufRead>(reader: &'a mut Reader<R>) -> OtherXmlReader<'a, R> {
        OtherXmlReader { reader }
    }
}

pub struct OtherXmlWriter<W: Write> {
    writer: Writer<W>,
    num_packages: usize,
    packages_written: usize,
}

impl<W: Write> OtherXmlWriter<W> {
    pub fn write_header(&mut self, num_pkgs: usize) -> Result<(), MetadataError> {
        self.num_packages = num_pkgs;

        // <?xml version="1.0" encoding="UTF-8"?>
        self.writer
            .write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;

        // <otherdata xmlns="http://linux.duke.edu/metadata/other" packages="200">
        let mut other_tag = BytesStart::borrowed_name(TAG_OTHERDATA);
        other_tag.push_attribute(("xmlns", XML_NS_OTHER));
        other_tag.push_attribute(("packages", num_pkgs.to_string().as_str()));
        self.writer.write_event(Event::Start(other_tag))?;

        Ok(())
    }

    pub fn write_package(&mut self, package: &Package) -> Result<(), MetadataError> {
        let mut package_tag = BytesStart::borrowed_name(TAG_PACKAGE);
        let (_, pkgid) = package.checksum().to_values()?;
        package_tag.push_attribute(("pkgid", pkgid));
        package_tag.push_attribute(("name", package.name()));
        package_tag.push_attribute(("arch", package.arch()));
        self.writer
            .write_event(Event::Start(package_tag.to_borrowed()))?;

        let (epoch, version, release) = package.evr().values();
        // <version epoch="0" ver="2.8.0" rel="5.el6"/>
        let mut version_tag = BytesStart::borrowed_name(TAG_VERSION);
        version_tag.push_attribute(("epoch", epoch));
        version_tag.push_attribute(("ver", version));
        version_tag.push_attribute(("rel", release));
        self.writer.write_event(Event::Empty(version_tag))?;

        for changelog in package.changelogs() {
            //  <changelog author="dalley &lt;dalley@redhat.com&gt; - 2.7.2-1" date="1251720000">- Update to 2.7.2</changelog>
            self.writer
                .create_element(TAG_CHANGELOG)
                .with_attribute(("author", changelog.author.as_str()))
                .with_attribute(("date", format!("{}", changelog.date).as_str()))
                .write_text_content(BytesText::from_escaped(partial_escape(
                    &changelog.description.as_bytes(),
                )))?;
        }

        // </package>
        self.writer.write_event(Event::End(package_tag.to_end()))?;

        self.packages_written += 1;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), MetadataError> {
        assert_eq!(
            self.packages_written, self.num_packages,
            "Number of packages written {} does not match number of packages declared {}.",
            self.packages_written, self.num_packages
        );

        // </otherdata>
        self.writer
            .write_event(Event::End(BytesEnd::borrowed(TAG_OTHERDATA)))?;

        // trailing newline
        self.writer
            .write_event(Event::Text(BytesText::from_plain_str("\n")))?;

        Ok(())
    }

    pub fn into_inner(self) -> W {
        self.writer.into_inner()
    }
}

pub struct OtherXmlReader<'a, R: BufRead> {
    reader: &'a mut Reader<R>,
}

impl<'a, R: BufRead> OtherXmlReader<'a, R> {
    pub fn read_header(&mut self) {}

    pub fn read_package(&mut self, package: &mut Package) {}

    pub fn finish(&mut self) {}
}

//   <package pkgid="6a915b6e1ad740994aa9688d70a67ff2b6b72e0ced668794aeb27b2d0f2e237b" name="fontconfig" arch="x86_64">
//     <version epoch="0" ver="2.8.0" rel="5.el6"/>
//     <changelog author="Behdad Esfahbod &lt;besfahbo@redhat.com&gt; - 2.7.2-1" date="1251720000">- Update to 2.7.2</changelog>
//     <changelog author="Behdad Esfahbod &lt;besfahbo@redhat.com&gt; - 2.7.3-1" date="1252411200">- Update to 2.7.3</changelog>
//     <changelog author="Behdad Esfahbod &lt;besfahbo@redhat.com&gt; - 2.8.0-1" date="1259841600">- Update to 2.8.0</changelog>
//   </package>
pub fn parse_package<R: BufRead>(
    repository: &mut Repository,
    reader: &mut Reader<R>,
    open_tag: &BytesStart,
) -> Result<(), MetadataError> {
    let mut buf = Vec::new();

    let pkgid = open_tag
        .try_get_attribute("pkgid")?
        .ok_or_else(|| MetadataError::MissingAttributeError("pkgid"))?
        .unescape_and_decode_value(reader)?;
    let name = open_tag
        .try_get_attribute("name")?
        .ok_or_else(|| MetadataError::MissingAttributeError("name"))?
        .unescape_and_decode_value(reader)?;
    let arch = open_tag
        .try_get_attribute("arch")?
        .ok_or_else(|| MetadataError::MissingAttributeError("arch"))?
        .unescape_and_decode_value(reader)?;

    let mut package = repository
        .packages_mut()
        .entry(pkgid)
        .or_insert(Package::default()); // TODO

    // TODO: using empty strings as null value is slightly questionable
    if package.name().is_empty() {
        package.set_name(&name);
    }

    if package.arch().is_empty() {
        package.set_arch(&arch);
    }

    loop {
        match reader.read_event(&mut buf)? {
            Event::End(e) if e.name() == TAG_PACKAGE => break,

            Event::Start(e) => match e.name() {
                TAG_VERSION => {
                    package.set_evr(parse_evr(reader, &e)?);
                }
                TAG_CHANGELOG => {
                    let changelog = parse_changelog(reader, &e)?;
                    // TODO: Temporary changelog?
                    package.add_changelog(
                        &changelog.author,
                        &changelog.description,
                        changelog.date,
                    );
                }
                _ => (),
            },
            _ => (),
        }
    }

    Ok(())
}

// <version epoch="0" ver="2.8.0" rel="5.el6"/>
pub fn parse_evr<R: BufRead>(
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
