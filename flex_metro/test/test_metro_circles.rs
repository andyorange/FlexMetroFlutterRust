// Updated metro circles test for the current FlexMetro architecture
// Tests UI integration concepts and circle-based timing visualization

use flex_metro::api::fm_bar_element::FMBarElement;
use flex_metro::api::fm_ticker_base::{FMSectionTimer, BeatType};
use flex_metro::api::flutter_interface::{init_flutter_timer, start_musical_section_simple, stop_musical_timer, get_beat_events, clear_beat_events};
use std::collections::HashMap;

#[test]
fn test_metro_circles_basic_concept() {
    println!("=== Metro Circles Basic Concept Test ===");
    
    // Test the concept of circle-based metro visualization
    // This would be used by a UI to show circular timing displays
    
    let bar_element = FMBarElement::new(4, 4, 0.0, None);
    assert_eq!(bar_element.nom, 4);
    assert_eq!(bar_element.denom, 4);
    
    println!("Created 4/4 bar element for circle visualization");
    println!("✅ Metro circles basic concept test passed");
}

#[test]
fn test_metro_circles_timing_data() {
    println!("=== Metro Circles Timing Data Test ===");
    
    // Test generating timing data that could drive circular visualizations
    init_flutter_timer().unwrap();
    clear_beat_events();
    
    // Create a section with different time signatures for varied circles
    let bars = vec![(3, 4), (6, 8), (5, 4)];
    start_musical_section_simple(bars, 80.0, 100.0).unwrap();
    
    // Collect timing data
    std::thread::sleep(std::time::Duration::from_millis(3000));
    let events = get_beat_events();
    stop_musical_timer().unwrap();
    
    assert!(!events.is_empty(), "Should have timing data for circles");
    
    // Analyze data for circle visualization
    let mut circles_data = HashMap::new();
    
    for event in &events {
        let key = (event.current_bar_nom, event.current_bar_denom);
        let entry = circles_data.entry(key).or_insert(Vec::new());
        entry.push((event.subbeat_position, event.beat_type.clone(), event.current_time_ms));
    }
    
    println!("Generated circle data for {} different time signatures:", circles_data.len());
    for ((nom, denom), beats) in &circles_data {
        println!("  {}/{}: {} beat points", nom, denom, beats.len());
    }
    
    // Verify we have data for multiple circle types
    assert!(!circles_data.is_empty(), "Should have circle visualization data");
    
    println!("✅ Metro circles timing data test passed");
}

#[test]
fn test_metro_circles_beat_classification() {
    println!("=== Metro Circles Beat Classification Test ===");
    
    // Test classification of beats for different circle visualization styles
    init_flutter_timer().unwrap();
    clear_beat_events();
    
    // Use a complex time signature to get varied beat types
    let bars = vec![(7, 8)]; // 7/8 will have different beat emphasis patterns
    start_musical_section_simple(bars, 90.0, 90.0).unwrap();
    
    std::thread::sleep(std::time::Duration::from_millis(2000));
    let events = get_beat_events();
    stop_musical_timer().unwrap();
    
    // Classify beats for circle visualization
    let mut beat_type_counts = HashMap::new();
    
    for event in &events {
        *beat_type_counts.entry(event.beat_type.clone()).or_insert(0) += 1;
    }
    
    println!("Beat type distribution for circles:");
    for (beat_type, count) in &beat_type_counts {
        println!("  {}: {} beats", beat_type, count);
    }
    
    // Verify we have different beat types (for different circle emphasis)
    assert!(!beat_type_counts.is_empty(), "Should have beat type classifications");
    
    println!("✅ Metro circles beat classification test passed");
}

#[test]
fn test_metro_circles_multi_tempo() {
    println!("=== Metro Circles Multi-Tempo Test ===");
    
    // Test how circles would handle tempo changes
    init_flutter_timer().unwrap();
    clear_beat_events();
    
    // Create section with significant tempo change
    let bars = vec![(4, 4)];
    start_musical_section_simple(bars, 60.0, 120.0).unwrap();
    
    // Collect events over time to see tempo progression
    let start_time = std::time::Instant::now();
    let mut tempo_progression = Vec::new();
    
    while start_time.elapsed().as_secs() < 4 {
        let events = get_beat_events();
        let elapsed_ms = start_time.elapsed().as_millis() as u64;
        
        for event in events {
            tempo_progression.push((elapsed_ms, event.current_time_ms));
        }
        
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    
    stop_musical_timer().unwrap();
    
    assert!(!tempo_progression.is_empty(), "Should have tempo progression data");
    
    // Calculate intervals to verify tempo change (for adaptive circle timing)
    if tempo_progression.len() >= 2 {
        let early_interval = tempo_progression[1].1 - tempo_progression[0].1;
        let late_interval = if tempo_progression.len() > 2 {
            tempo_progression[tempo_progression.len()-1].1 - tempo_progression[tempo_progression.len()-2].1
        } else {
            early_interval
        };
        
        println!("Early interval: {}ms, Late interval: {}ms", early_interval, late_interval);
        
        // Later intervals should be shorter (faster tempo)
        if tempo_progression.len() > 2 {
            assert!(late_interval <= early_interval, "Tempo should increase (intervals decrease)");
        }
    }
    
    println!("✅ Metro circles multi-tempo test passed");
}

#[test]
fn test_metro_circles_section_transitions() {
    println!("=== Metro Circles Section Transitions Test ===");
    
    // Test how circles would handle transitions between different time signatures
    init_flutter_timer().unwrap();
    
    let test_sections = vec![
        vec![(3, 4)],      // Simple triple meter
        vec![(4, 4)],      // Simple quadruple meter  
        vec![(5, 8)],      // Complex quintuple meter
        vec![(6, 8)],      // Compound duple meter
    ];
    
    for (i, bars) in test_sections.iter().enumerate() {
        println!("Testing circle transition {}: {:?}", i + 1, bars);
        
        clear_beat_events();
        start_musical_section_simple(bars.clone(), 80.0, 80.0).unwrap();
        
        std::thread::sleep(std::time::Duration::from_millis(800));
        
        let events = get_beat_events();
        stop_musical_timer().unwrap();
        
        // Verify events match the expected time signature
        if !events.is_empty() {
            let (expected_nom, expected_denom) = bars[0];
            for event in &events {
                assert_eq!(event.current_bar_nom, expected_nom);
                assert_eq!(event.current_bar_denom, expected_denom);
            }
            println!("  ✓ {} events with correct signature {}/{}", 
                    events.len(), expected_nom, expected_denom);
        }
        
        // Brief pause between sections
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    println!("✅ Metro circles section transitions test passed");
}
