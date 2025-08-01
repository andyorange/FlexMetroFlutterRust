pub mod api;

// Test modules
#[cfg(test)]
pub mod realtime_test;

#[cfg(test)]
pub mod tempo_sequence_test;

#[cfg(test)]
pub mod fm_ticker_base_test;

// Re-export types for FRB
pub use chrono::TimeDelta;
pub use std::error::Error;

// Logging functions for internal use
pub fn log_info(msg: &str) {
    println!("[INFO] {}", msg);
}

pub fn log_debug(msg: &str) {
    println!("[DEBUG] {}", msg);
}

pub fn log_warn(msg: &str) {
    println!("[WARN] {}", msg);
}

pub fn log_error(msg: &str) {
    println!("[ERROR] {}", msg);
}

#[cfg(test)]
mod integration_tests {
    use crate::api::fm_ticker_base::FMSectionTimer;
    use crate::api::fm_bar_element::FMBarElement;
    
    #[test]
    fn test_basic_fm_ticker_functionality() {
        println!("\n=== Basic FMTicker Test: 4/4, 6/8, 4/4 | Tempo 60->90 BPM ===");
        
        // Create the exact section: 4/4, 6/8, 4/4
        let bars = vec![
            FMBarElement::new(4, 4, 0.0, None), // First 4/4 bar
            FMBarElement::new(6, 8, 0.0, None), // 6/8 bar
            FMBarElement::new(4, 4, 0.0, None), // Second 4/4 bar
        ];
        
        println!("Section bars: 4/4, 6/8, 4/4");
        println!("Tempo change: 60.0 BPM → 90.0 BPM");
        
        // Create timer with the musical section
        let timer = match FMSectionTimer::new_with_section(bars.clone(), 60.0, 90.0) {
            Ok(timer) => timer,
            Err(e) => {
                println!("❌ Failed to create timer: {}", e);
                return;
            }
        };
        
        // Get debug information about the beat events
        if let Some(debug_info) = timer.get_beat_events_debug() {
            println!("\nGenerated {} beat events:", debug_info.len());
            for (i, info) in debug_info.iter().enumerate() {
                println!("  Beat #{:2}: {}", i + 1, info);
            }
        }
        
        // Process all beat events directly from the event queue
        let beat_events = timer.get_all_beat_events();
        
        println!("\n🚀 Processing all {} beat events from queue...", beat_events.len());
        
        // Process each beat event
        for (i, beat_event) in beat_events.iter().enumerate() {
            let beat_type_str = match beat_event.beat_type {
                crate::api::fm_ticker_base::BeatType::Major => "MAJOR",
                crate::api::fm_ticker_base::BeatType::Medium => "MEDIUM", 
                crate::api::fm_ticker_base::BeatType::Minor => "MINOR",
            };
            
            println!("🎵 Beat #{:2} | {}/{} | {:6} | Tempo: {:.1} BPM | Time: {:6.1}ms",
                    i + 1,
                    beat_event.nom, beat_event.denom,
                    beat_type_str,
                    beat_event.tempo_bpm,
                    beat_event.time_offset_ms);
        }
        
        println!("⏹️  Event queue processing completed");
        
        // Analyze the results
        println!("\n=== Analysis ===");
        println!("Total beats in queue: {}", beat_events.len());
        
        // Verify we got some events
        assert!(!beat_events.is_empty(), "Should have generated some events");
        
        // Check tempo progression
        if beat_events.len() >= 2 {
            if let (Some(first_event), Some(last_event)) = (beat_events.first(), beat_events.last()) {
                let first_tempo = first_event.tempo_bpm;
                let last_tempo = last_event.tempo_bpm;
                
                println!("Tempo progression: {:.1} BPM -> {:.1} BPM", first_tempo, last_tempo);
                assert!(last_tempo > first_tempo, "Tempo should increase");
            } else {
                println!("⚠️ Could not get first/last events for tempo check");
            }
        }
        
        println!("✅ FMTickerBase test completed successfully!");
    }

