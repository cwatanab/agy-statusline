use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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
    if cfg!(target_os = "linux") {
        let sys_ps = Path::new("/sys/class/power_supply");
        if sys_ps.exists() {
            let mut online = true;
            let mut cap = None;

            if let Ok(entries) = fs::read_dir(sys_ps) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let online_file = path.join("online");
                    if online_file.exists() {
                        if let Ok(val) = fs::read_to_string(&online_file) {
                            if val.trim() == "0" {
                                online = false;
                            }
                        }
                    }
                    let cap_file = path.join("capacity");
                    if cap_file.exists() {
                        if let Ok(val) = fs::read_to_string(&cap_file) {
                            cap = val.trim().parse::<u32>().ok();
                        }
                    }
                }
            }
            return Some(PowerInfo {
                is_ac: online,
                battery_pct: cap,
            });
        }
    }
    None
}
