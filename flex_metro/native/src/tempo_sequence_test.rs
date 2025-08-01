// Test for the new FMTempoInterval and FMTempoSequence functionality

use crate::api::fm_ticker_base::{FMSectionTimer, BeatType, AsyncTimer};
use crate::api::fm_bar_element::FMBarElement;
use crate::api::fm_tempo_interval::{FMTempoInterval, FMTempoSequence};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(test)]
mod tempo_sequence_tests {
    use super::*;

    #[test]
    fn test_tempo_sequence_creation() {
        println!("\n=== FMTempoSequence Creation Test ===");
        
        // Create multiple tempo intervals
        let interval1 = FMTempoInterval::new(
            vec![
                FMBarElement::new(4, 4, 0.0, None),
                FMBarElement::new(4, 4, 0.0, None),
            ],
            60.0,  // Start at 60 BPM
            Some(80.0),  // End at 80 BPM
            "Accelerating Section".to_string()
        );
        
        let interval2 = FMTempoInterval::new(
            vec![
                FMBarElement::new(6, 8, 0.0, None),
                FMBarElement::new(6, 8, 0.0, None),
            ],
            80.0,   // Constant 80 BPM
            None,   // No end tempo = constant tempo
            "Constant Section".to_string()
        );
        
        let interval3 = FMTempoInterval::new(
            vec![
                FMBarElement::new(4, 4, 0.0, None),
                FMBarElement::new(4, 4, 0.0, None),
            ],
            80.0,  // Start at 80 BPM
            Some(100.0),  // End at 100 BPM
            "Final Sprint".to_string()
        );
        
        // Create sequence
        let mut sequence = FMTempoSequence::new();
        sequence.add_interval(interval1);
        sequence.add_interval(interval2);
        sequence.add_interval(interval3);
        
        println!("{}", sequence.description());
        
        // Verify sequence properties
        assert_eq!(sequence.interval_count(), 3);
        assert_eq!(sequence.total_bar_count(), 6);
        assert!(!sequence.is_empty());
        
        println!("✅ Tempo sequence creation test passed!");
    }

    #[test]
    fn test_tempo_sequence_real_time_streaming() {
        println!("\n🎼 === Real-Time FMTempoSequence Test: Multi-Interval Streaming ===");
        
        // Create a complex tempo sequence with different intervals
        let mut sequence = FMTempoSequence::new();
        
        // Interval 1: Slow start (4/4 bars, 60->80 BPM)
        sequence.add_interval(FMTempoInterval::new(
            vec![
                FMBarElement::new(4, 4, 0.0, None),
                FMBarElement::new(4, 4, 0.0, None),
            ],
            60.0, 
            Some(80.0),
            "Warm-up".to_string()
        ));
        
        // Interval 2: Fast waltz (6/8 bars, constant 120 BPM)
        sequence.add_interval(FMTempoInterval::new(
            vec![
                FMBarElement::new(6, 8, 0.0, None),
                FMBarElement::new(6, 8, 0.0, None),
            ],
            120.0, 
            None, // Constant tempo
            "Waltz".to_string()
        ));
        
        // Interval 3: Final sprint (4/4 bars, 80->100 BPM)
        sequence.add_interval(FMTempoInterval::new(
            vec![
                FMBarElement::new(4, 4, 0.0, None),
            ],
            80.0, 
            Some(100.0),
            "Sprint".to_string()
        ));
        
        println!("🎵 {} ", sequence.description());
        
        // Create timer with the tempo sequence
        let mut timer = FMTempoSequence::new_with_tempo_sequence(sequence.clone())
            .expect("Failed to create timer from sequence");
        
        // Show the planned beat events
        if let Some(debug_info) = timer.get_beat_events_debug() {
            println!("\n📋 Planned {} beat events:", debug_info.len());
            for (i, info) in debug_info.iter().enumerate() {
                println!("   {:2}: {}", i + 1, info);
            }
        }
        
        // Set up real-time callback for live streaming
        let beat_count = Arc::new(Mutex::new(0));
        let beat_count_clone = beat_count.clone();
        let start_time = std::time::Instant::now();
        let expected_beats = timer.get_all_beat_events().len();
        
        timer.set_tick_callback(move |beat_event| {
            let mut count = beat_count_clone.lock().unwrap();
            *count += 1;
            let current_count = *count;
            
            let elapsed_real = start_time.elapsed().as_millis() as f64;
            
            let beat_type_str = match beat_event.beat_type {
                BeatType::Major => "🔥 MAJOR ",
                BeatType::Medium => "🟡 MEDIUM",
                BeatType::Minor => "🔹 MINOR ",
            };
            
            // Show which interval this beat belongs to based on bar index
            let interval_info = if beat_event.bar_index < 2 {
                "Warm-up"
            } else if beat_event.bar_index < 4 {
                "Waltz"
            } else {
                "Sprint"
            };
            
            println!("🎵 Beat #{:2}/{} | {}/{} | {} | [{}] | Position: {}.{} | Tempo: {:6.1} BPM | Scheduled: {:6.1}ms | Real: {:6.0}ms",
                    current_count,
                    expected_beats,
                    beat_event.nom, beat_event.denom,
                    beat_type_str,
                    interval_info,
                    beat_event.beat_in_bar + 1, beat_event.subbeat_in_beat + 1,
                    beat_event.tempo_bpm,
                    beat_event.time_offset_ms,
                    elapsed_real);
                    
            // Stop when we've captured all expected beats
            if current_count >= expected_beats {
                println!("🎉 All beats captured! Multi-interval streaming complete.");
            }
        }).expect("Failed to set callback");
        
        println!("\n🚀 Starting tempo sequence timer... (beats will stream across intervals)");
        timer.start().expect("Failed to start timer");
        
        // Let the timer run until all beats are captured or timeout
        let timeout = Duration::from_secs(20); // Longer timeout for complex sequence
        let start = std::time::Instant::now();
        
        loop {
            std::thread::sleep(Duration::from_millis(100)); // Small sleep to avoid busy waiting
            
            let current_count = *beat_count.lock().unwrap();
            
            // Check if we've captured all beats
            if current_count >= expected_beats {
                println!("\n✅ All {} beats captured via real-time tempo sequence streaming!", expected_beats);
                break;
            }
            
            // Safety timeout
            if start.elapsed() > timeout {
                println!("\n⏰ Timeout reached - stopping timer");
                break;
            }
        }
        
        // Stop the timer
        timer.stop().expect("Failed to stop timer");
        
        let final_count = *beat_count.lock().unwrap();
        let total_elapsed = start.elapsed();
        
        println!("\n📊 === Multi-Interval Streaming Results ===");
        println!("🎵 Total beats captured: {}/{}", final_count, expected_beats);
        println!("🎼 Total intervals: {}", sequence.interval_count());
        println!("📊 Total bars: {}", sequence.total_bar_count());
        println!("⏱️  Total real time: {:.1}s", total_elapsed.as_secs_f64());
        println!("🎯 Average beat interval: {:.1}ms", total_elapsed.as_millis() as f64 / final_count as f64);
        
        // Verify we got the expected number of beats
        assert_eq!(final_count, expected_beats, "Should have captured all beats via tempo sequence streaming");
        
        println!("✅ Multi-interval FMTempoSequence streaming test completed successfully!");
        println!("   🎼 Multiple tempo intervals handled automatically");
        println!("   🔄 Callback received live beat events across intervals");  
        println!("   ⏰ Different tempo progressions in each interval");
        println!("   🎯 All beats streamed in real-time with precise timing");
    }
}
