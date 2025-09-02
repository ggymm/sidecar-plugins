use serde::{Deserialize, Serialize};
use sysinfo::{Components, ProcessesToUpdate, System};

#[derive(Serialize, Deserialize)]
pub struct DynamicInfo {
    pub processes: Vec<ProcessInfo>,
    pub temperature: Vec<TemperatureInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub user: String,
}

#[derive(Serialize, Deserialize)]
pub struct TemperatureInfo {
    pub component: String,
    pub temperature: f32,
    pub critical: Option<f32>,
    pub max: Option<f32>,
}

pub fn collect_dynamic_info() -> DynamicInfo {
    DynamicInfo {
        processes: collect_top_processes(),
        temperature: collect_temperature_info(),
    }
}

fn collect_top_processes() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut processes: Vec<_> = sys
        .processes()
        .iter()
        .filter_map(|(pid, process)| {
            if let Some(exe_path) = process.exe() {
                let path_str = exe_path.to_string_lossy();

                #[cfg(target_os = "macos")]
                {
                    if path_str.contains("/Applications/") && path_str.contains(".app/") {
                        let display_name = if let Some(app_pos) = path_str.find(".app/") {
                            if let Some(start) = path_str[..app_pos].rfind('/') {
                                path_str[start + 1..app_pos].to_string()
                            } else {
                                process.name().to_string_lossy().to_string()
                            }
                        } else {
                            process.name().to_string_lossy().to_string()
                        };

                        return Some(ProcessInfo {
                            name: display_name,
                            pid: pid.as_u32(),
                            cpu_usage: process.cpu_usage(),
                            memory_usage: process.memory(),
                            user: process
                                .user_id()
                                .map(|uid| uid.to_string())
                                .unwrap_or_else(|| "Unknown".to_string()),
                        });
                    }
                }

                #[cfg(target_os = "windows")]
                {
                    if path_str.contains("\\Program Files\\") || path_str.contains("\\Program Files (x86)\\") {
                        let display_name = if let Some(exe_name) = exe_path.file_stem() {
                            exe_name.to_string_lossy().to_string()
                        } else {
                            process.name().to_string_lossy().to_string()
                        };

                        return Some(ProcessInfo {
                            name: display_name,
                            pid: pid.as_u32(),
                            cpu_usage: process.cpu_usage(),
                            memory_usage: process.memory(),
                            user: process
                                .user_id()
                                .map(|uid| uid.to_string())
                                .unwrap_or_else(|| "Unknown".to_string()),
                        });
                    }
                }
            }

            None
        })
        .collect();

    processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
    processes.truncate(5);
    processes
}

fn collect_temperature_info() -> Vec<TemperatureInfo> {
    let components = Components::new_with_refreshed_list();

    let mut temperatures: Vec<_> = components
        .iter()
        .filter_map(|component| {
            component.temperature().map(|temp| TemperatureInfo {
                component: component.label().to_string(),
                temperature: temp,
                critical: component.critical(),
                max: component.max(),
            })
        })
        .collect();

    // 按温度降序排序
    temperatures.sort_by(|a, b| {
        b.temperature
            .partial_cmp(&a.temperature)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    temperatures
}
