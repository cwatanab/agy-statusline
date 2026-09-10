use std::env;
use std::fs;
use statusline::sys;

#[test]
fn git_info_empty_dir() {
    let (branch, dirty) = sys::git_info("", "", false);
    let _ = branch;
    assert!(!dirty || dirty); // trivial, just ensuring no panic
}

#[test]
fn test_power_supply_detection() {
    let base = env::temp_dir().join(format!("agy_test_power_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);

    // 7a: Laptop on AC charging
    let s1 = base.join("s1");
    fs::create_dir_all(s1.join("AC")).unwrap();
    fs::create_dir_all(s1.join("BAT0")).unwrap();
    fs::write(s1.join("AC/type"), "Mains\n").unwrap();
    fs::write(s1.join("AC/online"), "1\n").unwrap();
    fs::write(s1.join("BAT0/type"), "Battery\n").unwrap();
    fs::write(s1.join("BAT0/status"), "Charging\n").unwrap();
    fs::write(s1.join("BAT0/capacity"), "45\n").unwrap();

    env::set_var("STATUSLINE_POWER_SUPPLY_DIR", &s1);
    let p1 = sys::get_power_info().expect("s1 power info should be detected");
    assert!(p1.is_ac, "s1 should be on AC");

    // 7b: Laptop on AC threshold (Not charging)
    let s2 = base.join("s2");
    fs::create_dir_all(s2.join("AC")).unwrap();
    fs::create_dir_all(s2.join("BAT0")).unwrap();
    fs::write(s2.join("AC/type"), "Mains\n").unwrap();
    fs::write(s2.join("AC/online"), "1\n").unwrap();
    fs::write(s2.join("BAT0/type"), "Battery\n").unwrap();
    fs::write(s2.join("BAT0/status"), "Not charging\n").unwrap();
    fs::write(s2.join("BAT0/capacity"), "80\n").unwrap();

    env::set_var("STATUSLINE_POWER_SUPPLY_DIR", &s2);
    let p2 = sys::get_power_info().expect("s2 power info should be detected");
    assert!(p2.is_ac, "s2 should be on AC");

    // 7c: Laptop on battery discharging with capacity
    let s3 = base.join("s3");
    fs::create_dir_all(s3.join("AC")).unwrap();
    fs::create_dir_all(s3.join("BAT0")).unwrap();
    fs::write(s3.join("AC/type"), "Mains\n").unwrap();
    fs::write(s3.join("AC/online"), "0\n").unwrap();
    fs::write(s3.join("BAT0/type"), "Battery\n").unwrap();
    fs::write(s3.join("BAT0/status"), "Discharging\n").unwrap();
    fs::write(s3.join("BAT0/capacity"), "65\n").unwrap();

    env::set_var("STATUSLINE_POWER_SUPPLY_DIR", &s3);
    let p3 = sys::get_power_info().expect("s3 power info should be detected");
    assert!(!p3.is_ac, "s3 should be on battery");
    assert_eq!(p3.battery_pct, Some(65));

    // 7d: Laptop on AC with peripheral mouse (hidpp_battery_0 scope: Device online: 0)
    let s4 = base.join("s4");
    fs::create_dir_all(s4.join("AC")).unwrap();
    fs::create_dir_all(s4.join("BAT0")).unwrap();
    fs::create_dir_all(s4.join("hidpp_battery_0")).unwrap();
    fs::write(s4.join("AC/type"), "Mains\n").unwrap();
    fs::write(s4.join("AC/online"), "1\n").unwrap();
    fs::write(s4.join("BAT0/type"), "Battery\n").unwrap();
    fs::write(s4.join("BAT0/status"), "Charging\n").unwrap();
    fs::write(s4.join("BAT0/capacity"), "90\n").unwrap();
    fs::write(s4.join("hidpp_battery_0/type"), "Battery\n").unwrap();
    fs::write(s4.join("hidpp_battery_0/scope"), "Device\n").unwrap();
    fs::write(s4.join("hidpp_battery_0/online"), "0\n").unwrap();

    env::set_var("STATUSLINE_POWER_SUPPLY_DIR", &s4);
    let p4 = sys::get_power_info().expect("s4 power info should be detected");
    assert!(p4.is_ac, "s4 should be on AC despite peripheral mouse online: 0");

    // 7e: Desktop workstation with only peripheral devices (scope: Device)
    let s5 = base.join("s5");
    fs::create_dir_all(s5.join("hidpp_battery_0")).unwrap();
    fs::create_dir_all(s5.join("ucsi-source-psy")).unwrap();
    fs::write(s5.join("hidpp_battery_0/type"), "Battery\n").unwrap();
    fs::write(s5.join("hidpp_battery_0/scope"), "Device\n").unwrap();
    fs::write(s5.join("hidpp_battery_0/online"), "0\n").unwrap();
    fs::write(s5.join("ucsi-source-psy/type"), "USB\n").unwrap();
    fs::write(s5.join("ucsi-source-psy/scope"), "Device\n").unwrap();
    fs::write(s5.join("ucsi-source-psy/online"), "0\n").unwrap();

    env::set_var("STATUSLINE_POWER_SUPPLY_DIR", &s5);
    let p5 = sys::get_power_info().expect("s5 power info should be detected");
    assert!(p5.is_ac, "s5 desktop workstation should default to AC");

    // Empty string env var should not scan current directory
    env::set_var("STATUSLINE_POWER_SUPPLY_DIR", "");
    let _ = sys::get_power_info();

    // Cleanup
    env::remove_var("STATUSLINE_POWER_SUPPLY_DIR");
    let _ = fs::remove_dir_all(&base);
}
