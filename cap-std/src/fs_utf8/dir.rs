use crate::{
    fs::OpenOptions,
    fs_utf8::{from_utf8, to_utf8, DirBuilder, File, Metadata, ReadDir},
};
use std::{fmt, fs, io};

#[cfg(any(unix, target_os = "fuchsia"))]
use {
    crate::os::unix::net::{UnixDatagram, UnixListener, UnixStream},
    std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd},
};

#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, RawHandle};

/// A reference to an open directory on a filesystem.
///
/// This does not directly correspond to anything in `std`, however its methods
/// correspond to the [functions in `std::fs`] and the constructor methods for
/// [`std::fs::File`].
///
/// Unlike `std::fs`, this API's `canonicalize` returns a relative path since
/// absolute paths don't interoperate well with the capability model. And it lacks
/// a `set_permissions` method because popular host platforms don't have a way to
/// perform that operation in a manner compatible with cap-std's sandbox; instead,
/// open the file and call [`File::set_permissions`].
///
/// [functions in `std::fs`]: https://doc.rust-lang.org/std/fs/index.html#functions
/// [`std::fs::File`]: https://doc.rust-lang.org/std/fs/struct.File.html
/// [`File::set_permissions`]: struct.File.html#method.set_permissions
pub struct Dir {
    cap_std: crate::fs::Dir,
}

impl Dir {
    /// Constructs a new instance of `Self` from the given `std::fs::File`.
    #[inline]
    pub fn from_std_file(std_file: fs::File) -> Self {
        Self::from_cap_std(crate::fs::Dir::from_std_file(std_file))
    }

    /// Constructs a new instance of `Self` from the given `cap_std::fs::Dir`.
    #[inline]
    pub fn from_cap_std(cap_std: crate::fs::Dir) -> Self {
        Self { cap_std }
    }

    /// Attempts to open a file in read-only mode.
    ///
    /// This corresponds to [`std::fs::File::open`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::File::open`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.open
    #[inline]
    pub fn open<P: AsRef<str>>(&self, path: P) -> io::Result<File> {
        let path = from_utf8(path)?;
        self.cap_std.open(path).map(File::from_cap_std)
    }

    /// Opens a file at `path` with the options specified by `self`.
    ///
    /// This corresponds to [`std::fs::OpenOptions::open`].
    ///
    /// Instead of being a method on `OpenOptions`, this is a method on `Dir`,
    /// and it only accesses paths relative to `self`.
    ///
    /// [`std::fs::OpenOptions::open`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.open
    #[inline]
    pub fn open_with<P: AsRef<str>>(&self, path: P, options: &OpenOptions) -> io::Result<File> {
        let path = from_utf8(path)?;
        self.cap_std
            .open_with(path, options)
            .map(File::from_cap_std)
    }

    /// Attempts to open a directory.
    #[inline]
    pub fn open_dir<P: AsRef<str>>(&self, path: P) -> io::Result<Self> {
        let path = from_utf8(path)?;
        self.cap_std.open_dir(path).map(Self::from_cap_std)
    }

    /// Creates a new, empty directory at the provided path.
    ///
    /// This corresponds to [`std::fs::create_dir`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::create_dir`]: https://doc.rust-lang.org/std/fs/fn.create_dir.html
    #[inline]
    pub fn create_dir<P: AsRef<str>>(&self, path: P) -> io::Result<()> {
        let path = from_utf8(path)?;
        self.cap_std.create_dir(path)
    }

    /// Recursively create a directory and all of its parent components if they are missing.
    ///
    /// This corresponds to [`std::fs::create_dir_all`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::create_dir_all`]: https://doc.rust-lang.org/std/fs/fn.create_dir_all.html
    #[inline]
    pub fn create_dir_all<P: AsRef<str>>(&self, path: P) -> io::Result<()> {
        let path = from_utf8(path)?;
        self.cap_std.create_dir_all(path)
    }

    /// Creates the specified directory with the options configured in this builder.
    ///
    /// This corresponds to [`std::fs::DirBuilder::create`].
    ///
    /// [`std::fs::DirBuilder::create`]: https://doc.rust-lang.org/std/fs/struct.DirBuilder.html#method.create
    #[inline]
    pub fn create_dir_with<P: AsRef<str>>(
        &self,
        path: P,
        dir_builder: &DirBuilder,
    ) -> io::Result<()> {
        let path = from_utf8(path)?;
        self.cap_std.create_dir_with(path, dir_builder)
    }

