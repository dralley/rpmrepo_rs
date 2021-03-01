use std::io::{BufRead, Write};

// use super::metadata::RpmMetadata;
use quick_xml::{
    events::{BytesDecl, BytesStart, BytesText, Event},
    Reader, Writer,
};

use super::common::Checksum;
use super::metadata::{MetadataError, RpmMetadata, XML_NS_REPO, XML_NS_RPM};

// RepoMd
const TAG_REPOMD: &[u8] = b"repomd";
const TAG_REVISION: &[u8] = b"revision";
const TAG_CONTENTHASH: &[u8] = b"contenthash";
const TAG_TAGS: &[u8] = b"tags";
const TAG_DATA: &[u8] = b"data";
// Tags
const TAG_REPO: &[u8] = b"repo";
const TAG_CONTENT: &[u8] = b"content";
const TAG_DISTRO: &[u8] = b"distro";
// RepoMdRecord
const TAG_LOCATION: &[u8] = b"location";
const TAG_CHECKSUM: &[u8] = b"checksum";
const TAG_OPEN_CHECKSUM: &[u8] = b"open-checksum";
const TAG_HEADER_CHECKSUM: &[u8] = b"header-checksum";
const TAG_TIMESTAMP: &[u8] = b"timestamp";
const TAG_SIZE: &[u8] = b"size";
const TAG_OPEN_SIZE: &[u8] = b"open-size";
const TAG_HEADER_SIZE: &[u8] = b"header-size";
const TAG_DATABASE_VERSION: &[u8] = b"database_version";

const METADATA_PRIMARY: &str = "primary";
const METADATA_FILELISTS: &str = "filelists";
const METADATA_OTHER: &str = "other";
const METADATA_PRIMARY_DB: &str = "primary_db";
const METADATA_FILELISTS_DB: &str = "filelists_db";
const METADATA_OTHER_DB: &str = "other_db";
const METADATA_PRIMARY_ZCK: &str = "primary_zck";
const METADATA_FILELISTS_ZCK: &str = "filelists_zck";
const METADATA_OTHER_ZCK: &str = "other_zck";

#[derive(Debug, PartialEq)]
pub enum MetadataType {
    Primary,
    Filelists,
    Other,

    PrimaryZck,
    FilelistsZck,
    OtherZck,

    PrimaryDb,
    FilelistsDb,
    OtherDb,

    Unknown,
}

