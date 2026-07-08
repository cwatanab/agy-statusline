use statusline::sys;

#[test]
fn hostname_returns_string() {
    let name = sys::hostname();
    // Should return something (even empty is fine - just shouldn't panic)
    let _ = name.len();
}

#[test]
fn tailscale_ip_returns_string() {
    let ip = sys::tailscale_ip();
    let _ = ip.len();
}

#[test]
fn power_status_returns_tuple() {
    let (on_bat, cap) = sys::power_status();
    // Shouldn't panic regardless of platform
    if let Some(c) = cap {
        assert!(c <= 100);
    }
    let _ = on_bat;
}

#[test]
fn git_info_empty_dir() {
    let (vcs_type, branch, dirty) = sys::git_info("");
    let _ = (vcs_type, branch);
    assert!(!dirty || dirty); // trivial, just ensuring no panic
}
