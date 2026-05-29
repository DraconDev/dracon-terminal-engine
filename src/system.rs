//! System monitoring types for CPU, memory, disk, and process information.
//!
//! This module provides data structures for system monitoring (behind the `system` feature):
//!
//! - [`DiskInfo`] — Mounted disk usage information
//! - [`ProcessInfo`] — Running process details
//! - [`SystemMonitor`] — Aggregated system statistics
//!
//! Data is obtained via the `sysinfo` crate.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use sysinfo::{Disks, ProcessesToUpdate, System};

/// Information about a single mounted or unmounted disk.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    /// Display name (mount point or label).
    pub name: String,
    /// Device path (e.g. `/dev/sda1`).
    pub device: String,
    /// Used space in bytes.
    pub used_space: f64,
    /// Available (free) space in bytes.
    pub available_space: f64,
    /// Total capacity in bytes.
    pub total_space: f64,
    /// Whether the disk is currently mounted.
    pub is_mounted: bool,
}

/// Information about a single running process.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID.
    pub pid: u32,
    /// Parent Process ID (if available).
    pub ppid: Option<u32>,
    /// Process name.
    pub name: String,
    /// CPU usage percentage.
    pub cpu: f32,
    /// Memory usage in MiB.
    pub mem: f32,
    /// Username of the process owner.
    pub user: String,
    /// Process scheduler state.
    pub status: String,
}

/// Aggregated system information snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemData {
    /// Overall CPU usage percentage (0–100).
    pub cpu_usage: f32,
    /// Per-core CPU usage percentages.
    pub cpu_cores: Vec<f32>,
    /// Used memory in GiB.
    pub mem_usage: f64,
    /// Total memory in GiB.
    pub total_mem: f64,
    /// Used swap in GiB.
    pub swap_usage: f64,
    /// Total swap in GiB.
    pub total_swap: f64,
    /// Disk information list.
    pub disks: Vec<DiskInfo>,
    /// Top processes by CPU usage.
    pub processes: Vec<ProcessInfo>,
    /// Total bytes received across all interfaces.
    pub net_in: u64,
    /// Total bytes transmitted across all interfaces.
    pub net_out: u64,
    /// System uptime in seconds.
    pub uptime: u64,
    /// Operating system name.
    pub os_name: String,
    /// OS version string.
    pub os_version: String,
    /// Kernel version string.
    pub kernel_version: String,
    /// Host name.
    pub hostname: String,
}

/// System resource monitor that periodically refreshes CPU, memory, disks, and process data.
pub struct SystemMonitor {
    sys: System,
    disks: Disks,
    networks: sysinfo::Networks,
    users: sysinfo::Users,
    last_process_refresh: Instant,
    process_refresh_interval: Duration,
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMonitor {
    /// Creates a new `SystemMonitor` and immediately refreshes all system data.
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = sysinfo::Networks::new_with_refreshed_list();
        let users = sysinfo::Users::new_with_refreshed_list();
        Self {
            sys,
            disks,
            networks,
            users,
            last_process_refresh: Instant::now(),
            process_refresh_interval: Duration::from_secs(2),
        }
    }

