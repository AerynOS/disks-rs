// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{
    fs, io,
    os::fd::{AsRawFd, OwnedFd},
};

use linux_raw_sys::loop_device::{LOOP_CLR_FD, LOOP_CTL_GET_FREE, LOOP_SET_FD, LOOP_SET_STATUS64};
use log::{debug, error, info};
use nix::libc;

/// Represents a loop device that can be used to mount files as block devices
pub struct LoopDevice {
    /// File descriptor for the loop device
    fd: OwnedFd,
    /// Path to the loop device (e.g. /dev/loop0)
    pub path: String,
}

impl LoopDevice {
    /// Creates a new loop device by obtaining the next available device number
    /// from /dev/loop-control and opening the corresponding device file.
    ///
    /// # Returns
    /// `io::Result<LoopDevice>` containing the new loop device on success
    pub fn create() -> io::Result<Self> {
        use std::fs::OpenOptions;

        debug!("Opening loop control device");
        let ctrl = OpenOptions::new().read(true).write(true).open("/dev/loop-control")?;

        // Get next free loop device number
        let devno = unsafe { libc::ioctl(ctrl.as_raw_fd(), LOOP_CTL_GET_FREE as _) };
        if devno < 0 {
            error!("Failed to acquire free loop device number");
            return Err(io::Error::last_os_error());
        }

        let path = format!("/dev/loop{devno}");
        debug!("Creating new loop device at {path}");
        let fd = OpenOptions::new().read(true).write(true).open(&path)?.into();

        info!("Successfully initialized loop device {path}");
        Ok(LoopDevice { fd, path })
    }

    /// Attaches a backing file to this loop device, allowing the file to be
    /// accessed as a block device.
    ///
    /// # Arguments
    /// * `backing_file` - Path to the file to attach
    ///
    /// # Returns
    /// `io::Result<()>` indicating success or failure
    pub fn attach(&self, backing_file: &str) -> io::Result<()> {
        debug!("Attempting to attach backing file {} to {}", backing_file, self.path);
        let f = fs::OpenOptions::new().read(true).write(true).open(backing_file)?;

        let file_fd = f.as_raw_fd();
        let our_fd = self.fd.as_raw_fd();
        let res = unsafe { libc::ioctl(our_fd, LOOP_SET_FD as _, file_fd) };

        if res < 0 {
            error!("Failed to attach backing file {backing_file} - OS error");
            return Err(io::Error::last_os_error());
        }

        // Force loop device to immediately update by setting empty status
        let info: linux_raw_sys::loop_device::loop_info64 = unsafe { std::mem::zeroed() };
        let res = unsafe { libc::ioctl(our_fd, LOOP_SET_STATUS64 as _, &info) };
        if res < 0 {
            error!("Failed to update loop device status - device may be in inconsistent state");
            return Err(io::Error::last_os_error());
        }

        info!("Successfully attached backing file {backing_file} to loop device");
        Ok(())
    }

    /// Detaches the current backing file from this loop device.
    ///
    /// # Returns
    /// `io::Result<()>` indicating success or failure
    pub fn detach(&self) -> io::Result<()> {
        debug!("Initiating detachment of backing file from {}", self.path);
        let res = unsafe { libc::ioctl(self.fd.as_raw_fd(), LOOP_CLR_FD as _, 0) };
        if res < 0 {
            error!("Failed to detach backing file from {} - OS error", self.path);
            return Err(io::Error::last_os_error());
        }

        info!("Successfully detached backing file from loop device {}", self.path);
        Ok(())
    }
}