impl From<&str> for MetadataType {
    fn from(name: &str) -> Self {
        match name {
            METADATA_PRIMARY => MetadataType::Primary,
            METADATA_FILELISTS => MetadataType::Filelists,
            METADATA_OTHER => MetadataType::Other,

            METADATA_PRIMARY_DB => MetadataType::PrimaryDb,
            METADATA_FILELISTS_DB => MetadataType::FilelistsDb,
            METADATA_OTHER_DB => MetadataType::OtherDb,

            METADATA_PRIMARY_ZCK => MetadataType::PrimaryZck,
            METADATA_FILELISTS_ZCK => MetadataType::FilelistsZck,
            METADATA_OTHER_ZCK => MetadataType::OtherZck,

            _ => MetadataType::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct Distro {
    cpeid: Option<String>,
    name: String,
}

#[derive(Debug, PartialEq, Default)]
pub struct RepoMdRecord {
    // TODO: location on disk?

    /// Record type
    mdtype: String,
    /// Relative location of the file in a repository
    pub location_href: String,
    /// Mtime of the file
    timestamp: u64,
    /// Size of the file
    size: u64,
    /// Checksum of the file
    pub checksum: Checksum,

    /// Size of the archive content
    open_size: Option<u64>,
    /// Checksum of the archive content
    open_checksum: Option<Checksum>,

    /// Size of the Zchunk header
    header_size: Option<u64>,
    /// Checksum of the Zchunk header
    header_checksum: Option<Checksum>,

    /// Database version (used only for sqlite databases like primary.sqlite etc.)
    database_version: Option<u32>,
}

#[derive(Debug, PartialEq, Default)]
pub struct RepoMd {
    revision: String,
    data: Vec<RepoMdRecord>,
    repo_tags: Vec<String>,
    content_tags: Vec<String>,
    distro_tags: Vec<Distro>,
    contenthash: Option<Checksum>, // checksum of metatadata of packages in a sorted order
}

impl RepoMd {
    pub fn add_record(&mut self, record: RepoMdRecord) {
        self.data.push(record);
    }

    pub fn get_record(&self, rectype: &str) -> Option<&RepoMdRecord> {
        self.records().iter().find(|r| &r.mdtype == rectype)
    }

    pub fn records(&self) -> &Vec<RepoMdRecord> {
        &self.data
    }

    pub fn remove_record(&mut self, rectype: &str) {
        self.data.retain(|r| &r.mdtype != rectype);
    }

    pub fn add_repo_tag(&mut self, repo: String) {
        self.repo_tags.push(repo)
    }

    pub fn repo_tags(&self) -> &Vec<String> {
        &self.repo_tags
    }

    pub fn add_content_tag(&mut self, content: String) {
        self.content_tags.push(content)
    }

    pub fn content_tags(&self) -> &Vec<String> {
        &self.content_tags
    }

    pub fn add_distro_tag(&mut self, name: String, cpeid: Option<String>) {
        let distro = Distro { name, cpeid };
        self.distro_tags.push(distro)
    }

    pub fn distro_tags(&self) -> &Vec<Distro> {
        &self.distro_tags
    }

    pub fn sort_records(&mut self) {
        fn value(item: &RepoMdRecord) -> u32 {
            let mdtype = MetadataType::from(item.mdtype.as_str());
            match mdtype {
                MetadataType::Primary => 1,
                MetadataType::Filelists => 2,
                MetadataType::Other => 3,
                MetadataType::PrimaryDb => 4,
                MetadataType::FilelistsDb => 5,
                MetadataType::OtherDb => 6,
                MetadataType::PrimaryZck => 7,
                MetadataType::FilelistsZck => 8,
                MetadataType::OtherZck => 9,
                MetadataType::Unknown => 10,
            }
        }
        self.data.sort_by(|a, b| value(a).cmp(&value(b)));
    }

    pub fn set_contenthash(&mut self, contenthash: Checksum) {
        self.contenthash = Some(contenthash);
    }

    pub fn contenthash(&self) -> Option<&Checksum> {
        self.contenthash.as_ref()
    }

    pub fn set_revision(&mut self, revision: String) {
        self.revision = revision;
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn metadata_files(&self) -> &[RepoMdRecord] {
        &self.data
    }

    pub fn get_primary_data(&self) -> &RepoMdRecord {
        self.get_record(METADATA_PRIMARY)
            .expect("Cannot find primary.xml")
    }

    pub fn get_filelist_data(&self) -> &RepoMdRecord {
        self.get_record(METADATA_FILELISTS)
            .expect("Cannot find filelists.xml")
    }

    pub fn get_other_data(&self) -> &RepoMdRecord {
        self.get_record(METADATA_OTHER)
            .expect("Cannot find other.xml")
    }
}

impl RpmMetadata for RepoMd {
    fn deserialize<R: BufRead>(reader: &mut Reader<R>) -> Result<RepoMd, MetadataError> {
        let mut repomd = RepoMd::default();
        let mut buf = Vec::new();
        let mut record_buf = Vec::new();

        let mut found_metadata_tag = false;

        loop {
            match reader.read_event(&mut buf)? {
                Event::Start(e) => match e.name() {
                    TAG_REPOMD => {
                        found_metadata_tag = true;
                    }
                    TAG_REVISION => {
                        repomd.revision = reader.read_text(e.name(), &mut record_buf)?;
                    }
                    TAG_DATA => {
                        let data = parse_repomdrecord(reader, &e)?;
                        repomd.add_record(data);
                    }
                    TAG_CONTENTHASH => {
                        let contenthash_type = (&e).try_get_attribute("type")?.unwrap();
                        let contenthash = reader.read_text(e.name(), &mut record_buf)?;
                        let contenthash = Checksum::try_create(
                            contenthash_type.value.as_ref(),
                            contenthash.as_bytes(),
                        )?;
                        repomd.set_contenthash(contenthash);
                    }
                    TAG_TAGS => {
                        //   <tags>
                        //     <repo>Fedora</repo>
                        //     <content>binary-x86_64</content>
                        //     <distro cpeid="cpe:/o:fedoraproject:fedora:33">Fedora 33</distro>
                        //   </tags>
                        let mut tags_buf = Vec::new();
                        loop {
                            match reader.read_event(&mut record_buf)? {
                                Event::Start(e) => match e.name() {
                                    TAG_DISTRO => {
                                        let cpeid = (&e)
                                            .try_get_attribute("cpeid")?
                                            .and_then(|a| a.unescape_and_decode_value(reader).ok());
                                        let name = reader.read_text(TAG_DISTRO, &mut Vec::new())?;
                                        repomd.add_distro_tag(name, cpeid);
                                    }
                                    TAG_REPO => {
                                        let content = reader.read_text(e.name(), &mut tags_buf)?;
                                        repomd.add_repo_tag(content);
                                    }
                                    TAG_CONTENT => {
                                        let content = reader.read_text(e.name(), &mut tags_buf)?;
                                        repomd.add_content_tag(content);
                                    }
                                    _ => (),
                                },

                                Event::End(e) if e.name() == TAG_TAGS => break,
                                _ => (),
                            }
                            tags_buf.clear();
                        }
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
        Ok(repomd)
    }

    fn serialize<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), MetadataError> {
        // <?xml version="1.0" encoding="UTF-8"?>
        writer.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;

        // <repomd xmlns="http://linux.duke.edu/metadata/repo" xmlns:rpm="http://linux.duke.edu/metadata/rpm">
        let mut repomd_tag = BytesStart::borrowed_name(TAG_REPOMD);
        repomd_tag.push_attribute(("xmlns", XML_NS_REPO));
        repomd_tag.push_attribute(("xmlns:rpm", XML_NS_RPM));
        writer.write_event(Event::Start(repomd_tag.to_borrowed()))?;

        // <revision>123897</revision>
        writer
            .create_element(TAG_REVISION)
            .write_text_content(BytesText::from_plain_str(&self.revision()))?;

        // <contenthash type="sha256">09e1ee2ecaca3e43382e8db290922fd7c6533d56c397978893bb946f392f759d</contenthash>
        if let Some(contenthash) = self.contenthash() {
            let (contenthash_type, contenthash_value) = contenthash.to_values();
            writer
                .create_element(TAG_CONTENTHASH)
                .with_attribute(("type", contenthash_type))
                .write_text_content(BytesText::from_plain_str(contenthash_value))?;
        }

        write_tags(writer, &self)?;
        for data in self.records() {
            write_data(writer, data)?;
        }

        // </repomd>
        writer.write_event(Event::End(repomd_tag.to_end()))?;
        Ok(())
    }
}

// <data type="other_db">
//     <checksum type="sha256">fd2ff685b13d5b18b7c16d1316f7ccf299283cdf5db27ab780cb6b855b022000</checksum>
//     <open-checksum type="sha256">fd0619cc82de1a6475c98bd11cdd09e38b359c57a3ef1ab8411e5cc6076cbab8</open-checksum>
//     <location href="repodata/fd2ff685b13d5b18b7c16d1316f7ccf299283cdf5db27ab780cb6b855b022000-other.sqlite.xz"/>
//     <timestamp>1602869947</timestamp>
//     <database_version>10</database_version>
//     <size>78112</size>
//     <open-size>651264</open-size>
// </data>
pub fn parse_repomdrecord<R: BufRead>(
    reader: &mut Reader<R>,
    open_tag: &BytesStart,
) -> Result<RepoMdRecord, MetadataError> {
    let mut record = RepoMdRecord::default();

    record.mdtype = String::from_utf8(
        open_tag
            .try_get_attribute("type")?
            .unwrap()
            .value
            .iter()
            .cloned()
            .collect(),
    )
    .map_err(|e| e.utf8_error())?;

    let mut buf = Vec::new();
    let mut record_buf = Vec::new();

    loop {
        match reader.read_event(&mut buf)? {
            Event::Start(e) => match e.name() {
                TAG_CHECKSUM => {
                    let checksum_type = (&e).try_get_attribute("type")?.unwrap();
                    let checksum = reader.read_text(e.name(), &mut record_buf)?;
                    record.checksum =
                        Checksum::try_create(checksum_type.value.as_ref(), checksum.as_bytes())?;
                }
                TAG_OPEN_CHECKSUM => {
                    let checksum_type = (&e).try_get_attribute("type")?.unwrap();
                    let checksum = reader.read_text(e.name(), &mut record_buf)?;
                    record.open_checksum = Some(Checksum::try_create(
                        checksum_type.value.as_ref(),
                        checksum.as_bytes(),
                    )?);
                }
                TAG_HEADER_CHECKSUM => {
                    let checksum_type = (&e).try_get_attribute("type")?.unwrap();
                    let checksum = reader.read_text(e.name(), &mut record_buf)?;
                    record.header_checksum = Some(Checksum::try_create(
                        checksum_type.value.as_ref(),
                        checksum.as_bytes(),
                    )?);
                }
                TAG_LOCATION => {
                    record.location_href = (&e)
                        .try_get_attribute("href")?
                        .unwrap()
                        .unescape_and_decode_value(reader)?;
                }
                TAG_TIMESTAMP => {
                    record.timestamp = reader.read_text(e.name(), &mut record_buf)?.parse()?;
                }
                TAG_SIZE => {
                    record.size = reader.read_text(e.name(), &mut record_buf)?.parse()?;
                }
                TAG_HEADER_SIZE => {
                    record.header_size = reader.read_text(e.name(), &mut record_buf)?.parse().ok();
                }
                TAG_OPEN_SIZE => {
                    record.open_size = reader.read_text(e.name(), &mut record_buf)?.parse().ok();
                }
                TAG_DATABASE_VERSION => {
                    record.database_version =
                        reader.read_text(e.name(), &mut record_buf)?.parse().ok();
                }
                _ => (),
            },
            Event::End(e) if e.name() == TAG_DATA => break,
            _ => (),
        }
        record_buf.clear();
    }
    Ok(record)
}

/// <tags>
///   <repo>Fedora</repo>
///   <distro cpeid="cpe:/o:fedoraproject:fedora:33">Fedora 33</distro>
///   <content>binary-x86_64</content>
//// </tags>
fn write_tags<W: Write>(writer: &mut Writer<W>, repomd: &RepoMd) -> Result<(), quick_xml::Error> {
    let has_distro_tags = !repomd.distro_tags().is_empty();
    let has_repo_tags = !repomd.repo_tags().is_empty();
    let has_content_tags = !repomd.content_tags().is_empty();

    if has_distro_tags || has_repo_tags || has_content_tags {
        // <tags>
        let tags_tag = BytesStart::borrowed_name(TAG_DATA);
        writer.write_event(Event::Start(tags_tag.to_borrowed()))?;

        for item in repomd.repo_tags() {
            // <repo>Fedora</repo>
            writer
                .create_element(TAG_REPO)
                .write_text_content(BytesText::from_plain_str(&item))?;
        }

        for item in repomd.content_tags() {
            // <content>binary-x86_64</content>
            writer
                .create_element(TAG_CONTENT)
                .write_text_content(BytesText::from_plain_str(&item))?;
        }

        for item in repomd.distro_tags() {
            // <distro cpeid="cpe:/o:fedoraproject:fedora:33">Fedora 33</distro>
            let mut distro_tag = BytesStart::borrowed_name(TAG_DISTRO);
            if let Some(cpeid) = &item.cpeid {
                distro_tag.push_attribute(("cpeid", cpeid.as_str()))
            }
            writer.write_event(Event::End(distro_tag.to_end()))?;
        }

        // </tags>
        writer.write_event(Event::End(tags_tag.to_end()))?;
    }

    Ok(())
}

///   <data type="primary">
///    .....
///    <timestamp>1614969700</timestamp>
///    <size>5830735</size>
///    <open-size>53965949</open-size>
///  </data>
fn write_data<W: Write>(
    writer: &mut Writer<W>,
    data: &RepoMdRecord,
) -> Result<(), quick_xml::Error> {
    // <data>
    let mut data_tag = BytesStart::borrowed_name(TAG_DATA);
    data_tag.push_attribute(("type".as_bytes(), data.mdtype.as_bytes()));
    writer.write_event(Event::Start(data_tag.to_borrowed()))?;

    // <checksum type="sha256">afdc6dc379e58d097ed0b350536812bc6a604bbce50c5c109d8d98e28301dc4b</checksum>
    let (checksum_type, checksum_value) = data.checksum.to_values();
    writer
        .create_element(TAG_CHECKSUM)
        .with_attribute(("type", checksum_type))
        .write_text_content(BytesText::from_plain_str(checksum_value))?;

    // <open-checksum type="sha256">afdc6dc379e58d097ed0b350536812bc6a604bbce50c5c109d8d98e28301dc4b</open-checksum> (maybe)
    if let Some(open_checksum) = &data.open_checksum {
        let (checksum_type, checksum_value) = open_checksum.to_values();
        writer
            .create_element(TAG_OPEN_CHECKSUM)
            .with_attribute(("type", checksum_type))
            .write_text_content(BytesText::from_plain_str(checksum_value))?;
    }

    // <header-checksum type="sha256">afdc6dc379e58d097ed0b350536812bc6a604bbce50c5c109d8d98e28301dc4b</header-checksum> (maybe)
    if let Some(header_checksum) = &data.header_checksum {
        let (checksum_type, checksum_value) = header_checksum.to_values();
        writer
            .create_element(TAG_HEADER_CHECKSUM)
            .with_attribute(("type", checksum_type))
            .write_text_content(BytesText::from_plain_str(checksum_value))?;
    }

    // <location href="repodata/primary.xml.gz">
    writer
        .create_element(TAG_LOCATION)
        .with_attribute(("href", data.location_href.as_str()))
        .write_empty()?;

    // <timestamp>1602869947</timestamp>
    writer
        .create_element(TAG_TIMESTAMP)
        .write_text_content(BytesText::from_plain_str(
            data.timestamp.to_string().as_str(),
        ))?;

    // <size>123987</size>
    writer
        .create_element(TAG_SIZE)
        .write_text_content(BytesText::from_plain_str(&data.size.to_string()))?;

    // <open-size>68652</open-size> (maybe)
    if let Some(open_size) = data.open_size {
        writer
            .create_element(TAG_OPEN_SIZE)
            .write_text_content(BytesText::from_plain_str(&open_size.to_string()))?;
    }

    // <header-size>761487</header-size> (maybe)
    if let Some(size_header) = data.header_size {
        writer
            .create_element(TAG_HEADER_SIZE)
            .write_text_content(BytesText::from_plain_str(&size_header.to_string()))?;
    }

    // <database_version>10</database_version>
    if let Some(database_version) = data.database_version {
        writer
            .create_element(TAG_DATABASE_VERSION)
            .write_text_content(BytesText::from_plain_str(&database_version.to_string()))?;
    }

    // </data>
    writer.write_event(Event::End(data_tag.to_end()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::MetadataIO;
    use once_cell::sync::OnceCell;
    use pretty_assertions::assert_eq;
    use std::path::Path;

    const FIXTURE_REPOMD_PATH: &str = "./tests/assets/complex_repo/repodata/repomd.xml";

    /// Fixture should cover all fields / tag types for repomd.xml
    /// Started w/ Fedora 33 updates repodata, added contenthash + repo, content, distro tags
    /// FilelistsDb covers standard fields + database_version, UpdateInfoZck covers header_size, header_checksum
    fn fixture_data() -> &'static RepoMd {
        static INSTANCE: OnceCell<RepoMd> = OnceCell::new();
        INSTANCE.get_or_init(|| {
            let mut repomd = RepoMd::default();
            repomd.set_revision(String::from("1615686706"));
            // repomd.set_contenthash(Checksum::SHA256(String::from("09e1ee2ecaca3e43382e8db290922fd7c6533d56c397978893bb946f392f759d")));
            repomd.add_repo_tag(String::from("Fedora"));
            repomd.add_repo_tag(String::from("Fedora-Updates"));
            repomd.add_content_tag(String::from("binary-x86_64"));
            repomd.add_distro_tag(
                String::from("Fedora 33"),
                Some(String::from("cpe:/o:fedoraproject:fedora:33")),
            );
            repomd
        })
    }

    /// Test deserialization of repomd with full coverage of all fields of RepoMd and RepoMdRecord
    #[test]
    fn test_deserialization() -> Result<(), MetadataError> {
        let actual = &RepoMd::from_file(Path::new(FIXTURE_REPOMD_PATH))?;
        let expected = fixture_data();

        assert_eq!(actual.revision(), expected.revision());
        assert_eq!(actual.contenthash(), expected.contenthash());
        assert_eq!(actual.repo_tags(), expected.repo_tags());
        assert_eq!(actual.content_tags(), expected.content_tags());
        assert_eq!(actual.distro_tags(), expected.distro_tags());

        // TODO
        // assert_eq!(
        //     actual.get_record("filelists_db"),
        //     expected.get_record("filelists_db")
        // );
        // assert_eq!(
        //     actual.get_record("updateinfo_zck"),
        //     expected.get_record("updateinfo_zck")
        // );

        // assert_eq!(actual.records().len(), 17);
        // let expected_types = vec![
        //     "primary",
        //     "filelists",
        //     "other",
        //     "primary_db",
        //     "filelists_db",
        //     "other_db",
        //     "primary_zck",
        //     "filelists_zck",
        //     "other_zck",
        // ];
        // let actual_types = actual
        //     .records()
        //     .iter()
        //     .map(|r| r.mdtype.as_str())
        //     .collect::<Vec<&str>>();
        // assert_eq!(actual_types, expected_types);

        Ok(())
    }

    // /// Test Serialization on a real repomd.xml (Fedora 33 x86_64 release "everything")
    // #[test]
    // fn test_serialization() -> Result<(), MetadataError> {
    //     let actual = fixture_data().to_string()?;

    //     let mut expected = String::new();
    //     File::open(FIXTURE_REPOMD_PATH)?.read_to_string(&mut expected)?;

    //     assert_eq!(&expected, &actual);

    //     Ok(())
    // }

    /// Test roundtrip (serialize + deserialize) on a real repomd.xml (Fedora 33 x86_64 release "everything")
    #[test]
    fn test_roundtrip() -> Result<(), MetadataError> {
        let first_deserialize = RepoMd::from_file(Path::new(FIXTURE_REPOMD_PATH))?;
        let first_serialize = first_deserialize.to_string()?;
        let second_deserialize = RepoMd::from_str(&first_serialize)?;
        let second_serialize = second_deserialize.to_string()?;

        assert_eq!(first_deserialize, second_deserialize);
        assert_eq!(first_serialize, second_serialize);

        Ok(())
    }
}
