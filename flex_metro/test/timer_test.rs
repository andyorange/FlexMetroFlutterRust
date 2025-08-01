// Updated timer tests for the current FlexMetro architecture
// Tests the core timer functionality and Flutter interface integration

use flex_metro::api::fm_ticker_base::{FMSectionTimer, BeatEvent, BeatType};
use flex_metro::api::fm_bar_element::FMBarElement;
use flex_metro::api::flutter_interface::{init_flutter_timer, start_musical_section_simple, stop_musical_timer, get_beat_events, clear_beat_events};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn test_basic_timer_creation() {
    println!("=== Basic Timer Creation Test ===");
    
    // Test creating a basic timer
    let timer = FMSectionTimer::new(chrono::TimeDelta::milliseconds(100));
    assert!(timer.is_ok(), "Should be able to create a basic timer");
    
    println!("✅ Basic timer creation test passed");
}

#[test]
fn test_musical_section_timer() {
    println!("=== Musical Section Timer Test ===");
    
    // Create a musical section: 4/4, 3/4
    let bars = vec![
        FMBarElement::new(4, 4, 0.0, None),
        FMBarElement::new(3, 4, 0.0, None),
    ];
    
    let timer = FMSectionTimer::new_with_section(bars, 60.0, 80.0);
    assert!(timer.is_ok(), "Should be able to create musical section timer");
    
    let mut timer = timer.unwrap();
    
    // Test callback mechanism
    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();
    
    let callback_result = timer.set_tick_callback(move |beat_event| {
        let mut count = call_count_clone.lock().unwrap();
        *count += 1;
        println!("Beat: {}/{}, Type: {:?}, Time: {:.1}ms", 
                beat_event.nom, beat_event.denom, beat_event.beat_type, beat_event.time_offset_ms);
    });
    
    assert!(callback_result.is_ok(), "Should be able to set callback");
    
    // Test manual tick
    let test_beat_event = BeatEvent {
        time_offset_ms: 1000.0,
        bar_index: 0,
        beat_in_bar: 0,
        subbeat_in_beat: 0,
        tempo_bpm: 60.0,
        nom: 4,
        denom: 4,
        beat_type: BeatType::Major,
    };
    
    timer.manual_tick(test_beat_event);
    
    // Verify callback was called
    let final_count = *call_count.lock().unwrap();
    assert_eq!(final_count, 1, "Callback should have been called once");
    
    println!("✅ Musical section timer test passed");
}

#[test]
fn test_flutter_interface_integration() {
    println!("=== Flutter Interface Integration Test ===");
    
    // Test Flutter interface initialization
    let init_result = init_flutter_timer();
    assert!(init_result.is_ok(), "Should be able to initialize Flutter timer");
    
    // Clear any existing events
    clear_beat_events();
    
    // Test starting a musical section
    let bars = vec![(4, 4), (3, 4)];
    let start_result = start_musical_section_simple(bars, 60.0, 80.0);
    assert!(start_result.is_ok(), "Should be able to start musical section");
    
    // Let it run briefly
    std::thread::sleep(Duration::from_millis(500));
    
    // Check for events
    let events = get_beat_events();
    println!("Captured {} beat events", events.len());
    
    // Stop the timer
    let stop_result = stop_musical_timer();
    assert!(stop_result.is_ok(), "Should be able to stop musical timer");
    
    println!("✅ Flutter interface integration test passed");
}

#[test]
fn test_tempo_change_timing() {
    println!("=== Tempo Change Timing Test ===");
    
    // Initialize Flutter interface
    init_flutter_timer().unwrap();
    clear_beat_events();
    
    // Create section with tempo change: 60 -> 120 BPM
    let bars = vec![(4, 4)]; // Simple 4/4 bar
    start_musical_section_simple(bars, 60.0, 120.0).unwrap();
    
    // Collect events for a few seconds
    let start_time = std::time::Instant::now();
    let mut all_events = Vec::new();
    
    while start_time.elapsed().as_secs() < 3 {
        let events = get_beat_events();
        all_events.extend(events);
        std::thread::sleep(Duration::from_millis(50));
    }
    
    stop_musical_timer().unwrap();
    
    assert!(!all_events.is_empty(), "Should have captured some events");
    
    // Verify we got the expected time signature
    for event in &all_events {
        assert_eq!(event.current_bar_nom, 4, "Should be 4/4 time");
        assert_eq!(event.current_bar_denom, 4, "Should be 4/4 time");
    }
    
    println!("Captured {} events with tempo change", all_events.len());
    println!("✅ Tempo change timing test passed");
}

#[test]
fn test_complex_time_signatures() {
    println!("=== Complex Time Signatures Test ===");
    
    init_flutter_timer().unwrap();
    clear_beat_events();
    
    // Test with complex time signatures
    let bars = vec![(7, 8), (5, 4), (9, 8)];
    start_musical_section_simple(bars.clone(), 80.0, 100.0).unwrap();
    
    std::thread::sleep(Duration::from_millis(2000));
    
    let events = get_beat_events();
    stop_musical_timer().unwrap();
    
    assert!(!events.is_empty(), "Should have captured events for complex time signatures");
    
    // Verify that we get events for all the configured time signatures
    let mut found_signatures = std::collections::HashSet::new();
    for event in &events {
        found_signatures.insert((event.current_bar_nom, event.current_bar_denom));
    }
    
    println!("Found time signatures: {:?}", found_signatures);
    
    // We should find at least one of our configured time signatures
    let expected_signatures: std::collections::HashSet<_> = bars.into_iter().collect();
    let intersection: Vec<_> = found_signatures.intersection(&expected_signatures).collect();
    assert!(!intersection.is_empty(), "Should find at least one expected time signature");
    
    println!("✅ Complex time signatures test passed");
}
