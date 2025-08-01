// Flutter timing tests - included from lib.rs
// These tests verify the timing accuracy and precision of the Flutter interface

use crate::api::flutter_interface::*;
use std::time::Instant;

#[test]
fn test_timing_accuracy() {
    println!("=== Timing Accuracy Test ===");
    
    // Initialize and create a simple 4/4 section at 60 BPM
    init_flutter_timer().unwrap();
    let bars = vec![(4, 4)];
    start_musical_section_simple(bars, 60.0, 60.0).unwrap();
    
    // At 60 BPM, quarter notes should be 1000ms apart
    let expected_interval_ms = 1000.0;
    let tolerance_ms = 50.0; // 5% tolerance
    
    let start_time = Instant::now();
    let mut timestamps = Vec::new();
    
    // Collect events for 5 seconds
    while start_time.elapsed().as_secs() < 5 {
        let events = get_beat_events();
        
        for event in events {
            timestamps.push(event.current_time_ms);
        }
        
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    stop_musical_timer().unwrap();
    
    // Analyze timing intervals
    assert!(timestamps.len() >= 2, "Should have captured at least 2 beats");
    
    let mut intervals = Vec::new();
    for i in 1..timestamps.len() {
        let interval = timestamps[i] as f64 - timestamps[i-1] as f64;
        intervals.push(interval);
    }
    
    println!("Captured {} beats, {} intervals", timestamps.len(), intervals.len());
    println!("Expected interval: {:.1}ms ± {:.1}ms", expected_interval_ms, tolerance_ms);
    
    for (i, interval) in intervals.iter().enumerate() {
        println!("Interval {}: {:.1}ms", i+1, interval);
        let error = (interval - expected_interval_ms).abs();
        assert!(error <= tolerance_ms, 
               "Interval {} ({:.1}ms) exceeds tolerance (error: {:.1}ms)", 
               i+1, interval, error);
    }
    
    println!("✅ All timing intervals within tolerance");
}

#[test]
fn test_tempo_progression() {
    println!("=== Tempo Progression Test ===");
    
    init_flutter_timer().unwrap();
    
    // Create section with tempo change: 60 -> 120 BPM
    let bars = vec![(4, 4), (4, 4)]; // 8 beats total
    start_musical_section_simple(bars, 60.0, 120.0).unwrap();
    
    let start_time = Instant::now();
    let mut beat_times = Vec::new();
    
    // Collect all beats
    while start_time.elapsed().as_secs() < 10 {
        let events = get_beat_events();
        
        for event in events {
            beat_times.push(event.current_time_ms as f64);
        }
        
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    stop_musical_timer().unwrap();
    
    assert!(beat_times.len() >= 4, "Should have captured at least 4 beats");
    println!("Captured {} beats", beat_times.len());
    
    // Calculate intervals and verify tempo progression
    let mut intervals = Vec::new();
    for i in 1..beat_times.len() {
        let interval = beat_times[i] - beat_times[i-1];
        intervals.push(interval);
    }
    
    // Verify that intervals get shorter (tempo increases)
    let first_interval = intervals[0];
    let last_interval = intervals[intervals.len()-1];
    
    println!("First interval: {:.1}ms", first_interval);
    println!("Last interval: {:.1}ms", last_interval);
    
    assert!(last_interval < first_interval, 
           "Last interval should be shorter than first (tempo should increase)");
    
    // First interval should be close to 60 BPM (1000ms)
    assert!((first_interval - 1000.0).abs() < 100.0, 
           "First interval should be close to 1000ms (60 BPM)");
    
    // Last interval should be close to 120 BPM (500ms)
    assert!((last_interval - 500.0).abs() < 100.0, 
           "Last interval should be close to 500ms (120 BPM)");
    
    println!("✅ Tempo progression verified");
}

#[test]
fn test_event_queue_behavior() {
    println!("=== Event Queue Behavior Test ===");
    
    init_flutter_timer().unwrap();
    
    // Clear any existing events
    clear_beat_events();
    
    // Verify queue is empty
    let events = get_beat_events();
    assert!(events.is_empty(), "Queue should be empty after clear");
    
    // Start a section
    let bars = vec![(2, 4)]; // Simple 2/4 bar
    start_musical_section_simple(bars, 120.0, 120.0).unwrap();
    
    // Wait for some events to accumulate
    std::thread::sleep(std::time::Duration::from_millis(1500));
    
    // Get events (should clear the queue)
    let events1 = get_beat_events();
    assert!(!events1.is_empty(), "Should have captured some events");
    
    // Immediate second call should return empty queue
    let events2 = get_beat_events();
    assert!(events2.is_empty(), "Queue should be empty after reading");
    
    stop_musical_timer().unwrap();
    
    println!("✅ Event queue behavior verified");
    println!("  First read: {} events", events1.len());
    println!("  Second read: {} events", events2.len());
}

#[test]
fn test_multiple_section_changes() {
    println!("=== Multiple Section Changes Test ===");
    
    init_flutter_timer().unwrap();
    
    let test_configs = vec![
        (vec![(3, 4)], 80.0, 80.0),
        (vec![(5, 8)], 100.0, 100.0),
        (vec![(7, 8)], 60.0, 90.0),
        (vec![(4, 4), (6, 8)], 70.0, 110.0),
    ];
    
    for (i, (bars, start_tempo, end_tempo)) in test_configs.iter().enumerate() {
        println!("Test config {}: {:?} @ {:.1}->{:.1} BPM", 
                i+1, bars, start_tempo, end_tempo);
        
        clear_beat_events();
        
        start_musical_section_simple(bars.clone(), *start_tempo, *end_tempo).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(800));
        
        let events = get_beat_events();
        println!("  Captured {} events", events.len());
        
        // Verify events have correct time signature
        for event in &events {
            let found_bar = bars.iter().any(|(nom, denom)| 
                event.current_bar_nom == *nom && event.current_bar_denom == *denom);
            assert!(found_bar, "Event should match one of the configured bars");
        }
        
        stop_musical_timer().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    println!("✅ Multiple section changes test passed");
}
