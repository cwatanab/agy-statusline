use std::process::Command;

pub fn git_info(working_dir: &str) -> (String, bool) {
    let dir = if working_dir.is_empty() { "." } else { working_dir };

    // Windows環境等で cmd.exe などのシェルを挟まないよう git.exe を直接指定
    let git_bin = if cfg!(windows) { "git.exe" } else { "git" };

    let mut git_branch_cmd = Command::new(git_bin);
    git_branch_cmd.env("CLINK_NOINJECT", "1");
    git_branch_cmd.args(["-C", dir, "rev-parse", "--abbrev-ref", "HEAD"]);

    let branch = git_branch_cmd
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if branch.is_empty() {
        return (String::new(), false);
    }

    let mut git_dirty_cmd = Command::new(git_bin);
    git_dirty_cmd.env("CLINK_NOINJECT", "1");
    git_dirty_cmd.args(["-C", dir, "status", "--porcelain"]);

    let dirty = git_dirty_cmd
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    (branch, dirty)
}

