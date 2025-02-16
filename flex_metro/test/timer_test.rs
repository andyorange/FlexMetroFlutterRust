use rust::api::FMBase::{parse_metric_ticks, FMMeasureTimer, Sections};
use std::time::Duration;

#[test]
fn test_parse_metric_ticks_integration() {
    let json_data = r#"
        {
            "name": "cpu_usage",
            "value": 0.85,
            "timestamp": 1625247600
        }
    "#;

    let result = parse_metric_ticks(json_data);
    assert!(result.is_ok());

    let metric = result.unwrap();
    assert_eq!(metric.name, "cpu_usage");
    assert_eq!(metric.value, 0.85);
    assert_eq!(metric.timestamp, 1625247600);
}

#[test]
fn test_fm_measure_timer_integration() {
    let sections = Sections { nom: 4 };
    let timers = vec![Duration::from_secs(1); 4];
    let mut timer = FMMeasureTimer::new(sections, 2, timers);

    timer.start();

    assert_eq!(timer.cnt_beat, 1);
    assert_eq!(timer.beat_idx, 1);
    assert_eq!(timer.cnt_measure, 0);
    assert!(!timer.stop);

    timer.cancel();
    assert!(timer.stop);
}