    /// Opens a file in write-only mode.
    ///
    /// This corresponds to [`std::fs::File::create`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::File::create`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.create
    #[inline]
    pub fn create<P: AsRef<str>>(&self, path: P) -> io::Result<File> {
        let path = from_utf8(path)?;
        self.cap_std.create(path).map(File::from_cap_std)
    }

    /// Returns the canonical form of a path with all intermediate components normalized
    /// and symbolic links resolved.
    ///
    /// This corresponds to [`std::fs::canonicalize`], but instead of returning an
    /// absolute path, returns a path relative to the directory represented by `self`.
    ///
    /// [`std::fs::canonicalize`]: https://doc.rust-lang.org/std/fs/fn.canonicalize.html
    #[inline]
    pub fn canonicalize<P: AsRef<str>>(&self, path: P) -> io::Result<String> {
        let path = from_utf8(path)?;
        self.cap_std.canonicalize(path).and_then(to_utf8)
    }

    /// Copies the contents of one file to another. This function will also copy the permission
    /// bits of the original file to the destination file.
    ///
    /// This corresponds to [`std::fs::copy`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::copy`]: https://doc.rust-lang.org/std/fs/fn.copy.html
    #[inline]
    pub fn copy<P: AsRef<str>, Q: AsRef<str>>(
        &self,
        from: P,
        to_dir: &Self,
        to: Q,
    ) -> io::Result<u64> {
        let from = from_utf8(from)?;
        let to = from_utf8(to)?;
        self.cap_std.copy(from, &to_dir.cap_std, to)
    }

    /// Creates a new hard link on a filesystem.
    ///
    /// This corresponds to [`std::fs::hard_link`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::hard_link`]: https://doc.rust-lang.org/std/fs/fn.hard_link.html
    #[inline]
    pub fn hard_link<P: AsRef<str>, Q: AsRef<str>>(
        &self,
        src: P,
        dst_dir: &Self,
        dst: Q,
    ) -> io::Result<()> {
        let src = from_utf8(src)?;
        let dst = from_utf8(dst)?;
        self.cap_std.hard_link(src, &dst_dir.cap_std, dst)
    }

    /// Given a path, query the file system to get information about a file, directory, etc.
    ///
    /// This corresponds to [`std::fs::metadata`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::metadata`]: https://doc.rust-lang.org/std/fs/fn.metadata.html
    #[inline]
    pub fn metadata<P: AsRef<str>>(&self, path: P) -> io::Result<Metadata> {
        let path = from_utf8(path)?;
        self.cap_std.metadata(path)
    }

    /// Returns an iterator over the entries within `self`.
    #[inline]
    pub fn entries(&self) -> io::Result<ReadDir> {
        self.cap_std.entries().map(ReadDir::from_cap_std)
    }

    /// Returns an iterator over the entries within a directory.
    ///
    /// This corresponds to [`std::fs::read_dir`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::read_dir`]: https://doc.rust-lang.org/std/fs/fn.read_dir.html
    #[inline]
    pub fn read_dir<P: AsRef<str>>(&self, path: P) -> io::Result<ReadDir> {
        let path = from_utf8(path)?;
        self.cap_std.read_dir(path).map(ReadDir::from_cap_std)
    }

    /// Read the entire contents of a file into a bytes vector.
    ///
    /// This corresponds to [`std::fs::read`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::read`]: https://doc.rust-lang.org/std/fs/fn.read.html
    #[inline]
    pub fn read<P: AsRef<str>>(&self, path: P) -> io::Result<Vec<u8>> {
        let path = from_utf8(path)?;
        self.cap_std.read(path)
    }

    /// Reads a symbolic link, returning the file that the link points to.
    ///
    /// This corresponds to [`std::fs::read_link`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::read_link`]: https://doc.rust-lang.org/std/fs/fn.read_link.html
    #[inline]
    pub fn read_link<P: AsRef<str>>(&self, path: P) -> io::Result<String> {
        let path = from_utf8(path)?;
        self.cap_std.read_link(path).and_then(to_utf8)
    }

    /// Read the entire contents of a file into a string.
    ///
    /// This corresponds to [`std::fs::read_to_string`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::read_to_string`]: https://doc.rust-lang.org/std/fs/fn.read_to_string.html
    #[inline]
    pub fn read_to_string<P: AsRef<str>>(&self, path: P) -> io::Result<String> {
        let path = from_utf8(path)?;
        self.cap_std.read_to_string(path)
    }

