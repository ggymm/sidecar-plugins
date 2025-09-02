use serde::{Deserialize, Serialize};
use std::env;
use std::net::UdpSocket;
use std::process::Command;
use sysinfo::{Disks, System};

#[derive(Serialize, Deserialize)]
pub struct BasicInfo {
    pub os: OsInfo,
    pub env: EnvInfo,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub network: Option<NetworkInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct CpuInfo {
    pub arch: String,
    pub brand: String,
    pub frequency: u64,
    pub logical_cores: usize,
    pub physical_cores: usize,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryInfo {
    pub used: u64,
    pub total: u64,
    pub total_swap: u64,
}

#[derive(Serialize, Deserialize)]
pub struct DiskInfo {
    pub disk_type: String,
    pub file_system: String,
    pub total_capacity: u64,
    pub mount_point: String,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkInfo {
    pub name: String,
    pub ip_address: String,
    pub mac_address: String,
}

#[derive(Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub serial_number: String,
    pub kernel_version: String,
}

#[derive(Serialize, Deserialize)]
pub struct EnvInfo {
    pub shell: String,
    pub current_user: String,
}

pub fn collect_basic_info() -> BasicInfo {
    let sys = System::new_all();

    // CPU 信息
    let mut cpu_info = CpuInfo {
        arch: env::consts::ARCH.to_string(),
        brand: "Unknown".to_string(),
        frequency: 0,
        logical_cores: 0,
        physical_cores: 0,
    };
    if let Some(cpu) = sys.cpus().first() {
        cpu_info = CpuInfo {
            arch: env::consts::ARCH.to_string(),
            brand: cpu.brand().to_string(),
            frequency: cpu.frequency(),
            logical_cores: sys.cpus().len(),
            physical_cores: System::physical_core_count().unwrap_or(0),
        }
    }

    // 硬盘 信息
    let disks = Disks::new_with_refreshed_list();
    let mut disks_info = Vec::new();
    for disk in &disks {
        disks_info.push(DiskInfo {
            disk_type: format!("{:?}", disk.kind()),
            file_system: disk.file_system().to_string_lossy().to_string(),
            total_capacity: disk.total_space(),
            mount_point: disk.mount_point().display().to_string(),
        });
    }

    // 网卡 信息
    let mut network_info = None;
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = socket.local_addr() {
                let local_ip = local_addr.ip();

                // Find the network interface that has this IP
                use sysinfo::Networks;
                let networks = Networks::new_with_refreshed_list();

                for (interface_name, network) in &networks {
                    // Check if this interface has the matching IP
                    for ip_network in network.ip_networks() {
                        if ip_network.addr == local_ip {
                            network_info = Some(NetworkInfo {
                                name: interface_name.clone(),
                                mac_address: network.mac_address().to_string(),
                                ip_address: local_ip.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    BasicInfo {
        os: OsInfo {
            name: System::name().unwrap_or("Unknown".to_string()),
            version: System::os_version().unwrap_or("Unknown".to_string()),
            serial_number: get_system_serial_number(),
            kernel_version: System::kernel_version().unwrap_or("Unknown".to_string()),
        },
        env: EnvInfo {
            shell: env::var("SHELL").unwrap_or("Unknown".to_string()),
            current_user: env::var("USER")
                .or_else(|_| env::var("USERNAME"))
                .unwrap_or("Unknown".to_string()),
        },
        cpu: cpu_info,
        memory: MemoryInfo {
            used: sys.used_memory(),
            total: sys.total_memory(),
            total_swap: sys.total_swap(),
        },
        disks: disks_info,
        network: network_info,
    }
}

fn get_system_serial_number() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("system_profiler").args(["SPHardwareDataType"]).output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.trim().contains("Serial Number") && line.contains(":") {
                    return line.split(':').nth(1).unwrap_or("").trim().to_string();
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("wmic")
            .args(["csproduct", "get", "identifyingnumber", "/value"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.starts_with("IdentifyingNumber=") {
                    return line.split('=').nth(1).unwrap_or("").trim().to_string();
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("dmidecode").args(["-s", "system-serial-number"]).output() {
            let serial = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !serial.is_empty() && serial != "Not Specified" {
                return serial;
            }
        }
    }

    "Unknown".to_string()
}
