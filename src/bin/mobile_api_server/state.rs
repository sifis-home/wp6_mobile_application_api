//! Managed state for the server
//!
//! The DeviceState is used to ensure that multiple commands are not run at the same time.
//! The module also contains some other components needed for the backend.

use crate::device_status::{DeviceStatus, DiskStatus, MemStatus};
use mobile_api::configs::{DeviceConfig, DeviceInfo};
use mobile_api::SifisHome;
use std::cmp::Ordering;
use std::env;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};
use sysinfo::{CpuExt, CpuRefreshKind, Disk, DiskExt, RefreshKind, System, SystemExt};

/// Managed state structure
pub struct DeviceState {
    /// SIFIS Home configurations instance
    sifis_home: SifisHome,

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
    /// Function tries to load device info and return DeviceState for the server on success.
    ///
    /// If something goes wrong, then message is returned as error
    pub fn new(sifis_home: SifisHome) -> Result<DeviceState, String> {
        // Try to load device info
        let device_info = match sifis_home.load_info() {
            Ok(device_info) => device_info,
            Err(error) => {
                // Special message for file not found error
                if let mobile_api::error::ErrorKind::IoError(io_error) = error.kind() {
                    if io_error.kind() == std::io::ErrorKind::NotFound {
                        return Err(format!(
                            "Device information file {:?} not found.\n\
                             You can use create_device_info application to create it.",
                            sifis_home.info_file_path()
                        ));
                    }
                };

                // Error message for any other error
                return Err(format!(
                    "Could not load device information file: {:?}\n{}",
                    sifis_home.info_file_path(),
                    error
                ));
            }
        };

        let busy_reason = Mutex::new("");
        let device_config = RwLock::new(sifis_home.load_config().ok());

        let sys_info_refreshes = RefreshKind::new()
            .with_cpu(CpuRefreshKind::new().with_cpu_usage())
            .with_memory()
            .with_disks_list();
        let mut sys = System::new_with_specifics(sys_info_refreshes);
        sys.refresh_specifics(sys_info_refreshes);
        let sys_info = Mutex::new(sys);

        Ok(DeviceState {
            sifis_home,
            busy_reason,
            device_config,
            device_info,
            sys_info,
            sys_info_refreshes,
        })
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
            None => self.sifis_home.remove_config()?,
            Some(config) => self.sifis_home.save_config(config)?,
        }
        *write_lock = config;
        Ok(())
    }

    /// Access device info reference
    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    /// Try to find requested resource path
    ///
    /// This function tries to find requested relative path in the following order:
    ///
    /// 1. From SIFIS-Home path
    /// 2. From current dir
    /// 3. From exe dir
    /// 4. From CARGO_MANIFEST_DIR
    ///
    pub fn resource_path(&self, path: &str) -> Result<PathBuf, std::io::Error> {
        // Try to find from SIFIS Home path
        let mut target_path = PathBuf::from(self.sifis_home.home_path());
        target_path.push(path);
        if target_path.exists() {
            return Ok(target_path);
        }

        // Try to find from current dir
        if let Ok(mut target_path) = env::current_dir() {
            target_path.push(path);
            if target_path.exists() {
                return Ok(target_path);
            }
        }

        // Try to find from current exe dir
        if let Ok(target_path) = env::current_exe() {
            if let Some(target_path) = target_path.parent() {
                let mut target_path = PathBuf::from(target_path);
                target_path.push(path);
                if target_path.exists() {
                    return Ok(target_path);
                }
            }
        }

        // Try to find from CARGO_MANIFEST_DIR
        if let Ok(target_path) = env::var("CARGO_MANIFEST_DIR") {
            let mut target_path = PathBuf::from(target_path);
            target_path.push(path);
            if target_path.exists() {
                return Ok(target_path);
            }
        }

        Err(std::io::Error::from(std::io::ErrorKind::NotFound))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_v1::tests_common::create_test_state;

    // Test ignored for Miri because the server has time and io-related
    // functions that are not available in isolation mode
    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_busy_guard() {
        // Shouldn't be busy at start
        let (_, state) = create_test_state();
        assert_eq!(state.busy(), "");

        // Making "server" busy
        let busy_message = "Testing BusyGuard";
        {
            let guard = BusyGuard::try_busy(&state, busy_message);
            assert!(guard.is_ok());
            assert_eq!(state.busy(), busy_message);

            // Second guard should also fail with the busy message
            let result = BusyGuard::try_busy(&state, busy_message);
            assert!(result.is_err());
            assert_eq!(result.err().unwrap(), busy_message);
        }

        // Busy guard went out of scope, "server" should be free now.
        assert_eq!(state.busy(), "");
    }
}