    /// Removes an existing, empty directory.
    ///
    /// This corresponds to [`std::fs::remove_dir`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::remove_dir`]: https://doc.rust-lang.org/std/fs/fn.remove_dir.html
    #[inline]
    pub fn remove_dir<P: AsRef<str>>(&self, path: P) -> io::Result<()> {
        let path = from_utf8(path)?;
        self.cap_std.remove_dir(path)
    }

    /// Removes a directory at this path, after removing all its contents. Use carefully!
    ///
    /// This corresponds to [`std::fs::remove_dir_all`], but only accesses paths
    /// relative to `self`.
    ///
    /// XXX: On Windows, this requires Rust nightly and the "nightly" feature (windows_file_type_ext).
    ///
    /// [`std::fs::remove_dir_all`]: https://doc.rust-lang.org/std/fs/fn.remove_dir_all.html
    #[cfg(any(not(windows), feature = "windows_file_type_ext"))]
    #[inline]
    pub fn remove_dir_all<P: AsRef<str>>(&self, path: P) -> io::Result<()> {
        let path = from_utf8(path)?;
        self.cap_std.remove_dir_all(path)
    }

    /// Remove the directory referenced by `self` and consume `self`.
    ///
    /// Note that even though this implementation works in terms of handles
    /// as much as possible, removal is not guaranteed to be atomic with respect
    /// to a concurrent rename of the directory.
    ///
    /// XXX: On Windows, this requires Rust nightly and the "nightly" feature (windows_file_type_ext).
    #[cfg(any(not(windows), feature = "windows_file_type_ext"))]
    #[inline]
    pub fn remove_open_dir(self) -> io::Result<()> {
        self.cap_std.remove_open_dir()
    }

    /// Removes the directory referenced by `self`, after removing all its contents, and
    /// consume `self`. Use carefully!
    ///
    /// Note that even though this implementation works in terms of handles
    /// as much as possible, removal is not guaranteed to be atomic with respect
    /// to a concurrent rename of the directory.
    ///
    /// XXX: On Windows, this requires Rust nightly and the "nightly" feature (windows_file_type_ext).
    #[cfg(any(not(windows), feature = "windows_file_type_ext"))]
    #[inline]
    pub fn remove_open_dir_all(self) -> io::Result<()> {
        self.cap_std.remove_open_dir_all()
    }

    /// Removes a file from a filesystem.
    ///
    /// This corresponds to [`std::fs::remove_file`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::remove_file`]: https://doc.rust-lang.org/std/fs/fn.remove_file.html
    #[inline]
    pub fn remove_file<P: AsRef<str>>(&self, path: P) -> io::Result<()> {
        let path = from_utf8(path)?;
        self.cap_std.remove_file(path)
    }

    /// Rename a file or directory to a new name, replacing the original file if to already exists.
    ///
    /// This corresponds to [`std::fs::rename`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::rename`]: https://doc.rust-lang.org/std/fs/fn.rename.html
    #[inline]
    pub fn rename<P: AsRef<str>, Q: AsRef<str>>(
        &self,
        from: P,
        to_dir: &Self,
        to: Q,
    ) -> io::Result<()> {
        let from = from_utf8(from)?;
        let to = from_utf8(to)?;
        self.cap_std.rename(from, &to_dir.cap_std, to)
    }

    /// Query the metadata about a file without following symlinks.
    ///
    /// This corresponds to [`std::fs::symlink_metadata`], but only accesses paths
    /// relative to `self`.
    ///
    /// XXX: On Windows, this requires Rust nightly and the "nightly" feature (windows_file_type_ext).
    ///
    /// [`std::fs::symlink_metadata`]: https://doc.rust-lang.org/std/fs/fn.symlink_metadata.html
    #[cfg(any(not(windows), feature = "windows_file_type_ext"))]
    #[inline]
    pub fn symlink_metadata<P: AsRef<str>>(&self, path: P) -> io::Result<Metadata> {
        let path = from_utf8(path)?;
        self.cap_std.symlink_metadata(path)
    }

    /// Write a slice as the entire contents of a file.
    ///
    /// This corresponds to [`std::fs::write`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::fs::write`]: https://doc.rust-lang.org/std/fs/fn.write.html
    #[inline]
    pub fn write<P: AsRef<str>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> io::Result<()> {
        let path = from_utf8(path)?;
        self.cap_std.write(path, contents)
    }

