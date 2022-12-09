//! Managed state for the server
//!
//! The DeviceState is used to ensure that multiple commands are not run at the same time.
//! The module also contains some other components needed for the backend.

use crate::device_status::{DeviceStatus, DiskStatus, MemStatus};
use mobile_api::configs::{DeviceConfig, DeviceInfo};
use mobile_api::device_config_path;
use std::cmp::Ordering;
use std::io::ErrorKind;
use std::ops::Deref;
use std::sync::{Mutex, RwLock};
use sysinfo::{CpuExt, CpuRefreshKind, Disk, DiskExt, RefreshKind, System, SystemExt};

#[cfg(test)]
mod tests;

/// Managed state structure
pub struct DeviceState {
    /// Reason message, why is the server busy
    busy_reason: Mutex<&'static str>,

    /// Device configuration
    device_config: RwLock<Option<DeviceConfig>>,

    /// Device information
    device_info: DeviceInfo,

    /// An object for querying the system status
    sys_info: Mutex<System>,

    /// What system information is updated when the system status is queried
    sys_info_refreshes: RefreshKind,
}

/// Sorting disk information based on device file
fn sort_disks_by_device_name(a: &Disk, b: &Disk) -> Ordering {
    a.name().cmp(b.name())
}

impl DeviceState {
    /// Creating server state object
    ///
    /// Device info is prepared before starting the server
    pub fn new(device_info: DeviceInfo) -> DeviceState {
        let busy_reason = Mutex::new("");
        let device_config = RwLock::new(match DeviceConfig::load() {
            Ok(config) => Some(config),
            _ => None,
        });

        let sys_info_refreshes = RefreshKind::new()
            .with_cpu(CpuRefreshKind::new().with_cpu_usage())
            .with_memory()
            .with_disks_list();
        let mut sys = System::new_with_specifics(sys_info_refreshes);
        sys.refresh_specifics(sys_info_refreshes);
        let sys_info = Mutex::new(sys);

        DeviceState {
            busy_reason,
            device_config,
            device_info,
            sys_info,
            sys_info_refreshes,
        }
    }

    /// Check if server is busy
    ///
    /// Returns busy reason or empty str if server is free
    pub fn busy(&self) -> &'static str {
        self.busy_reason.lock().unwrap().deref()
    }

    /// Clearing server busy status
    pub fn clear_busy(&self) {
        *self.busy_reason.lock().unwrap() = "";
    }

    /// Set server busy reason message
    ///
    /// See also: [BusyGuard]
    pub fn set_busy(&self, reason: &'static str) -> Result<(), &'static str> {
        let mut guard = self.busy_reason.lock().unwrap();
        if guard.is_empty() {
            *guard = reason;
            Ok(())
        } else {
            Err(*guard)
        }
    }
    /// Requesting system status
    pub fn device_status(&self) -> DeviceStatus {
        let mut sys_info = self.sys_info.lock().unwrap();
        sys_info.refresh_specifics(self.sys_info_refreshes);
        sys_info.sort_disks_by(sort_disks_by_device_name);

        let mut cpu_usage = Vec::new();
        for cpu in sys_info.cpus() {
            cpu_usage.push(cpu.cpu_usage() * 0.01);
        }

        // Divide by zero if the computer does not have memory... unlikely
        let mem_usage = MemStatus::new(
            sys_info.total_memory(),
            sys_info.available_memory(),
            sys_info.used_memory(),
        );

        // However systems without swap do exists
        let swap_usage = if sys_info.total_swap() > 0 {
            Some(MemStatus::new(
                sys_info.total_swap(),
                sys_info.free_swap(),
                sys_info.used_swap(),
            ))
        } else {
            None
        };

        let mut disks = Vec::new();
        for disk in sys_info.disks() {
            disks.push(DiskStatus {
                device: String::from(disk.name().to_str().unwrap_or_default()),
                file_system: String::from_utf8_lossy(disk.file_system()).into(),
                total_space: disk.total_space(),
                mount_point: String::from(disk.mount_point().to_str().unwrap_or_default()),
                available_space: disk.available_space(),
                usage: if disk.total_space() > 0 {
                    1.0 - (disk.available_space() as f32 / disk.total_space() as f32)
                } else {
                    1.0
                },
            });
        }

        let uptime = sys_info.uptime();

        let load_average = [
            sys_info.load_average().one as f32,
            sys_info.load_average().five as f32,
            sys_info.load_average().fifteen as f32,
        ];

        DeviceStatus {
            cpu_usage,
            mem_usage,
            swap_usage,
            disks,
            uptime,
            load_average,
        }
    }

    /// Get a copy current config if available
    pub fn get_config(&self) -> Option<DeviceConfig> {
        if let Ok(config) = self.device_config.read() {
            config.clone()
        } else {
            None
        }
    }

    /// Set new config
    ///
    /// Given config is written to `config.json` file.
    /// Sending None will delete `config.json` file.
    pub fn set_config(
        &self,
        config: Option<DeviceConfig>,
    ) -> Result<(), Box<dyn std::error::Error + '_>> {
        let mut write_lock = self.device_config.write()?;
        match &config {
            None => {
                if let Err(err) = std::fs::remove_file(device_config_path()) {
                    match err.kind() {
                        ErrorKind::NotFound => (),   // This is okay
                        _ => return Err(err.into()), // Anything else is error
                    }
                }
            }
            Some(config) => config.save()?,
        }
        *write_lock = config;
        Ok(())
    }

    /// Access device info reference
    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }
}

/// Guardian for server busy messages
///
/// The guardian automatically clears the busy message when the object goes out of scope.
///
/// # Example
///
/// ```rust
/// match BusyGuard::try_busy(state, "Calculating the meaning of life") {
///     Ok(_) => {
///         // Making heavy calculations here...
///         CommandResponse::TextOk("42"),
///     }   // Guard object goes out of scope here and busy message is cleared
///
///     // Server is already busy with other task
///     Err(reason) => CommandResponse::Busy(reason),
/// }
/// ```
pub struct BusyGuard<'a> {
    /// Reference to state object
    state: &'a DeviceState,
}

impl BusyGuard<'_> {
    /// Tries to make system busy
    ///
    /// If the server is free, then it is marked busy with the *reason* and guardian object is
    /// returned in Ok.
    ///
    /// If the serve is busy, then the reason is returned in the Err.
    pub fn try_busy<'a>(
        state: &'a DeviceState,
        reason: &'static str,
    ) -> Result<BusyGuard<'a>, &'static str> {
        match state.set_busy(reason) {
            Ok(_) => Ok(BusyGuard { state }),
            Err(reason) => Err(reason),
        }
    }
}

impl Drop for BusyGuard<'_> {
    /// Clearing busy message when guardian goes out of scope
    fn drop(&mut self) {
        self.state.clear_busy();
    }
}
