use std::{
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
};

use anyhow::Result;
use quick_xml::Writer;
use rpmrepo::metadata::{Filelist, MetadataIO, Primary, RepoMd};

use super::TestCommand;

pub fn test(config: TestCommand) -> Result<()> {
    let path = PathBuf::from("./tests/assets/fedora33_updates_modified/repodata/repomd.xml");
    let repomd: RepoMd = RepoMd::from_file(&path).unwrap();
    // println!("{:#?}", repomd);

    let repomd_xml = repomd.to_string()?;

    println!("{}", repomd_xml);
    Ok(())

    // let path = PathBuf::from("./tests/assets/centos6/repodata/9f09d931b6d4d5da5dea9727c0b3998a363572c03b9a58b426e900d0317231da-filelists.xml");
    // let filelist: Filelist = Filelist::from_file(&path).unwrap();
    // println!("{:#?}", filelist.packages.get(0));
    // Ok(())

    // let xml = include_str!("../../tests/assets/fedora32/releases/Everything/x86_64/os/repodata/repomd.xml");
    // let repomd: RepoMd = from_str(xml).unwrap();
    // let xml: String = to_string(&repomd).unwrap();
    // println!("{:?}", xml);
    // Ok(())

    // let xml = include_str!("../../tests/assets/centos6/os/x86_64/repodata/8a106b58d2b45b4757ebba9f431cfcd6197392b5ee7640ab288b062fa754822c-primary.xml");
    // let primary: Primary = Primary::from_str(xml).unwrap();
    // println!("{:#?}", primary);
    // Ok(())

    // let xml = include_str!("../../tests/assets/centos6/os/x86_64/repodata/9f09d931b6d4d5da5dea9727c0b3998a363572c03b9a58b426e900d0317231da-filelists.xml");
    // let filelist: Filelist = from_str(xml)?;
    // println!("{:#?}", filelist);
    // Ok(())
}
