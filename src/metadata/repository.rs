struct RpmRepository {}

impl RpmRepository {
    // allocation?
    // one allocator per package, everything freed at once

    pub fn add_package() {}

    pub fn remove_package() {}

    // what to do with updateinfo, groups, modules when packages added or removed?

    pub fn load_from_metadata() { // path to repo
    }

    pub fn write_metadata() { // path to repo
    }

    // configuration options for writing metadata:
    // * number of old packages?
    // * checksum types for metadata
    // * compression types. how customizable does it need to be?
    // * sqlite metadata yes/no
    // * zchunk metadata?
    // * signing
}

// struct Package {
//     name
//     epoch
//     version
//     release
//     arch:
//     pkg_id
//     checksum:

//     location_href
//     description
//     summary
//     url

//     conflicts:
//     enhances
//     provides
//     recommends
//     requires
//     suggests
//     supplements

//     rpm_buildhost
//     rpm_group
//     rpm_header_end
//     rpm_header_start
//     rpm_license
//     rpm_packager
//     rpm_sourcerpm
//     rpm_vendor

//     size_archive
//     size_installed
//     size_package

//     time_build
//     time_file

//     changelog:
//     files

//     nvra
//     nevra
//     location_base
// }
