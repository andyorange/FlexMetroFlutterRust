// FMTickerBase focused test - 4/4, 6/8, 4/4 bars with tempo 60->90 BPM
// This test focuses purely on the Rust core functionality without Flutter interface

use crate::api::fm_ticker_base::{FMSectionTimer, BeatEvent, BeatType, MusicalTiming, AsyncTimer};
use crate::api::fm_bar_element::FMBarElement;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fm_ticker_base_three_bars_tempo_change() {
        println!("\n=== FMTickerBase Test: 4/4, 6/8, 4/4 | Tempo 60->90 BPM ===");
        
        // Create the exact section you want: 4/4, 6/8, 4/4
        let bars = vec![
            FMBarElement::new(4, 4, 0.0, None), // First 4/4 bar
            FMBarElement::new(6, 8, 0.0, None), // 6/8 bar
            FMBarElement::new(4, 4, 0.0, None), // Second 4/4 bar
        ];
        
        println!("Section bars: 4/4, 6/8, 4/4");
        println!("Tempo change: 60.0 BPM → 90.0 BPM");
        
        // Create timer with the musical section
        let mut timer = FMSectionTimer::new_with_section(bars.clone(), 60.0, 90.0).unwrap();
        
        // Get debug information about the beat events
        if let Some(debug_info) = timer.get_beat_events_debug() {
            println!("\nGenerated {} beat events:", debug_info.len());
            for (i, info) in debug_info.iter().enumerate() {
                println!("  Beat #{:2}: {}", i + 1, info);
            }
        }
        
        // Set up callback to capture events with detailed analysis
        let captured_events = Arc::new(Mutex::new(Vec::<BeatEvent>::new()));
        let captured_events_clone = captured_events.clone();
        
        timer.set_tick_callback(move |beat_event| {
            let mut events = captured_events_clone.lock().unwrap();
            events.push(beat_event.clone());
            
            let beat_type_str = match beat_event.beat_type {
                BeatType::Major => "MAJOR",
                BeatType::Medium => "MEDIUM",
                BeatType::Minor => "MINOR",
            };
            
            println!("🎵 Beat #{:2} | {}/{} | {:6} | Position: {}.{} | Tempo: {:.1} BPM | Time: {:6.1}ms",
                    events.len(),
                    beat_event.nom, beat_event.denom,
                    beat_type_str,
                    beat_event.beat_in_bar + 1, beat_event.subbeat_in_beat + 1,
                    beat_event.tempo_bpm,
                    beat_event.time_offset_ms);
        }).unwrap();
        
        println!("\n🚀 Starting timer...");
        timer.start().unwrap();
        
        // Let it run long enough to complete all beats
        // At 60-90 BPM, this should take about 6-10 seconds
        println!("⏱️  Running for 12 seconds to capture all beats...");
        std::thread::sleep(Duration::from_secs(12));
        
        // Stop the timer
        timer.stop().unwrap();
        println!("⏹️  Timer stopped");
        
        // Analyze the captured events
        let events = captured_events.lock().unwrap();
        println!("\n=== Analysis ===");
        println!("Total beats captured: {}", events.len());
        
        // Group events by time signature
        let mut beats_by_signature = std::collections::HashMap::new();
        for event in events.iter() {
            let key = (event.nom, event.denom);
            let entry = beats_by_signature.entry(key).or_insert(Vec::new());
            entry.push(event);
        }
        
        println!("\nBeats by time signature:");
        for ((nom, denom), signature_events) in &beats_by_signature {
            println!("  {}/{}: {} beats", nom, denom, signature_events.len());
            
            // Show beat types for this signature
            let mut beat_type_counts = std::collections::HashMap::new();
            for event in signature_events {
                *beat_type_counts.entry(event.beat_type).or_insert(0) += 1;
            }
            for (beat_type, count) in beat_type_counts {
                println!("    {:?}: {}", beat_type, count);
            }
        }
        
        // Verify tempo progression
        if events.len() >= 2 {
            let first_tempo = events.first().unwrap().tempo_bpm;
            let last_tempo = events.last().unwrap().tempo_bpm;
            
            println!("\nTempo progression:");
            println!("  First beat: {:.1} BPM", first_tempo);
            println!("  Last beat:  {:.1} BPM", last_tempo);
            println!("  Change:     {:.1} BPM ({:.1}% increase)", 
                    last_tempo - first_tempo, 
                    ((last_tempo - first_tempo) / first_tempo) * 100.0);
            
            // Verify tempo increases
            assert!(last_tempo > first_tempo, "Tempo should increase from start to end");
            assert!((first_tempo - 60.0).abs() < 5.0, "First tempo should be close to 60 BPM");
            assert!((last_tempo - 90.0).abs() < 5.0, "Last tempo should be close to 90 BPM");
        }
        
        // Calculate and verify timing intervals
        if events.len() >= 2 {
            println!("\nTiming intervals:");
            let mut intervals = Vec::new();
            for i in 1..events.len() {
                let interval = events[i].time_offset_ms - events[i-1].time_offset_ms;
                intervals.push(interval);
                if i <= 5 || i > events.len() - 5 {
                    println!("  Interval {}: {:.1}ms", i, interval);
                } else if i == 6 {
                    println!("  ...");
                }
            }
            
            // Verify intervals get shorter (tempo increases)
            let early_avg = intervals[0..3.min(intervals.len())].iter().sum::<f64>() / 3.0_f64.min(intervals.len() as f64);
            let late_avg = if intervals.len() > 3 {
                let start = intervals.len() - 3;
                intervals[start..].iter().sum::<f64>() / 3.0
            } else {
                early_avg
            };
            
            println!("  Early average: {:.1}ms", early_avg);
            println!("  Late average:  {:.1}ms", late_avg);
            
            if intervals.len() > 3 {
                assert!(late_avg < early_avg, "Later intervals should be shorter (faster tempo)");
            }
        }
        
        // Expected beat counts for 4/4, 6/8, 4/4 section
        let expected_4_4_beats = 8;  // Two 4/4 bars = 8 quarter notes
        let expected_6_8_beats = 6;  // One 6/8 bar = 6 eighth notes
        let total_expected = expected_4_4_beats + expected_6_8_beats;
        
        println!("\nExpected vs Actual:");
        println!("  Expected 4/4 beats: {}", expected_4_4_beats);
        println!("  Expected 6/8 beats: {}", expected_6_8_beats);
        println!("  Expected total:     {}", total_expected);
        println!("  Actual total:       {}", events.len());
        
        // Verify we got the expected number of beats
        assert_eq!(events.len(), total_expected as usize, 
                  "Should have exactly {} beats", total_expected);
        
        // Verify time signatures are correct
        assert!(beats_by_signature.contains_key(&(4, 4)), "Should have 4/4 beats");
        assert!(beats_by_signature.contains_key(&(6, 8)), "Should have 6/8 beats");
        assert_eq!(beats_by_signature[&(4, 4)].len(), expected_4_4_beats, "Should have {} 4/4 beats", expected_4_4_beats);
        assert_eq!(beats_by_signature[&(6, 8)].len(), expected_6_8_beats, "Should have {} 6/8 beats", expected_6_8_beats);
        
        println!("\n✅ FMTickerBase test completed successfully!");
        println!("   - Correct number of beats generated");
        println!("   - Tempo progression working (60→90 BPM)");
        println!("   - Time signatures correct (4/4, 6/8, 4/4)");
        println!("   - Beat timing intervals decreasing as expected");
    }

    #[test]
    fn test_musical_timing_calculation() {
        println!("\n=== Musical Timing Calculation Test ===");
        
        // Test the core MusicalTiming calculation directly
        let bars = vec![
            FMBarElement::new(4, 4, 0.0, None),
            FMBarElement::new(6, 8, 0.0, None),
            FMBarElement::new(4, 4, 0.0, None),
        ];
        
        let timing = MusicalTiming::new(&bars, 60.0, 90.0).unwrap();
        
        println!("Section analysis:");
        println!("  Total duration: {:.1}ms", timing.total_duration_ms);
        println!("  Total beats: {}", timing.beat_events.len());
        println!("  Start tempo: {:.1} BPM", timing.start_tempo_bpm);
        println!("  End tempo: {:.1} BPM", timing.end_tempo_bpm);
        
        // Analyze beat events by bar
        let mut events_by_bar = std::collections::HashMap::new();
        for event in &timing.beat_events {
            let entry = events_by_bar.entry(event.bar_index).or_insert(Vec::new());
            entry.push(event);
        }
        
        println!("\nBeats by bar:");
        for (bar_index, bar_events) in &events_by_bar {
            let bar = &bars[*bar_index];
            println!("  Bar {}: {}/{} - {} beats", 
                    bar_index + 1, bar.nom, bar.denom, bar_events.len());
            
            // Show first and last beat timing for this bar
            if let (Some(first), Some(last)) = (bar_events.first(), bar_events.last()) {
                println!("    Time span: {:.1}ms - {:.1}ms (duration: {:.1}ms)",
                        first.time_offset_ms, last.time_offset_ms,
                        last.time_offset_ms - first.time_offset_ms);
            }
        }
        
        // Verify beat type distribution
        let mut beat_type_counts = std::collections::HashMap::new();
        for event in &timing.beat_events {
            *beat_type_counts.entry(event.beat_type).or_insert(0) += 1;
        }
        
        println!("\nBeat type distribution:");
        for (beat_type, count) in beat_type_counts {
            println!("  {:?}: {}", beat_type, count);
        }
        
        // We should have exactly 3 Major beats (one per bar)
        let major_beats = timing.beat_events.iter().filter(|e| e.beat_type == BeatType::Major).count();
        assert_eq!(major_beats, 3, "Should have exactly 3 Major beats (one per bar)");
        
        println!("\n✅ Musical timing calculation test passed!");
    }
}