    #[test]
    fn test_pause_resume_functionality() {
        println!("\n=== Pause/Resume Test: FMTempoInterval Sequence ===");
        
        use crate::api::fm_tempo_interval::{FMTempoInterval, FMTempoSequence};
        use crate::api::fm_ticker_base::AsyncTimer;
        use std::sync::{Arc, Mutex};
        use std::time::Duration;
        
        // Create a tempo sequence with multiple intervals
        let interval1 = FMTempoInterval::new(
            vec![
                FMBarElement::new(4, 4, 0.0, None),
                FMBarElement::new(4, 4, 0.0, None),
            ],
            60.0,
            Some(80.0),
            "Warm-up".to_string()
        );
        
        let interval2 = FMTempoInterval::new(
            vec![
                FMBarElement::new(6, 8, 0.0, None),
                FMBarElement::new(6, 8, 0.0, None),
            ],
            120.0,
            Some(120.0), // Constant tempo
            "Fast".to_string()
        );
        
        let mut sequence = FMTempoSequence::new();
        sequence.add_interval(interval1);
        sequence.add_interval(interval2);
        
        // Create timer with the sequence
        let mut timer = match FMTempoSequence::new_with_tempo_sequence(sequence) {
            Ok(timer) => timer,
            Err(e) => {
                println!("❌ Failed to create timer: {}", e);
                return;
            }
        };
        
        // Set up callback to track events
        let events_captured = Arc::new(Mutex::new(Vec::<String>::new()));
        let events_clone = events_captured.clone();
        
        if let Err(e) = timer.set_tick_callback(move |beat_event| {
            if let Ok(mut events) = events_clone.lock() {
                let event_num = events.len() + 1;
                events.push(format!("Beat {} at {:.1}ms - Tempo: {:.1} BPM", 
                    event_num, beat_event.time_offset_ms, beat_event.tempo_bpm));
            }
        }) {
            println!("❌ Failed to set callback: {}", e);
            return;
        }
        
        println!("🚀 Starting timer...");
        if let Err(e) = timer.start() {
            println!("❌ Failed to start timer: {}", e);
            return;
        }
        
        // Let it run for 2 seconds
        std::thread::sleep(Duration::from_secs(2));
        println!("⏸️  Pausing timer...");
        
        if let Err(e) = timer.pause() {
            println!("❌ Failed to pause timer: {}", e);
            return;
        }
        
        // Check events captured before pause
        let events_before_pause = if let Ok(events) = events_captured.lock() {
            events.len()
        } else { 0 };
        
        println!("📊 Events before pause: {}", events_before_pause);
        
        // Wait 2 seconds while paused (should not capture new events)
        std::thread::sleep(Duration::from_secs(2));
        
        let events_during_pause = if let Ok(events) = events_captured.lock() {
            events.len()
        } else { 0 };
        
        println!("📊 Events during pause: {} (should be same as before)", events_during_pause);
        
        // Resume
        println!("▶️  Resuming timer...");
        if let Err(e) = timer.resume() {
            println!("❌ Failed to resume timer: {}", e);
            return;
        }
        
        // Let it run for 2 more seconds
        std::thread::sleep(Duration::from_secs(2));
        
        // Stop the timer
        if let Err(e) = timer.stop() {
            println!("❌ Failed to stop timer: {}", e);
        }
        
        // Check final event count
        let final_events = if let Ok(events) = events_captured.lock() {
            events.len()
        } else { 0 };
        
        println!("📊 Final events: {}", final_events);
        
        // Verify pause worked (no events during pause)
        assert_eq!(events_before_pause, events_during_pause, 
                  "No events should be captured during pause");
        assert!(final_events > events_during_pause, 
                "More events should be captured after resume");
                
        // Print some captured events for verification
        if let Ok(events) = events_captured.lock() {
            println!("\n📋 Sample captured events:");
            for (i, event) in events.iter().take(5).enumerate() {
                println!("  {}: {}", i + 1, event);
            }
            if events.len() > 5 {
                println!("  ... and {} more events", events.len() - 5);
            }
        }
        
        println!("✅ Pause/Resume test completed successfully!");
        println!("   - Timer paused correctly (no events during pause)");
        println!("   - Timer resumed correctly (events continued after resume)");
        println!("   - FMTempoInterval sequence handled pause/resume seamlessly");
    }
}
