extern crate rpmrepo_metadata;

use pretty_assertions::assert_eq;
use quick_xml;
use rpmrepo_metadata::*;
use std::fs::OpenOptions;
use std::io::{Cursor, Read, Seek, SeekFrom};
use tempdir::TempDir;

mod common;

static EMPTY_PRIMARY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<metadata xmlns="http://linux.duke.edu/metadata/common" xmlns:rpm="http://linux.duke.edu/metadata/rpm" packages="0">
</metadata>
"#;

static COMPLEX_PRIMARY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<metadata xmlns="http://linux.duke.edu/metadata/common" xmlns:rpm="http://linux.duke.edu/metadata/rpm" packages="1">
  <package type="rpm">
    <name>complex-package</name>
    <arch>x86_64</arch>
    <version epoch="1" ver="2.3.4" rel="5.el8"/>
    <checksum type="sha256" pkgid="YES">6e46283a16954c9cecd3799246eb1a426d7d8a8b1bc8d57c55c3da4253e200e5</checksum>
    <summary>A package for exercising many different features of RPM metadata</summary>
    <description>Complex package</description>
    <packager>Michael Bluth</packager>
    <url>http://bobloblaw.com</url>
    <time file="1624680154" build="1624680153"/>
    <size package="8641" installed="117" archive="932"/>
    <location href="complex-package-2.3.4-5.el8.x86_64.rpm"/>
    <format>
      <rpm:license>MPLv2</rpm:license>
      <rpm:vendor>Bluth Company</rpm:vendor>
      <rpm:group>Development/Tools</rpm:group>
      <rpm:buildhost>localhost</rpm:buildhost>
      <rpm:sourcerpm>complex-package-2.3.4-5.el8.src.rpm</rpm:sourcerpm>
      <rpm:header-range start="4504" end="8377"/>
      <rpm:provides>
        <rpm:entry name="complex-package" flags="EQ" epoch="1" ver="2.3.4" rel="5.el8"/>
        <rpm:entry name="complex-package(x86-64)" flags="EQ" epoch="1" ver="2.3.4" rel="5.el8"/>
        <rpm:entry name="laughter" flags="EQ" epoch="0" ver="33"/>
        <rpm:entry name="narration(ronhoward)"/>
      </rpm:provides>
      <rpm:requires>
        <rpm:entry name="/usr/sbin/useradd" pre="1"/>
        <rpm:entry name="arson" flags="GE" epoch="0" ver="1.0.0" rel="1"/>
        <rpm:entry name="fur" flags="LE" epoch="0" ver="2"/>
        <rpm:entry name="staircar" flags="LE" epoch="0" ver="99.1" rel="3"/>
      </rpm:requires>
      <rpm:conflicts>
        <rpm:entry name="foxnetwork" flags="GT" epoch="0" ver="5555"/>
      </rpm:conflicts>
      <rpm:obsoletes>
        <rpm:entry name="bluemangroup" flags="LT" epoch="0" ver="32.1" rel="0"/>
        <rpm:entry name="cornballer" flags="LT" epoch="0" ver="444"/>
      </rpm:obsoletes>
      <rpm:suggests>
        <rpm:entry name="(bobloblaw &gt;= 1.1 if maritimelaw else anyone &lt; 0.5.1-2)"/>
        <rpm:entry name="(dove and return)"/>
        <rpm:entry name="(job or money &gt; 9000)"/>
      </rpm:suggests>
      <rpm:enhances>
        <rpm:entry name="(bananas or magic)"/>
      </rpm:enhances>
      <rpm:recommends>
        <rpm:entry name="((hiding and attic) if light-treason)"/>
        <rpm:entry name="GeneParmesan(PI)"/>
        <rpm:entry name="yacht" flags="GT" epoch="9" ver="11.0" rel="0"/>
      </rpm:recommends>
      <rpm:supplements>
        <rpm:entry name="((hiding and illusion) unless alliance-of-magicians)"/>
        <rpm:entry name="comedy" flags="EQ" epoch="0" ver="11.1" rel="4"/>
      </rpm:supplements>
      <file>/etc/complex/pkg.cfg</file>
      <file>/usr/bin/complex_a</file>
    </format>
  </package>
</metadata>
"#;

#[test]
fn test_primary_xml_writer_empty() -> Result<(), MetadataError> {
    let mut buf = Vec::new();

    let xml_writer = quick_xml::Writer::new_with_indent(Cursor::new(&mut buf), b' ', 2);
    let mut writer = PrimaryXml::new_writer(xml_writer);

    writer.write_header(0)?;
    writer.finish()?;

    let buffer = writer.into_inner().into_inner();

    let actual = std::str::from_utf8(buffer)?;
    let expected = EMPTY_PRIMARY;

    assert_eq!(&actual, &expected);

    Ok(())
}

#[test]
fn test_primary_xml_writer_complex_pkg() -> Result<(), MetadataError> {
    let mut buf = Vec::new();

    let xml_writer = quick_xml::Writer::new_with_indent(Cursor::new(&mut buf), b' ', 2);
    let mut writer = PrimaryXml::new_writer(xml_writer);

    writer.write_header(1)?;
    writer.write_package(&common::COMPLEX_PACKAGE)?;
    writer.finish()?;

    let buffer = writer.into_inner().into_inner();

    let actual = std::str::from_utf8(buffer)?;
    let expected = COMPLEX_PRIMARY;

    assert_eq!(&actual, &expected);

    Ok(())
}

#[test]
#[should_panic]
fn test_primary_xml_writer_not_enough_packages() {
    let mut buf = Vec::new();

    let xml_writer = quick_xml::Writer::new_with_indent(Cursor::new(&mut buf), b' ', 2);
    let mut writer = PrimaryXml::new_writer(xml_writer);

    writer.write_header(1).unwrap();
    writer.finish().unwrap();
}

#[test]
#[should_panic]
fn test_primary_xml_writer_too_many_packages() {
    let mut buf = Vec::new();

    let xml_writer = quick_xml::Writer::new_with_indent(Cursor::new(&mut buf), b' ', 2);
    let mut writer = PrimaryXml::new_writer(xml_writer);

    writer.write_header(1).unwrap();
    writer.write_package(&common::RPM_EMPTY).unwrap();
    writer.write_package(&common::RPM_WITH_NON_ASCII).unwrap();
    writer.finish().unwrap();
}

#[test]
fn test_primary_xml_writer_file() -> Result<(), MetadataError> {
    let working_dir = TempDir::new("")?;
    let other_name = working_dir.path().join("primary.xml");

    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(other_name)
        .unwrap();

    let xml_writer = quick_xml::Writer::new_with_indent(f, b' ', 2);
    let mut writer = PrimaryXml::new_writer(xml_writer);

    writer.write_header(0).unwrap();
    // writer.write_package(&common::RPM_EMPTY).unwrap();
    writer.finish()?;

    let mut f = writer.into_inner();

    f.seek(SeekFrom::Start(0))?;
    let mut actual = String::new();

    f.read_to_string(&mut actual).unwrap();

    assert_eq!(actual, EMPTY_PRIMARY);

    Ok(())
}
