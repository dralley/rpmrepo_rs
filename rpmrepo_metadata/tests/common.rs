extern crate rpmrepo_metadata;

use once_cell::sync::Lazy;
use rpmrepo_metadata::{Checksum, FileType, Package, Requirement, EVR};

// const FIXTURE_FILELIST_PATH: &str = "./tests/assets/complex_repo/repodata/filelists.xml.gz";

pub const COMPLEX_REPO_FIXTURE_PATH: &str = "./tests/assets/complex_repo/";
pub const EMPTY_REPO_FIXTURE_PATH: &str = "./tests/assets/empty_repo/";

pub static COMPLEX_PACKAGE: Lazy<Package> = Lazy::new(|| {
    let mut package = Package::default();

    package.set_name("complex-package");
    package.set_arch("x86_64");
    package.set_evr(EVR::new("1", "2.3.4", "5.el8"));
    package.set_checksum(Checksum::Sha256(
        "6e46283a16954c9cecd3799246eb1a426d7d8a8b1bc8d57c55c3da4253e200e5".to_owned(),
    ));
    package.set_summary("A package for exercising many different features of RPM metadata");
    package.set_description("Complex package");
    package.set_packager("Michael Bluth");
    package.set_url("http://bobloblaw.com");
    package.set_location_href("complex-package-2.3.4-5.el8.x86_64.rpm");
    package.set_time(1624680154, 1624680153);
    package.set_size(8641, 117, 932);

    package.set_rpm_license("MPLv2");
    package.set_rpm_buildhost("localhost");
    package.set_rpm_sourcerpm("complex-package-2.3.4-5.el8.src.rpm");
    package.set_rpm_group("Development/Tools");
    package.set_rpm_header_range(4504, 8377);
    package.set_rpm_vendor("Bluth Company");

    package.set_provides(vec![
        Requirement {
            name: "complex-package".to_owned(),
            flags: Some("EQ".to_owned()),
            epoch: Some("1".to_owned()),
            version: Some("2.3.4".to_owned()),
            release: Some("5.el8".to_owned()),
            ..Requirement::default()
        },
        Requirement {
            name: "complex-package(x86-64)".to_owned(),
            flags: Some("EQ".to_owned()),
            epoch: Some("1".to_owned()),
            version: Some("2.3.4".to_owned()),
            release: Some("5.el8".to_owned()),
            ..Requirement::default()
        },
        Requirement {
            name: "laughter".to_owned(),
            flags: Some("EQ".to_owned()),
            epoch: Some("0".to_owned()),
            version: Some("33".to_owned()),
            ..Requirement::default()
        },
        Requirement {
            name: "narration(ronhoward)".to_owned(),
            ..Requirement::default()
        },
    ]);
    package.set_requires(vec![
        Requirement {
            name: "/usr/sbin/useradd".to_owned(),
            preinstall: Some(true),
            ..Requirement::default()
        },
        Requirement {
            name: "arson".to_owned(),
            flags: Some("GE".to_owned()),
            epoch: Some("0".to_owned()),
            version: Some("1.0.0".to_owned()),
            release: Some("1".to_owned()),
            ..Requirement::default()
        },
        Requirement {
            name: "fur".to_owned(),
            flags: Some("LE".to_owned()),
            epoch: Some("0".to_owned()),
            version: Some("2".to_owned()),
            ..Requirement::default()
        },
        Requirement {
            name: "staircar".to_owned(),
            flags: Some("LE".to_owned()),
            epoch: Some("0".to_owned()),
            version: Some("99.1".to_owned()),
            release: Some("3".to_owned()),
            ..Requirement::default()
        },
    ]);

    package.set_conflicts(vec![Requirement {
        name: "foxnetwork".to_owned(),
        flags: Some("GT".to_owned()),
        epoch: Some("0".to_owned()),
        version: Some("5555".to_owned()),
        ..Requirement::default()
    }]);
    package.set_obsoletes(vec![
        Requirement {
            name: "bluemangroup".to_owned(),
            flags: Some("LT".to_owned()),
            epoch: Some("0".to_owned()),
            version: Some("32.1".to_owned()),
            release: Some("0".to_owned()),
            ..Requirement::default()
        },
        Requirement {
            name: "cornballer".to_owned(),
            flags: Some("LT".to_owned()),
            epoch: Some("0".to_owned()),
            version: Some("444".to_owned()),
            ..Requirement::default()
        },
    ]);
    package.set_suggests(vec![
        Requirement {
            name: "(bobloblaw >= 1.1 if maritimelaw else anyone < 0.5.1-2)".to_owned(),
            ..Requirement::default()
        },
        Requirement {
            name: "(dove and return)".to_owned(),
            ..Requirement::default()
        },
        Requirement {
            name: "(job or money > 9000)".to_owned(),
            ..Requirement::default()
        },
    ]);
    package.set_enhances(vec![Requirement {
        name: "(bananas or magic)".to_owned(),
        ..Requirement::default()
    }]);
    package.set_recommends(vec![
        Requirement {
            name: "((hiding and attic) if light-treason)".to_owned(),
            ..Requirement::default()
        },
        Requirement {
            name: "GeneParmesan(PI)".to_owned(),
            ..Requirement::default()
        },
        Requirement {
            name: "yacht".to_owned(),
            flags: Some("GT".to_owned()),
            epoch: Some("9".to_owned()),
            version: Some("11.0".to_owned()),
            release: Some("0".to_owned()),
            ..Requirement::default()
        },
    ]);
    package.set_supplements(vec![
        Requirement {
            name: "((hiding and illusion) unless alliance-of-magicians)".to_owned(),
            ..Requirement::default()
        },
        Requirement {
            name: "comedy".to_owned(),
            flags: Some("EQ".to_owned()),
            epoch: Some("0".to_owned()),
            version: Some("11.1".to_owned()),
            release: Some("4".to_owned()),
            ..Requirement::default()
        },
    ]);

    package.add_file(FileType::File, "/etc/complex/pkg.cfg");
    package.add_file(FileType::File, "/usr/bin/complex_a");
    package.add_file(FileType::Dir, "/usr/share/doc/complex-package");
    package.add_file(FileType::File, "/usr/share/doc/complex-package/README");
    package.add_file(FileType::Dir, "/var/lib/complex");
    package.add_file(FileType::Ghost, "/var/log/complex.log");

    package.add_changelog(
        "Lucille Bluth <lucille@bluthcompany.com> - 1.1.1-1",
        "- It's a banana, Michael. How much could it cost, $10?",
        1617192000,
    );
    package.add_changelog(
        "Job Bluth <job@alliance-of-magicians.com> - 2.2.2-2",
        "- I've made a huge mistake",
        1619352000,
    );
    package.add_changelog(
        "George Bluth <george@federalprison.gov> - 3.3.3-3",
        "- There’s always money in the banana stand",
        1623672000,
    );
    package
});

