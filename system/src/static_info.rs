use serde::{Deserialize, Serialize};
use std::env;
use std::net::UdpSocket;
use sysinfo::{Disks, System};

#[derive(Serialize, Deserialize)]
pub struct CpuInfo {
    pub brand: String,
    pub architecture: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub base_frequency: u64,
    pub max_frequency: u64,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub total_swap: u64,
}

#[derive(Serialize, Deserialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_capacity: u64,
    pub disk_type: String,
    pub file_system: String,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub mac_address: String,
    pub ip_address: String,
}

#[derive(Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub network_interface: Option<NetworkInterfaceInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub kernel_version: String,
    pub hostname: String,
    pub architecture: String,
    pub boot_time: String,
}

#[derive(Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub current_user: String,
    pub home_directory: String,
    pub shell: String,
    pub terminal: String,
}

#[derive(Serialize, Deserialize)]
pub struct SoftwareInfo {
    pub os: OsInfo,
    pub environment: EnvironmentInfo,
}

#[derive(Serialize, Deserialize)]
pub struct StaticInfo {
    pub hardware: HardwareInfo,
    pub software: SoftwareInfo,
}

#[derive(Serialize, Deserialize)]
pub struct SystemInfoOutput {
    pub timestamp: String,
    #[serde(rename = "static")]
    pub static_info: StaticInfo,
}

pub fn collect_static_info() -> SystemInfoOutput {
    let mut sys = System::new_all();
    sys.refresh_all();

    // CPU Info
    let cpu_info = if let Some(cpu) = sys.cpus().first() {
        CpuInfo {
            brand: cpu.brand().to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            physical_cores: sys.physical_core_count().unwrap_or(0),
            logical_cores: sys.cpus().len(),
            base_frequency: cpu.frequency(),
            max_frequency: cpu.frequency(),
        }
    } else {
        CpuInfo {
            brand: "Unknown".to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            physical_cores: 0,
            logical_cores: 0,
            base_frequency: 0,
            max_frequency: 0,
        }
    };

    // Memory Info
    let memory_info = MemoryInfo {
        total: sys.total_memory(),
        total_swap: sys.total_swap(),
    };

    // Storage Devices
    let disks = Disks::new_with_refreshed_list();
    let disk_info: Vec<DiskInfo> = disks
        .iter()
        .map(|disk| DiskInfo {
            mount_point: disk.mount_point().display().to_string(),
            total_capacity: disk.total_space(),
            disk_type: format!("{:?}", disk.kind()),
            file_system: disk.file_system().to_string_lossy().to_string(),
        })
        .collect();

    // Network Interfaces - Get active LAN interface
    let network_info = get_active_network_interface();

    // Hardware
    let hardware = HardwareInfo {
        cpu: cpu_info,
        memory: memory_info,
        disks: disk_info,
        network_interface: network_info,
    };

    // OS Info
    let os_info = OsInfo {
        name: System::name().unwrap_or("Unknown".to_string()),
        version: System::os_version().unwrap_or("Unknown".to_string()),
        kernel_version: System::kernel_version().unwrap_or("Unknown".to_string()),
        hostname: System::host_name().unwrap_or("Unknown".to_string()),
        architecture: std::env::consts::ARCH.to_string(),
        boot_time: "TBD".to_string(),
    };

    // Environment Info
    let environment = EnvironmentInfo {
        current_user: env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or("Unknown".to_string()),
        home_directory: env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .unwrap_or("Unknown".to_string()),
        shell: env::var("SHELL").unwrap_or("Unknown".to_string()),
        terminal: env::var("TERM").unwrap_or("Unknown".to_string()),
    };

    // Software
    let software = SoftwareInfo {
        os: os_info,
        environment,
    };

    // Static Info
    let static_info = StaticInfo { hardware, software };

    SystemInfoOutput {
        timestamp: chrono::Utc::now().to_rfc3339(),
        static_info,
    }
}

fn get_active_network_interface() -> Option<NetworkInterfaceInfo> {
    // Use UDP socket to determine active network interface and IP
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = socket.local_addr() {
                let local_ip = local_addr.ip().to_string();

                // Try to find the network interface with this IP
                use sysinfo::Networks;
                let networks = Networks::new_with_refreshed_list();

                for (interface_name, network) in &networks {
                    // Skip virtual interfaces
                    if interface_name.starts_with("vmnet")
                        || interface_name.starts_with("utun")
                        || interface_name.starts_with("bridge")
                        || interface_name.starts_with("lo")
                        || interface_name.starts_with("gif")
                        || interface_name.starts_with("stf")
                        || interface_name.starts_with("awdl")
                        || interface_name.starts_with("llw")
                        || interface_name.starts_with("anpi")
                        || interface_name.starts_with("ap")
                        || network.mac_address().to_string() == "00:00:00:00:00:00"
                    {
                        continue;
                    }

                    // Use the first valid physical interface with the detected IP
                    return Some(NetworkInterfaceInfo {
                        name: interface_name.clone(),
                        mac_address: network.mac_address().to_string(),
                        ip_address: local_ip,
                    });
                }
            }
        }
    }

    // Fallback: return None if no active interface found
    None
}
