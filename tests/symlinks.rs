#[macro_use]
mod sys_common;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;
use sys_common::{io::tmpdir, symlink_supported};

#[test]
fn basic_symlinks() {
    if !symlink_supported() {
        return;
    }

    let tmpdir = tmpdir();

    check!(tmpdir.create("file"));
    check!(tmpdir.create_dir("dir"));

    check!(tmpdir.symlink_file("file", "file_symlink_file"));
    check!(tmpdir.symlink_dir("dir", "dir_symlink_dir"));
    check!(tmpdir.symlink("file", "file_symlink"));
    check!(tmpdir.symlink("dir", "dir_symlink"));

    assert!(check!(tmpdir.metadata("file_symlink_file")).is_file());
    assert!(check!(tmpdir.metadata("dir_symlink_dir")).is_dir());
    assert!(check!(tmpdir.metadata("file_symlink")).is_file());
    assert!(check!(tmpdir.metadata("dir_symlink")).is_dir());

    assert!(check!(tmpdir.symlink_metadata("file_symlink_file"))
        .file_type()
        .is_symlink());
    assert!(check!(tmpdir.symlink_metadata("dir_symlink_dir"))
        .file_type()
        .is_symlink());
    assert!(check!(tmpdir.symlink_metadata("file_symlink"))
        .file_type()
        .is_symlink());
    assert!(check!(tmpdir.symlink_metadata("dir_symlink"))
        .file_type()
        .is_symlink());
}

#[test]
fn symlink_absolute() {
    let tmpdir = tmpdir();

    error_contains!(
        tmpdir.symlink("/thing", "thing_symlink_file"),
        "a path led outside of the filesystem"
    );
    error_contains!(
        tmpdir.symlink_file("/file", "file_symlink_file"),
        "a path led outside of the filesystem"
    );
    error_contains!(
        tmpdir.symlink_dir("/dir", "dir_symlink_dir"),
        "a path led outside of the filesystem"
    );
}

#[test]
fn readlink_absolute() {
    if !symlink_supported() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();

    #[cfg(not(windows))]
    check!(std::os::unix::fs::symlink(
        "/thing",
        dir.path().join("thing_symlink")
    ));
    #[cfg(windows)]
    check!(std::os::windows::fs::symlink_file(
        "/file",
        dir.path().join("file_symlink_file")
    ));
    #[cfg(windows)]
    check!(std::os::windows::fs::symlink_dir(
        "/dir",
        dir.path().join("dir_symlink_dir")
    ));

    let tmpdir = check!(unsafe { Dir::open_ambient_dir(dir.path()) });

    #[cfg(not(windows))]
    error_contains!(
        tmpdir.read_link("thing_symlink"),
        "a path led outside of the filesystem"
    );
    #[cfg(windows)]
    error_contains!(
        tmpdir.read_link("file_symlink_file"),
        "a path led outside of the filesystem"
    );
    #[cfg(windows)]
    error_contains!(
        tmpdir.read_link("dir_symlink_dir"),
        "a path led outside of the filesystem"
    );
}

// Opening directories without following symlinks.
#[test]
fn open_dir_nofollow() {
    use cap_fs_ext::DirExt;

    if !symlink_supported() {
        return;
    }

    let tmpdir = tmpdir();

    check!(tmpdir.create("file"));
    check!(tmpdir.create_dir("dir"));
    check!(tmpdir.symlink_file("file", "symlink_file"));
    check!(tmpdir.symlink_dir("dir", "symlink_dir"));

    // First try without `nofollow`. The "symlink_dir" case should succeed.
    assert!(tmpdir.open_dir("file").is_err());
    assert!(tmpdir.open_dir("symlink_file").is_err());
    check!(tmpdir.open_dir("symlink_dir"));
    check!(tmpdir.open_dir("dir"));

    // Next try with `nofollow`. The "symlink_dir" case should fail.
    assert!(tmpdir.open_dir_nofollow("file").is_err());
    assert!(tmpdir.open_dir_nofollow("symlink_file").is_err());
    assert!(tmpdir.open_dir_nofollow("symlink_dir").is_err());
    check!(tmpdir.open_dir_nofollow("dir"));

    // Check that we follow symlinks in non-leading components.
    check!(tmpdir.open_dir_nofollow("dir/../dir"));
    assert!(tmpdir
        .open_dir_nofollow("symlink_dir/../symlink_dir")
        .is_err());
    check!(tmpdir.open_dir_nofollow("symlink_dir/../dir"));
    assert!(tmpdir.open_dir_nofollow("dir/../symlink_dir").is_err());

    // Check trailing slashes. These all succeed because a trailing slash acts
    // like a trailing slashdot, and the `.` isn't a symlink.
    check!(tmpdir.open_dir_nofollow("dir/../dir/"));
    check!(tmpdir.open_dir_nofollow("symlink_dir/../symlink_dir/"));
    check!(tmpdir.open_dir_nofollow("symlink_dir/../dir/"));
    check!(tmpdir.open_dir_nofollow("dir/../symlink_dir/"));

    // Check trailing slashdots. As above, these all succeed because the `.`
    // isn't a symlink.
    check!(tmpdir.open_dir_nofollow("dir/../dir/."));
    check!(tmpdir.open_dir_nofollow("symlink_dir/../symlink_dir/."));
    check!(tmpdir.open_dir_nofollow("symlink_dir/../dir/."));
    check!(tmpdir.open_dir_nofollow("dir/../symlink_dir/."));
}