    /// Refreshes all system data and returns a `SystemData` snapshot.
    pub fn get_data(&mut self) -> SystemData {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        // Only refresh processes periodically (every 2 seconds by default)
        if self.last_process_refresh.elapsed() >= self.process_refresh_interval {
            self.sys.refresh_processes(ProcessesToUpdate::All, true);
            self.last_process_refresh = Instant::now();
        }
        self.disks.refresh_list();
        self.networks.refresh_list();
        self.users.refresh_list();

        let cpu_usage = self.sys.global_cpu_usage();
        let cpu_cores = self.sys.cpus().iter().map(|c| c.cpu_usage()).collect();
        let mem_usage = self.sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0; // GB
        let total_mem = self.sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0; // GB
        let swap_usage = self.sys.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0; // GB
        let total_swap = self.sys.total_swap() as f64 / 1024.0 / 1024.0 / 1024.0; // GB

        let mut final_processes = Vec::new();
        for (pid, process) in self.sys.processes() {
            let user = process
                .user_id()
                .and_then(|uid| self.users.iter().find(|u| u.id() == uid))
                .map(|u| u.name().to_string())
                .unwrap_or_else(|| "root".to_string());

            final_processes.push(ProcessInfo {
                pid: pid.as_u32(),
                ppid: process.parent().map(|p| p.as_u32()),
                name: process.name().to_string_lossy().to_string(),
                cpu: process.cpu_usage(),
                mem: process.memory() as f32 / 1024.0 / 1024.0, // MB
                user,
                status: format!("{:?}", process.status()),
            });
        }
        final_processes.sort_by(|a, b| {
            b.cpu
                .partial_cmp(&a.cpu)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        final_processes.truncate(200);

        let mut net_in = 0;
        let mut net_out = 0;
        for (_, data) in &self.networks {
            net_in += data.received();
            net_out += data.transmitted();
        }

        SystemData {
            cpu_usage,
            cpu_cores,
            mem_usage,
            total_mem,
            swap_usage,
            total_swap,
            disks: self.get_disk_data(),
            processes: final_processes,
            net_in,
            net_out,
            uptime: System::uptime(),
            os_name: System::name().unwrap_or_default(),
            os_version: System::os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            hostname: System::host_name().unwrap_or_default(),
        }
    }
    fn get_disk_data(&mut self) -> Vec<DiskInfo> {
        let mut final_disks = Vec::new();
        // (Existing disk logic moved to helper)
        for disk in self.disks.iter() {
            let mount = disk.mount_point().to_string_lossy();
            let fs_type = disk.file_system().to_string_lossy().to_lowercase();
            let device = disk.name().to_string_lossy().to_string();

            if mount == "/" {
                final_disks.push(DiskInfo {
                    name: mount.to_string(),
                    device,
                    used_space: (disk.total_space() - disk.available_space()) as f64,
                    available_space: disk.available_space() as f64,
                    total_space: disk.total_space() as f64,
                    is_mounted: true,
                });
                continue;
            }

            let is_real_fs = fs_type.contains("ext")
                || fs_type.contains("btrfs")
                || fs_type.contains("xfs")
                || fs_type.contains("zfs")
                || fs_type.contains("vfat")
                || fs_type.contains("fat")
                || fs_type.contains("ntfs")
                || fs_type.contains("exfat")
                || fs_type.contains("fuseblk");

            let is_removable_path = mount.starts_with("/media")
                || mount.starts_with("/mnt")
                || mount.starts_with("/run/media");
            let is_system_path = (mount.starts_with("/boot")
                || mount.starts_with("/nix")
                || mount.starts_with("/run")
                || mount.starts_with("/sys")
                || mount.starts_with("/proc")
                || mount.starts_with("/dev")
                || mount.starts_with("/tmp"))
                && !is_removable_path;

            if is_real_fs
                && (is_removable_path || !is_system_path)
                && disk.total_space() > 100_000_000
            {
                final_disks.push(DiskInfo {
                    name: mount.to_string(),
                    device,
                    used_space: (disk.total_space() - disk.available_space()) as f64,
                    available_space: disk.available_space() as f64,
                    total_space: disk.total_space() as f64,
                    is_mounted: true,
                });
            }
        }

        // 2. Supplement with unmounted drives from lsblk (Linux only)
        #[cfg(target_os = "linux")]
        if let Ok(output) = std::process::Command::new("lsblk")
            .arg("-rnbo")
            .arg("NAME,FSTYPE,SIZE,MOUNTPOINT,LABEL")
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('\x00').filter(|s| !s.is_empty()).collect();
                if parts.len() < 4 {
                    continue;
                }
                let name = parts[0];
                let fstype = parts[1];
                let size_str = parts[2];
                let mountpoint = parts.get(3).unwrap_or(&"");
                let label = parts.get(4).unwrap_or(&"");

                if !fstype.is_empty() && mountpoint.is_empty() {
                    if let Ok(size) = size_str.parse::<f64>() {
                        if size > 100_000_000.0 {
                            let dev_path = format!("/dev/{}", name);
                            let display_name = if !label.is_empty() {
                                label.to_string()
                            } else {
                                let gb = size / 1_073_741_824.0;
                                if gb >= 1.0 {
                                    format!("{:.0}G Drive", gb)
                                } else {
                                    format!("{:.0}M Drive", size / 1_048_576.0)
                                }
                            };

                            if fstype != "swap" && !fstype.contains("member") {
                                final_disks.push(DiskInfo {
                                    name: display_name,
                                    device: dev_path,
                                    used_space: 0.0,
                                    available_space: size,
                                    total_space: size,
                                    is_mounted: false,
                                });
                            }
                        }
                    }
                }
            }
        }
        final_disks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_info_creation() {
        let disk = DiskInfo {
            name: "/".to_string(),
            device: "/dev/sda1".to_string(),
            used_space: 50_000_000_000.0,
            available_space: 450_000_000_000.0,
            total_space: 500_000_000_000.0,
            is_mounted: true,
        };
        assert_eq!(disk.name, "/");
        assert_eq!(disk.device, "/dev/sda1");
        assert!(disk.is_mounted);
        assert!(disk.total_space > disk.used_space);
    }

    #[test]
    fn test_disk_info_clone() {
        let disk = DiskInfo {
            name: "/home".to_string(),
            device: "/dev/sda2".to_string(),
            used_space: 100_000_000_000.0,
            available_space: 400_000_000_000.0,
            total_space: 500_000_000_000.0,
            is_mounted: true,
        };
        let cloned = disk.clone();
        assert_eq!(cloned.name, disk.name);
        assert_eq!(cloned.total_space, disk.total_space);
    }

    #[test]
    fn test_process_info_creation() {
        let proc = ProcessInfo {
            pid: 1234,
            ppid: Some(1),
            name: "test_process".to_string(),
            cpu: 25.5,
            mem: 128.0,
            user: "testuser".to_string(),
            status: "Running".to_string(),
        };
        assert_eq!(proc.pid, 1234);
        assert_eq!(proc.ppid, Some(1));
        assert_eq!(proc.name, "test_process");
        assert!(proc.cpu >= 0.0);
        assert!(proc.mem >= 0.0);
    }

    #[test]
    fn test_process_info_no_ppid() {
        let proc = ProcessInfo {
            pid: 1,
            ppid: None,
            name: "init".to_string(),
            cpu: 0.0,
            mem: 10.0,
            user: "root".to_string(),
            status: "Sleeping".to_string(),
        };
        assert_eq!(proc.ppid, None);
    }

    #[test]
    fn test_process_info_clone() {
        let proc = ProcessInfo {
            pid: 5678,
            ppid: Some(1000),
            name: "clone_test".to_string(),
            cpu: 50.0,
            mem: 256.0,
            user: "admin".to_string(),
            status: "Running".to_string(),
        };
        let cloned = proc.clone();
        assert_eq!(cloned.pid, proc.pid);
        assert_eq!(cloned.name, proc.name);
    }

    #[test]
    fn test_system_data_creation() {
        let data = SystemData {
            cpu_usage: 45.0,
            cpu_cores: vec![40.0, 50.0, 45.0, 45.0],
            mem_usage: 8.5,
            total_mem: 16.0,
            swap_usage: 0.0,
            total_swap: 32.0,
            disks: vec![],
            processes: vec![],
            net_in: 1_000_000,
            net_out: 500_000,
            uptime: 3600,
            os_name: "Linux".to_string(),
            os_version: "6.1.0".to_string(),
            kernel_version: "6.1.0-amd64".to_string(),
            hostname: "localhost".to_string(),
        };
        assert_eq!(data.cpu_usage, 45.0);
        assert_eq!(data.cpu_cores.len(), 4);
        assert_eq!(data.mem_usage, 8.5);
        assert!(data.uptime > 0);
    }

    #[test]
    fn test_system_data_with_disks_and_processes() {
        let disk = DiskInfo {
            name: "/".to_string(),
            device: "/dev/sda1".to_string(),
            used_space: 100_000_000_000.0,
            available_space: 400_000_000_000.0,
            total_space: 500_000_000_000.0,
            is_mounted: true,
        };

        let proc = ProcessInfo {
            pid: 1000,
            ppid: Some(1),
            name: "systemd".to_string(),
            cpu: 0.5,
            mem: 50.0,
            user: "root".to_string(),
            status: "Sleeping".to_string(),
        };

        let data = SystemData {
            cpu_usage: 10.0,
            cpu_cores: vec![10.0],
            mem_usage: 4.0,
            total_mem: 16.0,
            swap_usage: 2.0,
            total_swap: 8.0,
            disks: vec![disk],
            processes: vec![proc],
            net_in: 0,
            net_out: 0,
            uptime: 86400,
            os_name: "Linux".to_string(),
            os_version: "5.15.0".to_string(),
            kernel_version: "5.15.0-generic".to_string(),
            hostname: "server".to_string(),
        };

        assert_eq!(data.disks.len(), 1);
        assert_eq!(data.processes.len(), 1);
        assert_eq!(data.disks[0].name, "/");
        assert_eq!(data.processes[0].name, "systemd");
    }

    #[test]
    fn test_system_data_clone() {
        let data = SystemData {
            cpu_usage: 25.0,
            cpu_cores: vec![25.0, 25.0],
            mem_usage: 8.0,
            total_mem: 32.0,
            swap_usage: 0.0,
            total_swap: 16.0,
            disks: vec![],
            processes: vec![],
            net_in: 1_000_000_000,
            net_out: 500_000_000,
            uptime: 100000,
            os_name: "Linux".to_string(),
            os_version: "6.0.0".to_string(),
            kernel_version: "6.0.0".to_string(),
            hostname: "test".to_string(),
        };
        let cloned = data.clone();
        assert_eq!(cloned.cpu_usage, data.cpu_usage);
        assert_eq!(cloned.uptime, data.uptime);
    }
}
