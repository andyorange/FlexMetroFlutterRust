// Real-time FMTickerBase test - 4/4, 6/8, 4/4 bars with tempo 60->90 BPM
// This test demonstrates the timer running in real-time with live callback events

use crate::api::fm_ticker_base::{FMSectionTimer, BeatType, AsyncTimer};
use crate::api::fm_bar_element::FMBarElement;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(test)]
mod realtime_tests {
    use super::*;

    #[test]
    fn test_realtime_fm_ticker_streaming() {
        println!("\n🎼 === Real-Time FMTickerBase Test: 4/4, 6/8, 4/4 | Tempo 60->90 BPM ===");
        
        // Create the musical section: 4/4, 6/8, 4/4
        let bars = vec![
            FMBarElement::new(4, 4, 0.0, None), // First 4/4 bar
            FMBarElement::new(6, 8, 0.0, None), // 6/8 bar  
            FMBarElement::new(4, 4, 0.0, None), // Second 4/4 bar
        ];
        
        println!("🎵 Section: 4/4 → 6/8 → 4/4");
        println!("🎯 Tempo: 60.0 BPM → 90.0 BPM");
        
        // Create timer with the musical section
        let mut timer = FMSectionTimer::new_with_section(bars.clone(), 60.0, 90.0)
            .expect("Failed to create timer");
        
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
            
            println!("🎵 Live Beat #{:2}/14 | {}/{} | {} | Position: {}.{} | Tempo: {:5.1} BPM | Scheduled: {:6.1}ms | Real: {:6.0}ms",
                    current_count,
                    beat_event.nom, beat_event.denom,
                    beat_type_str,
                    beat_event.beat_in_bar + 1, beat_event.subbeat_in_beat + 1,
                    beat_event.tempo_bpm,
                    beat_event.time_offset_ms,
                    elapsed_real);
                    
            // Stop when we've captured all expected beats
            if current_count >= 14 {
                println!("🎉 All beats captured! Real-time streaming complete.");
            }
        }).expect("Failed to set callback");
        
        println!("\n🚀 Starting real-time timer... (beats will stream live)");
        timer.start().expect("Failed to start timer");
        
        // Let the timer run until all beats are captured or timeout
        let timeout = Duration::from_secs(15); // Safety timeout
        let start = std::time::Instant::now();
        
        loop {
            std::thread::sleep(Duration::from_millis(100)); // Small sleep to avoid busy waiting
            
            let current_count = *beat_count.lock().unwrap();
            
            // Check if we've captured all beats
            if current_count >= 14 {
                println!("\n✅ All 14 beats captured via real-time streaming!");
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
        
        println!("\n📊 === Real-Time Streaming Results ===");
        println!("🎵 Total beats captured: {}/14", final_count);
        println!("⏱️  Total real time: {:.1}s", total_elapsed.as_secs_f64());
        println!("🎯 Average beat interval: {:.1}ms", total_elapsed.as_millis() as f64 / final_count as f64);
        
        // Verify we got the expected number of beats
        assert_eq!(final_count, 14, "Should have captured all 14 beats via real-time streaming");
        
        println!("✅ Real-time FMTickerBase streaming test completed successfully!");
        println!("   🎼 Timer handled all timing automatically");
        println!("   🔄 Callback received live beat events");  
        println!("   ⏰ No manual sleep timing required");
        println!("   🎯 All beats streamed in real-time");
    }
}