pub static RPM_WITH_INVALID_CHARS: Lazy<Package> = Lazy::new(|| {
    let mut package = Package::default();

    package.set_name("rpm-with-invalid-chars");
    package.set_arch("noarch");
    package.set_evr(EVR::new("0", "1", "1.fc33"));
    package.set_checksum(Checksum::Sha256(
        "64f1444f8e86a9ae6accdc2c4b12cb4a87fb2414c0998df461a8623a52eb3cc4".to_owned(),
    ));
    package
});

pub static RPM_EMPTY: Lazy<Package> = Lazy::new(|| {
    let mut package = Package::default();

    package.set_name("rpm-empty");
    package.set_arch("x86_64");
    package.set_evr(EVR::new("0", "0", "0"));
    package.set_checksum(Checksum::Sha256(
        "90fbba546300f507473547f33e229ee7bad94bbbe6e84b21d485e8e43b5f1132".to_owned(),
    ));
    package.set_location_href("rpm-empty-0-0.x86_64.rpm");
    package.set_time(1615737002, 1615686424);
    package.set_size(6005, 0, 124);

    package.set_rpm_license("LGPL");
    package.set_rpm_group("Unspecified");
    package.set_rpm_buildhost("localhost");
    package.set_rpm_sourcerpm("rpm-empty-0-0.src.rpm");
    package.set_rpm_header_range(4504, 5961);

    package.set_provides(vec![
        Requirement {
            name: "rpm-empty".to_owned(),
            flags: Some("EQ".to_owned()),
            epoch: Some("0".to_owned()),
            version: Some("0".to_owned()),
            release: Some("0".to_owned()),
            ..Requirement::default()
        },
        Requirement {
            name: "rpm-empty(x86-64)".to_owned(),
            flags: Some("EQ".to_owned()),
            epoch: Some("0".to_owned()),
            version: Some("0".to_owned()),
            release: Some("0".to_owned()),
            ..Requirement::default()
        },
    ]);

    package
});

pub static RPM_WITH_NON_ASCII: Lazy<Package> = Lazy::new(|| {
    let mut package = Package::default();
    package.set_name("rpm-with-non-ascii");
    package.set_arch("noarch");
    package.set_evr(EVR::new("0", "1", "1.fc33"));
    package.set_checksum(Checksum::Sha256(
        "957de8a966af8fe8e55102489099d8b20bbecc23954c8c2bd88fb59625260393".to_owned(),
    ));

    package
});

// /// Fixture should cover all fields / tag types for repomd.xml
// /// Started w/ Fedora 33 updates repodata, added repo, content, distro tags
// /// FilelistsDb covers standard fields + database_version, UpdateInfoZck covers header_size, header_checksum
pub fn complex_repo_fixture_data() -> Vec<&'static Package> {
    // static INSTANCE: OnceCell<&[&Package]> = OnceCell::new();
    // INSTANCE.get_or_init(|| {
    vec![
        &*COMPLEX_PACKAGE,
        &*RPM_WITH_INVALID_CHARS,
        &*RPM_EMPTY,
        &*RPM_WITH_NON_ASCII,
    ]
    // })
}
