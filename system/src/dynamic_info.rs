use serde::{Deserialize, Serialize};
use sysinfo::{Disks, System};

#[derive(Serialize, Deserialize)]
pub struct DynamicInfo {
    pub usage: UsageInfo,
    pub processes: Vec<ProcessInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct UsageInfo {
    pub cpu: f32,
    pub disks: Vec<DiskUsage>,
    pub memory: MemoryUsage,
    pub load_average: LoadAverage,
}

#[derive(Serialize, Deserialize)]
pub struct MemoryUsage {
    pub used: u64,
    pub total: u64,
}

#[derive(Serialize, Deserialize)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Serialize, Deserialize)]
pub struct DiskUsage {
    pub used: u64,
    pub total: u64,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub mount_point: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub usage_cpu: f32,
    pub usage_memory: u64,
}

pub fn collect_dynamic_info() -> DynamicInfo {
    DynamicInfo {
        usage: collect_usage_info(),
        processes: collect_processes_info(),
    }
}

fn collect_usage_info() -> UsageInfo {
    let sys = System::new_all();
    let load_avg = System::load_average();

    // 磁盘使用情况
    let disks = Disks::new_with_refreshed_list();
    let disk_usage: Vec<DiskUsage> = disks
        .iter()
        .map(|disk| {
            let usage = disk.usage();
            let total = disk.total_space();
            DiskUsage {
                mount_point: disk.mount_point().display().to_string(),
                used: total - disk.available_space(),
                total,
                read_bytes: usage.read_bytes,
                write_bytes: usage.written_bytes,
            }
        })
        .collect();

    UsageInfo {
        cpu: sys.global_cpu_usage(),
        disks: disk_usage,
        memory: MemoryUsage {
            used: sys.used_memory(),
            total: sys.total_memory(),
        },
        load_average: LoadAverage {
            one: load_avg.one,
            five: load_avg.five,
            fifteen: load_avg.fifteen,
        },
    }
}

fn collect_processes_info() -> Vec<ProcessInfo> {
    let sys = System::new_all();

    let mut processes: Vec<_> = sys
        .processes()
        .iter()
        .filter_map(|(pid, process)| {
            if let Some(exe_path) = process.exe() {
                let app_path = exe_path.to_string_lossy();

                #[cfg(target_os = "macos")]
                {
                    if app_path.contains(".app/") {
                        let app_name = app_path
                            .split('/')
                            .find(|part| part.ends_with(".app"))
                            .unwrap_or(&process.name().to_string_lossy())
                            .to_string();

                        return Some(ProcessInfo {
                            pid: pid.as_u32(),
                            name: app_name,
                            usage_cpu: process.cpu_usage(),
                            usage_memory: process.memory(),
                        });
                    }
                }
            }

            None
        })
        .collect();

    processes.sort_by(|a, b| b.usage_memory.cmp(&a.usage_memory));
    processes.truncate(5);
    processes
}
