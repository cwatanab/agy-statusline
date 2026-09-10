use std::env;
use std::fs;
use std::path::PathBuf;

pub struct SysInfo {
    pub mem_pct: Option<u32>,
    pub load_1m: Option<String>,
}

pub struct PowerInfo {
    pub is_ac: bool,
    pub battery_pct: Option<u32>,
}

/// Zero-process Git branch lookup directly reading .git/HEAD
pub fn git_info<'a>(working_dir: &'a str, parsed_branch: &'a str, parsed_dirty: bool) -> (std::borrow::Cow<'a, str>, bool) {
    if !parsed_branch.is_empty() {
        return (std::borrow::Cow::Borrowed(parsed_branch), parsed_dirty);
    }

    let start_dir = if working_dir.is_empty() { "." } else { working_dir };
    let mut curr = PathBuf::from(start_dir);

    // Search upwards for .git
    for _ in 0..10 {
        let git_path = curr.join(".git");
        if git_path.exists() {
            let head_path = if git_path.is_file() {
                // Worktree / Submodule: .git is a text file containing "gitdir: path"
                if let Ok(content) = fs::read_to_string(&git_path) {
                    if let Some(dir_part) = content.lines().find(|l| l.starts_with("gitdir:")) {
                        let rel_path = dir_part["gitdir:".len()..].trim();
                        let target = curr.join(rel_path);
                        target.join("HEAD")
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                git_path.join("HEAD")
            };

            if let Ok(head_str) = fs::read_to_string(&head_path) {
                let trimmed = head_str.trim();
                if let Some(branch_ref) = trimmed.strip_prefix("ref: refs/heads/") {
                    return (std::borrow::Cow::Owned(branch_ref.to_string()), false);
                } else if trimmed.len() >= 7 {
                    return (std::borrow::Cow::Owned(trimmed[..7].to_string()), false);
                }
            }
            break;
        }

        if !curr.pop() {
            break;
        }
    }

    (std::borrow::Cow::Borrowed(""), false)
}

pub fn get_sys_info() -> SysInfo {
    let mut mem_pct = None;
    let mut load_1m = None;

    if cfg!(target_os = "linux") {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            let mut total = 0u64;
            let mut avail = 0u64;
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        total = val.parse().unwrap_or(0);
                    }
                } else if line.starts_with("MemAvailable:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        avail = val.parse().unwrap_or(0);
                    }
                }
            }
            if total > 0 {
                mem_pct = Some((((total - avail) * 100) / total) as u32);
            }
        }

        if let Ok(loadavg) = fs::read_to_string("/proc/loadavg") {
            if let Some(load) = loadavg.split_whitespace().next() {
                load_1m = Some(load.to_string());
            }
        }
    }

    SysInfo { mem_pct, load_1m }
}

pub fn get_host_info() -> Option<String> {
    let hostname = env::var("HOSTNAME")
        .or_else(|_| env::var("COMPUTERNAME"))
        .ok()?;

    if hostname.is_empty() {
        return None;
    }

    Some(hostname)
}

pub fn get_power_info() -> Option<PowerInfo> {
    let env_dir = env::var("STATUSLINE_POWER_SUPPLY_DIR")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let is_linux = cfg!(target_os = "linux");

    if is_linux || env_dir.is_some() {
        let default_path = PathBuf::from("/sys/class/power_supply");
        let power_dir = env_dir.map(PathBuf::from).unwrap_or(default_path);

        if power_dir.is_dir() {
            let mut ac_connected = false;
            let mut has_sys_battery = false;
            let mut has_ac_adapter = false;
            let mut sys_bat_cap = None;

            if let Ok(entries) = fs::read_dir(&power_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let dev_name = entry.file_name().to_string_lossy().to_string();

                    // Skip peripheral devices (mice, keyboards, controllers)
                    if let Ok(scope) = fs::read_to_string(path.join("scope")) {
                        if scope.trim() == "Device" {
                            continue;
                        }
                    }
                    if dev_name.starts_with("hidpp_")
                        || dev_name.contains("mouse")
                        || dev_name.contains("keyboard")
                    {
                        continue;
                    }

                    let psy_type = fs::read_to_string(path.join("type"))
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();

                    // Check for AC / Mains / USB chargers
                    if psy_type == "Mains"
                        || dev_name.starts_with("AC")
                        || dev_name.starts_with("ACAD")
                        || dev_name.starts_with("ADP")
                        || dev_name.starts_with("Mains")
                    {
                        has_ac_adapter = true;
                        if let Ok(online) = fs::read_to_string(path.join("online")) {
                            if online.trim() == "1" {
                                ac_connected = true;
                            }
                        }
                    } else if psy_type == "USB" {
                        if !dev_name.contains("ucsi-source") {
                            if let Ok(online) = fs::read_to_string(path.join("online")) {
                                if online.trim() == "1" {
                                    ac_connected = true;
                                    has_ac_adapter = true;
                                }
                            }
                        }
                    } else if psy_type == "Battery" || psy_type == "UPS" || dev_name.starts_with("BAT") {
                        has_sys_battery = true;
                        if sys_bat_cap.is_none() {
                            if let Ok(cap_str) = fs::read_to_string(path.join("capacity")) {
                                sys_bat_cap = cap_str.trim().parse::<u32>().ok();
                            }
                        }
                        if let Ok(status) = fs::read_to_string(path.join("status")) {
                            let s = status.trim();
                            if s == "Charging" || s == "Full" || s == "Not charging" {
                                ac_connected = true;
                            }
                        }
                    }
                }
            }

            // Desktop / server without system battery and without laptop AC adapter
            if !has_sys_battery && !has_ac_adapter {
                ac_connected = true;
            }

            if ac_connected {
                return Some(PowerInfo {
                    is_ac: true,
                    battery_pct: None,
                });
            } else if has_sys_battery {
                return Some(PowerInfo {
                    is_ac: false,
                    battery_pct: sys_bat_cap,
                });
            }
        }
    }
    None
}
