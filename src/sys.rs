use std::process::Command;

pub fn power_status() -> (bool, Option<u8>) {
    #[cfg(target_os = "linux")]
    {
        let ac = std::fs::read_to_string("/sys/class/power_supply/ACAD/online")
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok())
            .unwrap_or(1);
        let battery = std::fs::read_to_string("/sys/class/power_supply/BAT1/capacity")
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok());
        (ac == 0, battery)
    }
    #[cfg(not(target_os = "linux"))]
    {
        (false, None)
    }
}

pub fn git_info(working_dir: &str) -> (String, String, bool) {
    let dir = if working_dir.is_empty() { "." } else { working_dir };

    let branch = Command::new("git")
        .args(["-C", dir, "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if branch.is_empty() {
        return (String::new(), String::new(), false);
    }

    let dirty = Command::new("git")
        .args(["-C", dir, "status", "--porcelain"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    ("git".to_string(), branch, dirty)
}
