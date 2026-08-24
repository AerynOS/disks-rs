// SPDX-FileCopyrightText: Copyright © 2026 aerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! The kernel's mounts table, parsed faithfully.
//!
//! Two details make this workth its own module rather than a `split_whitespace`
//! at the call site.
//!
//! The kernel escapes whitespace in the source and mount point fields as octal:
//! the space is `\040`, at tabl `\011`. Comparing raw text means a mount point with
//! a space in its name never matches anything, and the failure is silent.
//!
//! The source field is not always a path. bcachefs reports several devices
//! joined by colons, ZFS reports a dataset name, and pseudo-filesystems report
//! things like `tmpfs`. All three are legal. This modules records what the
//! kernel said and leaves judgement to the caller.

use crate::Error;
use std::{
    fs,
    path::{Path, PathBuf},
    str,
};

/// Path of the mounts table relative to a system root.
pub const PROC_MOUNTS: &str = "proc/self/mounts";

/// What the kernel reported in a row's source field.
///
/// Parsed here, judged elsewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MountSource {
    /// A single block device path.
    Single(PathBuf),
    /// Several devices joined by colons, as bcachefs reports.
    Multi(Vec<PathBuf>),
    /// Anything that is not a path: `tmpfs`, `overlay`, a ZFS dataset name.
    Other(String),
}

impl MountSource {
    /// Classify a source field.
    ///
    /// A leading `/` is what distinguishes a device path from a dataset name;
    /// ZFS names contain slashes but never begin with one.
    #[must_use]
    pub fn parse(source: &str) -> Self {
        if source.starts_with('/') {
            if source.contains(':') {
                return Self::Multi(source.split(':').map(PathBuf::from).collect());
            }
            return Self::Single(PathBuf::from(source));
        }
        Self::Other(source.to_owned())
    }

    /// THe single device path, when the source is one.
    #[must_use]
    pub fn single(&self) -> Option<&Path> {
        match self {
            Self::Single(path) => Some(path),
            Self::Multi(_) | Self::Other(_) => None,
        }
    }
}

/// One row of the mounts table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountEntry {
    /// What is mounted.
    pub source: MountSource,
    /// Where it is mounted, with escapes decoded.
    pub mountpoint: PathBuf,
    /// The filesystem type the kernel reported.
    pub filesystem: String,
    /// Mount options, in the order recorded.
    pub options: Vec<String>,
}

impl MountEntry {
    /// The value of a `key=value` option.
    ///
    /// The last occurrence wins, matching mount semantics where a later option
    /// overrides an earlier one.
    #[must_use]
    pub fn option_value(&self, key: &str) -> Option<&str> {
        self.options
            .iter()
            .rev()
            .find_map(|option| option.strip_prefix(key)?.strip_prefix('='))
    }

    /// Whether a bare flag option is present.
    #[must_use]
    pub fn has_flag(&self, flag: &str) -> bool {
        self.options.iter().any(|option| option == flag)
    }
}

/// The mounts table
#[derive(Debug, Clone, Default)]
pub struct MountTable {
    entries: Vec<MountEntry>,
}

impl MountTable {
    /// Read the table beneath a system root.
    ///
    /// A sysroot of `/` reads the running kernel's table.
    pub fn from_sysroot(sysroot: impl AsRef<Path>) -> Result<Self, Error> {
        let path = sysroot.as_ref().join(PROC_MOUNTS);
        let text = fs::read_to_string(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        Ok(Self::parse(&text))
    }

    /// Parse a table from text.
    ///
    /// Rows with fewer than four fields are skipped rather than failing: the
    /// table is written by the kernel a row at a time, and one unreadable row
    /// is no reason to refuse the rest.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let entries = text
            .lines()
            .filter_map(|line| {
                let mut fields = line.split_whitespace();
                let source = fields.next()?;
                let mountpoint = fields.next()?;
                let filesystem = fields.next()?;
                let options = fields.next()?;

                Some(MountEntry {
                    source: MountSource::parse(&unescape(source)),
                    mountpoint: PathBuf::from(unescape(mountpoint)),
                    filesystem: filesystem.to_owned(),
                    options: options.split(',').map(ToOwned::to_owned).collect(),
                })
            })
            .collect();
        Self { entries }
    }

