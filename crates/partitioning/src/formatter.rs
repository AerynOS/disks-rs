// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{path::Path, process::Command};

use types::Filesystem;

/// Trait for generating filesystem-specific formatting commands and arguments
pub trait FilesystemExt {
    /// Returns the appropriate mkfs command for the filesystem
    fn mkfs_command(&self) -> &str;

    /// Returns the command-line arguments for setting UUID, if applicable
    fn uuid_arg(&self) -> Vec<String>;

    /// Returns the command-line arguments for setting filesystem label, if applicable
    fn label_arg(&self) -> Vec<String>;

    /// Returns the force format argument if applicable
    fn force_arg(&self) -> Vec<String>;
}

impl FilesystemExt for Filesystem {
    fn mkfs_command(&self) -> &str {
        match self {
            Filesystem::Fat32 { .. } => "mkfs.fat",
            Filesystem::Standard { filesystem_type, .. } => match filesystem_type {
                types::StandardFilesystemType::F2fs => "mkfs.f2fs",
                types::StandardFilesystemType::Ext4 => "mkfs.ext4",
                types::StandardFilesystemType::Xfs => "mkfs.xfs",
                types::StandardFilesystemType::Swap => "mkswap",
            },
        }
    }

    fn uuid_arg(&self) -> Vec<String> {
        match self {
            Filesystem::Fat32 { volume_id, .. } => {
                if let Some(id) = volume_id {
                    vec!["-i".to_string(), id.to_string()]
                } else {
                    vec![]
                }
            }
            Filesystem::Standard {
                filesystem_type, uuid, ..
            } => {
                if let Some(uuid) = uuid {
                    match filesystem_type {
                        types::StandardFilesystemType::Ext4 => vec!["-U".to_string(), uuid.to_string()],
                        types::StandardFilesystemType::F2fs => vec!["-U".to_string(), uuid.to_string()],
                        types::StandardFilesystemType::Xfs => vec!["-m".to_string(), format!("uuid={}", uuid)],
                        types::StandardFilesystemType::Swap => vec!["-U".to_string(), uuid.to_string()],
                    }
                } else {
                    vec![]
                }
            }
        }
    }

    fn label_arg(&self) -> Vec<String> {
        match self {
            Filesystem::Fat32 { label, .. } => {
                if let Some(label) = label {
                    vec!["-n".to_string(), label.to_string()]
                } else {
                    vec![]
                }
            }
            Filesystem::Standard {
                filesystem_type, label, ..
            } => {
                if let Some(label) = label {
                    match filesystem_type {
                        types::StandardFilesystemType::Ext4 => vec!["-L".to_string(), label.to_string()],
                        types::StandardFilesystemType::F2fs => vec!["-l".to_string(), label.to_string()],
                        types::StandardFilesystemType::Xfs => vec!["-L".to_string(), label.to_string()],
                        types::StandardFilesystemType::Swap => vec!["-L".to_string(), label.to_string()],
                    }
                } else {
                    vec![]
                }
            }
        }
    }

    fn force_arg(&self) -> Vec<String> {
        match self {
            Filesystem::Fat32 { .. } => vec![],
            Filesystem::Standard { filesystem_type, .. } => match filesystem_type {
                types::StandardFilesystemType::F2fs => vec!["-f".to_string()],
                types::StandardFilesystemType::Ext4 => vec!["-F".to_string()],
                types::StandardFilesystemType::Xfs => vec!["-f".to_string()],
                types::StandardFilesystemType::Swap => vec!["-f".to_string()],
            },
        }
    }
}

/// Struct for formatting filesystems on devices
pub struct Formatter {
    pub filesystem: Filesystem,
    pub force: bool,
}

impl Formatter {
    /// Creates a new Formatter for the given filesystem
    pub fn new(filesystem: Filesystem) -> Self {
        Self {
            filesystem,
            force: false,
        }
    }

    /// Forces the format operation
    pub fn force(self) -> Self {
        Self { force: true, ..self }
    }

    /// Returns a Command configured to format the given device with the filesystem
    pub fn format(&self, device: &Path) -> Command {
        let mut cmd = Command::new(self.filesystem.mkfs_command());

        cmd.args(self.filesystem.uuid_arg());
        cmd.args(self.filesystem.label_arg());
        if self.force {
            cmd.args(self.filesystem.force_arg());
        }

        cmd.arg(device);
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_fat32_args() {
        let fs = Filesystem::Fat32 {
            label: Some("BOOT".to_string()),
            volume_id: Some(1234),
        };

        assert_eq!(fs.mkfs_command(), "mkfs.fat");
        assert_eq!(fs.uuid_arg(), vec!["-i", "1234"]);
        assert_eq!(fs.label_arg(), vec!["-n", "BOOT"]);
    }

    #[test]
    fn test_ext4_args() {
        let uuid = Uuid::new_v4();
        let fs = Filesystem::Standard {
            filesystem_type: types::StandardFilesystemType::Ext4,
            label: Some("root".to_string()),
            uuid: Some(uuid.to_string()),
        };

        assert_eq!(fs.mkfs_command(), "mkfs.ext4");
        assert_eq!(fs.uuid_arg(), vec!["-U".to_string(), uuid.to_string()]);
        assert_eq!(fs.label_arg(), vec!["-L", "root"]);
    }

    #[test]
    fn test_xfs_args() {
        let uuid = Uuid::new_v4();
        let fs = Filesystem::Standard {
            filesystem_type: types::StandardFilesystemType::Xfs,
            label: Some("data".to_string()),
            uuid: Some(uuid.to_string()),
        };

        assert_eq!(fs.mkfs_command(), "mkfs.xfs");
        assert_eq!(fs.uuid_arg(), vec!["-m".to_string(), format!("uuid={uuid}")]);
        assert_eq!(fs.label_arg(), vec!["-L", "data"]);
    }
}
