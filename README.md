# disks-rs 💽

This project began life in the [blsforme](https://github.com/aerynos/blsforme) project for AerynOS.
However as time went on it became clear we needed to extend the capabilities beyond simple topology scanning
and superblocks to support the installer and other use cases.

Importantly due to using blsforme in moss, and requiring static linking to avoid soname breakage on updates,
we were unable to leverage `libblkid` due to licensing incompatibilities.

## Goals 🎯

Provide safe and sane APIs for dealing with filesystems, block devices and partitioning in Rust. The intent
is to provide a high level API that can be used to build tools like installers, partitioners, and other disk
management tools.

With support, we will also provide the foundations for a Rust implementation of `libblkid`, while also providing
an alternative to `libparted`.

Per [issue 3](https://github.com/aerynos/disks-rs/issues/3) we do eventually plan to extend the superblock support
to have in-tree capabilities for writing filesystems, but this is a long term goal. TLDR generation of complete filesystem
images / rootfs without `euid = 0` requirements. If you want to make this happen faster, then read the next section. 😉

## Support Us ❤️

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/J3J511WM9N)

[![GitHub Sponsors](https://img.shields.io/github/sponsors/ikeycode?style=for-the-badge&logo=github&label=Sponsor)](https://github.com/sponsors/ikeycode)

## Crates 📦

- `disks` - A simplistic enumeration API built atop `sysfs` for discovering block devices and partitions.
- `superblock` - Pure Rust superblock parsing for various filesystems. Version-specific oddities and more filesystems
    will be added over time.

    Currently we support:

    - `luks2` - LUKS2 superblock parsing.
    - `ext4` - Ext4 superblock parsing.
    - `f2fs` - F2FS superblock parsing.
    - `btrfs` - Btrfs superblock parsing.
    - `xfs` - XFS superblock parsing.

- `partitioning` - A partitioning API for manipulating partition tables on block devices. This will be built atop
    `disks` and `superblock` to provide a high level API for partitioning. Currently focused on `gpt`.

    - The `loopback` module provides a way to create loopback devices and bind them for testing.
    - Notifying the kernel of partition table changes is supported for GPT (BLKPG).
    - The `planner` module is provided to assist in planning partitioning operations (undo support included)
    - The `strategy` module builds on top of `planner` to facilitate computation of partition layouts including
      disk wipe, dual boot scenarios, etc.

## License

`disks-rs` is available under the terms of the [MPL-2.0](https://spdx.org/licenses/MPL-2.0.html)
