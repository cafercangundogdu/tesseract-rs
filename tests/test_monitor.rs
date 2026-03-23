use tesseract_rs::TessMonitor;

#[test]
fn test_monitor_create() {
    let monitor = TessMonitor::new();
    drop(monitor);
}

#[test]
fn test_monitor_set_deadline() {
    let monitor = TessMonitor::new();
    let result = monitor.set_deadline(5000);
    assert!(result.is_ok());
}

#[test]
fn test_monitor_get_progress() {
    let monitor = TessMonitor::new();
    let progress = monitor.get_progress().unwrap();
    assert_eq!(progress, 0);
}

#[test]
fn test_monitor_set_deadline_and_check() {
    let monitor = TessMonitor::new();
    monitor.set_deadline(1000).unwrap();
    let progress = monitor.get_progress().unwrap();
    assert!(progress >= 0);
}