    /// Every row, in the order the kernel listed them.
    #[must_use]
    pub fn entries(&self) -> &[MountEntry] {
        &self.entries
    }

    /// The row for a mountpoint.
    ///
    /// The *last* matching row wins. Mounting twice at one path shadows the
    /// earlier mount, and the kernel lists both, so taking the first would
    /// describe a filesystem nothing can currently see.
    #[must_use]
    pub fn for_mountpoint(&self, mountpoint: impl AsRef<Path>) -> Option<&MountEntry> {
        let wanted = mountpoint.as_ref();
        self.entries.iter().rev().find(|entry| entry.mountpoint == wanted)
    }

    /// The rows mounted at or beneath a directory, shallowest first.
    ///
    /// Useful for "what belongs to this install", where the answer is root
    /// and everything under it.
    pub fn under<'a>(&'a self, directory: &'a Path) -> impl Iterator<Item = &'a MountEntry> {
        self.entries
            .iter()
            .filter(move |entry| entry.mountpoint.starts_with(directory))
    }
}

/// Decode the kernel's octal escapes.
///
/// A backslash followed by exactly three octal digits is one byte. Anything
/// else, including a trailing backslash, is passed through unchanged; the
/// kernel does not produce those, and mangling them would be worse than
/// leaving them alone.
#[must_use]
fn unescape(field: &str) -> String {
    if !field.contains('\\') {
        return field.to_owned();
    }

    let bytes = field.as_bytes();
    let mut decoded = String::with_capacity(field.len());
    let mut index = 0;

    while index < bytes.len() {
        let octal = (bytes[index] == b'\\')
            .then(|| bytes.get(index + 1..index + 4))
            .flatten()
            .filter(|digits| digits.iter().all(|digit| (b'0'..=b'7').contains(digit)))
            .and_then(|digits| u8::from_str_radix(str::from_utf8(digits).ok()?, 8).ok());

        match octal {
            Some(byte) => {
                decoded.push(byte as char);
                index += 4;
            }
            None => {
                decoded.push(bytes[index] as char);
                index += 1;
            }
        }
    }
    decoded
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: &str = "\
/dev/nvme0n1p3 / btrfs rw,relatime,subvol=/@ 0 0
/dev/nvme0n1p1 /boot vfat rw,relatime,fmask=0077 0 0
tmpfs /run tmpfs rw,nosuid,nodev 0 0
/dev/sda1:/dev/sdb1 /srv bcachefs rw,realtime 0 0
rpool/ROOT/os /mnt/zfs zfs rw,noatime 0 0
";

    #[test]
    fn source_classification() {
        assert_eq!(
            MountSource::parse("/dev/nvme0n1p3"),
            MountSource::Single(PathBuf::from("/dev/nvme0n1p3"))
        );
        assert_eq!(
            MountSource::parse("/dev/sda1:/dev/sdb1"),
            MountSource::Multi(vec![PathBuf::from("/dev/sda1"), PathBuf::from("/dev/sdb1")])
        );
        // a dataset name contains slashes but never leads with one
        assert_eq!(
            MountSource::parse("rpool/ROOT/os"),
            MountSource::Other("rpool/ROOT/os".to_owned())
        );
        assert_eq!(MountSource::parse("tmpfs"), MountSource::Other("tmpfs".to_owned()));
        assert_eq!(
            MountSource::parse("weird:source"),
            MountSource::Other("weird:source".to_owned())
        );
    }

    #[test]
    fn parsing_a_table() {
        let table = MountTable::parse(TABLE);
        assert_eq!(table.entries().len(), 5);

        let root = table.for_mountpoint("/").unwrap();
        assert_eq!(root.filesystem, "btrfs");
        assert_eq!(root.source.single().unwrap(), Path::new("/dev/nvme0n1p3"));
        assert_eq!(root.option_value("subvol"), Some("/@"));
        assert!(root.has_flag("rw"));
        assert!(!root.has_flag("ro"));

        let srv = table.for_mountpoint("/srv").unwrap();
        assert!(matches!(srv.source, MountSource::Multi(_)));
        assert!(srv.source.single().is_none());
    }

    #[test]
    fn options_follow_mount_semantics() {
        let table = MountTable::parse("/dev/sda1 / btrfs rw,subvol=/@old,subvol=/@ 0 0\n");
        let root = table.for_mountpoint("/").unwrap();

        // a later option overrides an earlier one
        assert_eq!(root.option_value("subvol"), Some("/@"));
        // a key must match on its whole name
        assert_eq!(root.option_value("subvolid"), None);
        // a flag has no value and a key=value pair is not a flag
        assert_eq!(root.option_value("rw"), None);
        assert!(!root.has_flag("subvol"));
    }

    #[test]
    fn escaped_whitespace_is_decoded() {
        let table = MountTable::parse(
            "/dev/sdb1 /mnt/my\\040drive ext4 rw 0 0\n\
            /dev/sdc1 /mnt/tab\\011here ext4 rw 0 0\n",
        );
        // the whole point: this only matches if the escape was decoded
        let spaces = table.for_mountpoint("/mnt/my drive").unwrap();

        assert_eq!(spaces.filesystem, "ext4");
        assert!(table.for_mountpoint("/mnt/tab\there").is_some());
        // and the raw form must not match
        assert!(table.for_mountpoint("/mnt/my\\040drive").is_none());
    }

    #[test]
    fn escapes_are_decoded_in_the_source_too() {
        let table = MountTable::parse("/dev/disk/by-label/My\\040Disk / ext4 rw 0 0\n");
        let root = table.for_mountpoint("/").unwrap();

        assert_eq!(root.source.single().unwrap(), Path::new("/dev/disk/by-label/My Disk"));
    }

    #[test]
    fn malformed_escapes_pass_through_untouched() {
        // not three digits, out of range, and a trailing backslash
        assert_eq!(unescape("/mnt/a\\04b"), "/mnt/a\\04b");
        assert_eq!(unescape("/mnt/a\\099"), "/mnt/a\\099");
        assert_eq!(unescape("/mnt/trailing\\"), "/mnt/trailing\\");
        // a literal backslash, which the kernel writes as \134
        assert_eq!(unescape("/mnt/back\\134slash"), "/mnt/back\\slash");
    }

    #[test]
    fn a_shadowed_mountpoint_reports_the_visible_one() {
        let table = MountTable::parse(
            "/dev/sda1 /boot vfat rw 0 0\n\
             /dev/sdb1 /boot vfat rw 0 0\n",
        );
        // the second mount hides the first, so it is the one that answers
        let boot = table.for_mountpoint("/boot").unwrap();

        assert_eq!(boot.source.single().unwrap(), Path::new("/dev/sdb1"));
        assert_eq!(table.entries().len(), 2);
    }

    #[test]
    fn short_rows_are_skipped_not_fatal() {
        let table = MountTable::parse(
            "/dev/sda1 / ext4 rw 0 0\n\
             garbage\n\
             \n\
             /dev/sda2 /home ext4 rw 0 0\n",
        );
        assert_eq!(table.entries().len(), 2);
        assert!(table.for_mountpoint("/home").is_some());
    }

    #[test]
    fn subtree_query() {
        let table = MountTable::parse(TABLE);
        let under_mnt: Vec<&Path> = table
            .under(Path::new("/mnt"))
            .map(|entry| entry.mountpoint.as_path())
            .collect();

        assert_eq!(under_mnt, vec![Path::new("/mnt/zfs")]);
    }

    #[test]
    fn reading_a_missing_table_names_the_path() {
        let error = MountTable::from_sysroot("/nonexistent-sysroot").unwrap_err();

        assert!(matches!(error, Error::Io { .. }));
        assert!(error.to_string().contains(PROC_MOUNTS));
    }
}