    /// Creates a new symbolic link on a filesystem.
    ///
    /// This corresponds to [`std::os::unix::fs::symlink`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::os::unix::fs::symlink`]: https://doc.rust-lang.org/std/os/unix/fs/fn.symlink.html
    #[cfg(not(windows))]
    #[inline]
    pub fn symlink<P: AsRef<str>, Q: AsRef<str>>(&self, src: P, dst: Q) -> io::Result<()> {
        let src = from_utf8(src)?;
        let dst = from_utf8(dst)?;
        self.cap_std.symlink(src, dst)
    }

    /// Creates a new file symbolic link on a filesystem.
    ///
    /// This corresponds to [`std::os::windows::fs::symlink_file`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::os::windows::fs::symlink_file`]: https://doc.rust-lang.org/std/os/windows/fs/fn.symlink_file.html
    #[cfg(windows)]
    #[inline]
    pub fn symlink_file<P: AsRef<str>, Q: AsRef<str>>(&self, src: P, dst: Q) -> io::Result<()> {
        let src = from_utf8(src)?;
        let dst = from_utf8(dst)?;
        self.cap_std.symlink_file(src, dst)
    }

    /// Creates a new directory symlink on a filesystem.
    ///
    /// This corresponds to [`std::os::windows::fs::symlink_dir`], but only accesses paths
    /// relative to `self`.
    ///
    /// [`std::os::windows::fs::symlink_dir`]: https://doc.rust-lang.org/std/os/windows/fs/fn.symlink_dir.html
    #[cfg(windows)]
    #[inline]
    pub fn symlink_dir<P: AsRef<str>, Q: AsRef<str>>(&self, src: P, dst: Q) -> io::Result<()> {
        let src = from_utf8(src)?;
        let dst = from_utf8(dst)?;
        self.cap_std.symlink_dir(src, dst)
    }

    /// Creates a new `UnixListener` bound to the specified socket.
    ///
    /// This corresponds to [`std::os::unix::net::UnixListener::bind`], but only
    /// accesses paths relative to `self`.
    ///
    /// XXX: This function is not yet implemented.
    ///
    /// [`std::os::unix::net::UnixListener::bind`]: https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html#method.bind
    #[cfg(unix)]
    #[inline]
    pub fn bind_unix_listener<P: AsRef<str>>(&self, path: P) -> io::Result<UnixListener> {
        let path = from_utf8(path)?;
        self.cap_std.bind_unix_listener(path)
    }

    /// Connects to the socket named by path.
    ///
    /// This corresponds to [`std::os::unix::net::UnixStream::connect`], but only
    /// accesses paths relative to `self`.
    ///
    /// XXX: This function is not yet implemented.
    ///
    /// [`std::os::unix::net::UnixStream::connect`]: https://doc.rust-lang.org/std/os/unix/net/struct.UnixStream.html#method.connect
    #[cfg(unix)]
    #[inline]
    pub fn connect_unix_stream<P: AsRef<str>>(&self, path: P) -> io::Result<UnixStream> {
        let path = from_utf8(path)?;
        self.cap_std.connect_unix_stream(path)
    }

    /// Creates a Unix datagram socket bound to the given path.
    ///
    /// This corresponds to [`std::os::unix::net::UnixDatagram::bind`], but only
    /// accesses paths relative to `self`.
    ///
    /// XXX: This function is not yet implemented.
    ///
    /// [`std::os::unix::net::UnixDatagram::bind`]: https://doc.rust-lang.org/std/os/unix/net/struct.UnixDatagram.html#method.bind
    #[cfg(unix)]
    #[inline]
    pub fn bind_unix_datagram<P: AsRef<str>>(&self, path: P) -> io::Result<UnixDatagram> {
        let path = from_utf8(path)?;
        self.cap_std.bind_unix_datagram(path)
    }

    /// Connects the socket to the specified address.
    ///
    /// This corresponds to [`std::os::unix::net::UnixDatagram::connect`], but only
    /// accesses paths relative to `self`.
    ///
    /// XXX: This function is not yet implemented.
    ///
    /// [`std::os::unix::net::UnixDatagram::connect`]: https://doc.rust-lang.org/std/os/unix/net/struct.UnixDatagram.html#method.connect
    #[cfg(unix)]
    #[inline]
    pub fn connect_unix_datagram<P: AsRef<str>>(
        &self,
        unix_datagram: &UnixDatagram,
        path: P,
    ) -> io::Result<()> {
        let path = from_utf8(path)?;
        self.cap_std.connect_unix_datagram(unix_datagram, path)
    }

