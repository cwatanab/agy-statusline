use statusline::sys;

#[test]
fn git_info_empty_dir() {
    let (branch, dirty) = sys::git_info("", "", false);
    let _ = branch;
    assert!(!dirty || dirty); // trivial, just ensuring no panic
}
