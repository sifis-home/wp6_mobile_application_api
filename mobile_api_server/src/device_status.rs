//! Device status structures
//!
//! System status information is collected into these structures
//! and sent to the client application in JSON format.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Memory information
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MemStatus {
    /// Total available memory in bytes
    pub total: u64,

    /// Amount of free memory in bytes
    ///
    /// For the RAM, we return available memory instead of free memory,
    /// as that is what regular users expect.
    pub free: u64,

    /// Amount of used RAM in bytes
    pub used: u64,

    /// Memory usage
    ///
    /// Memory usage is between zero and one, where zero is 0% and one is 100%.
    pub usage: f32,
}

impl MemStatus {
    /// Convenience function that calculates usage percentage from total and used
    pub fn new(total: u64, free: u64, used: u64) -> MemStatus {
        MemStatus {
            total,
            free,
            used,
            usage: used as f32 / total as f32,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
/// Disk information
pub struct DiskStatus {
    /// Device file
    pub device: String,

    /// Filesystem name
    pub file_system: String,

    /// Total diskspace in bytes
    pub total_space: u64,

    /// Mount point of the disk
    pub mount_point: String,

    /// Available disk space in bytes
    pub available_space: u64,

    /// Disk space usage
    ///
    /// Disk space usage is between zero and one, where zero is 0% and one is 100%.
    pub usage: f32,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
/// A collection of system information
pub struct DeviceStatus {
    /// CPU usage per core
    ///
    /// CPU usage is between zero and one, where zero is 0% and one is 100%.
    /// The array contains a value for each CPU core.
    pub cpu_usage: Vec<f32>,

    /// RAM information
    pub mem_usage: MemStatus,

    /// Swap information when available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_usage: Option<MemStatus>,

    /// A collection of disk information
    pub disks: Vec<DiskStatus>,

    /// System uptime in seconds
    pub uptime: u64,

    /// Load average values for 1 min, 5 min, and 15 min
    pub load_average: [f32; 3],
}
