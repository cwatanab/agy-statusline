use statusline::sys;



#[test]
fn git_info_empty_dir() {
    let (vcs_type, branch, dirty) = sys::git_info("");
    let _ = (vcs_type, branch);
    assert!(!dirty || dirty); // trivial, just ensuring no panic
}