    /// Sends data on the socket to the specified address.
    ///
    /// This corresponds to [`std::os::unix::net::UnixDatagram::send_to`], but only
    /// accesses paths relative to `self`.
    ///
    /// XXX: This function is not yet implemented.
    ///
    /// [`std::os::unix::net::UnixDatagram::send_to`]: https://doc.rust-lang.org/std/os/unix/net/struct.UnixDatagram.html#method.send_to
    #[cfg(unix)]
    #[inline]
    pub fn send_to_unix_datagram_addr<P: AsRef<str>>(
        &self,
        unix_datagram: &UnixDatagram,
        buf: &[u8],
        path: P,
    ) -> io::Result<usize> {
        let path = from_utf8(path)?;
        self.cap_std
            .send_to_unix_datagram_addr(unix_datagram, buf, path)
    }

    /// Creates a new `Dir` instance that shares the same underlying file handle as the existing
    /// `Dir` instance.
    #[inline]
    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            cap_std: self.cap_std.try_clone()?,
        })
    }

    /// Returns `true` if the path points at an existing entity.
    ///
    /// This corresponds to [`std::path::Path::exists`], but only
    /// accesses paths relative to `self`.
    ///
    /// [`std::path::Path::exists`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.exists
    #[inline]
    pub fn exists<P: AsRef<str>>(&self, path: P) -> bool {
        match from_utf8(path) {
            Ok(path) => self.cap_std.exists(path),
            Err(_) => false,
        }
    }

    /// Returns `true` if the path exists on disk and is pointing at a regular file.
    ///
    /// This corresponds to [`std::path::Path::is_file`], but only
    /// accesses paths relative to `self`.
    ///
    /// [`std::path::Path::is_file`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.is_file
    #[inline]
    pub fn is_file<P: AsRef<str>>(&self, path: P) -> bool {
        match from_utf8(path) {
            Ok(path) => self.cap_std.is_file(path),
            Err(_) => false,
        }
    }

    /// Checks if `path` is a directory.
    ///
    /// This is similar to [`std::path::Path::is_dir`] in that it checks if `path` relative to `Dir`
    /// is a directory. This function will traverse symbolic links to query information about the
    /// destination file. In case of broken symbolic links, this will return `false`.
    ///
    /// [`std::path::Path::is_dir`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.is_dir
    #[inline]
    pub fn is_dir<P: AsRef<str>>(&self, path: P) -> bool {
        match from_utf8(path) {
            Ok(path) => self.cap_std.is_dir(path),
            Err(_) => false,
        }
    }

    /// Constructs a new instance of `Self` by opening the given path as a
    /// directory using the host process' ambient authority.
    ///
    /// # Safety
    ///
    /// This function is not sandboxed and may access any path that the host
    /// process has access to.
    #[inline]
    pub unsafe fn open_ambient_dir<P: AsRef<str>>(path: P) -> io::Result<Self> {
        let path = from_utf8(path)?;
        crate::fs::Dir::open_ambient_dir(path).map(Self::from_cap_std)
    }
}

#[cfg(unix)]
impl FromRawFd for Dir {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self::from_std_file(fs::File::from_raw_fd(fd))
    }
}

#[cfg(windows)]
impl FromRawHandle for Dir {
    #[inline]
    unsafe fn from_raw_handle(handle: RawHandle) -> Self {
        Self::from_std_file(fs::File::from_raw_handle(handle))
    }
}

#[cfg(unix)]
impl AsRawFd for Dir {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.cap_std.as_raw_fd()
    }
}

#[cfg(windows)]
impl AsRawHandle for Dir {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.cap_std.as_raw_handle()
    }
}

#[cfg(unix)]
impl IntoRawFd for Dir {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.cap_std.into_raw_fd()
    }
}

#[cfg(windows)]
impl IntoRawHandle for Dir {
    #[inline]
    fn into_raw_handle(self) -> RawHandle {
        self.cap_std.into_raw_handle()
    }
}

impl fmt::Debug for Dir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cap_std.fmt(f)
    }
}