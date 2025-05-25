// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

/// Format a size in bytes into a human readable string
/// Format a byte size into a human-readable string with appropriate units
///
/// # Examples
///
/// ```
/// use disks::format_size;
/// assert_eq!(format_size(1500), "1.5KiB");
/// assert_eq!(format_size(1500000), "1.4MiB");
/// ```
pub fn format_size(size: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let size = size as f64;
    if size >= TB {
        format!("{:.1}TiB", size / TB)
    } else if size >= GB {
        format!("{:.1}GiB", size / GB)
    } else if size >= MB {
        format!("{:.1}MiB", size / MB)
    } else if size >= KB {
        format!("{:.1}KiB", size / KB)
    } else {
        format!("{size}B")
    }
}

/// Format a disk position as a percentage and absolute size
/// Format a disk position as both a percentage and absolute size
///
/// This is useful for displaying partition locations in a user-friendly way.
///
/// # Examples
///
/// ```
/// use disks::format_position;
/// let total = 1000;
/// assert_eq!(format_position(500, total), "50% (500B)");
/// ```
pub fn format_position(pos: u64, total: u64) -> String {
    format!("{}% ({})", (pos as f64 / total as f64 * 100.0) as u64, format_size(pos))
}

/// Check if a value is already aligned to the given boundary
pub fn is_aligned(value: u64, alignment: u64) -> bool {
    value % alignment == 0
}

/// Align up to the nearest multiple of alignment, unless already aligned
pub fn align_up(value: u64, alignment: u64) -> u64 {
    match value % alignment {
        0 => value,
        remainder if remainder > (alignment / 2) => value + (alignment - remainder),
        remainder => value - remainder,
    }
}

/// Align down to the nearest multiple of alignment, unless already aligned
pub fn align_down(value: u64, alignment: u64) -> u64 {
    match value % alignment {
        0 => value,
        remainder if remainder < (alignment / 2) => value - remainder,
        remainder => value + (alignment - remainder),
    }
}
