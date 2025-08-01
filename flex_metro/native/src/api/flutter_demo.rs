// Complete example demonstrating the Flutter-Rust musical timer interface
// This shows how Flutter would use the Rust timer through the minimal event queue interface

use crate::api::flutter_interface::{start_musical_section_simple, stop_musical_timer, get_beat_events, FlutterBeatEvent};
use std::collections::HashMap;

/// Example demonstrating the complete Flutter interface workflow
pub fn demo_flutter_interface() -> Result<(), String> {
    println!("\n=== Flutter Interface Demo ===");

    // Define a musical section: 4/4, 6/8, 4/4 with tempo change
    let bars = vec![(4, 4), (6, 8), (4, 4)];
    let start_tempo_bpm = 60.0;
    let end_tempo_bpm = 90.0;

    println!("Musical section: {:?}", bars);
    println!("Tempo: {:.1} → {:.1} BPM", start_tempo_bpm, end_tempo_bpm);

    // Start the musical timer - this is what Flutter would call
    start_musical_section_simple(bars, start_tempo_bpm, end_tempo_bpm)?;

    println!("⏰ Timer started - polling for events for 5 seconds...");
    
    // This is what Flutter would do - poll for events and update UI
    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_secs() < 5 {
        let events = get_beat_events();
        
        for event in events {
            // Flutter receives this data for UI updates:
            println!("🎵 Flutter UI Update:");
            println!("   Bar: {}/{}", event.current_bar_nom, event.current_bar_denom);
            println!("   Beat: {} ({})", event.subbeat_position, event.beat_type);
            
            if let Some(duration) = event.bar_duration_ms {
                let progress = (event.current_time_ms as f64 / duration as f64 * 100.0).min(100.0);
                println!("   Progress: {:.1}% ({}/{}ms)", progress, event.current_time_ms, duration);
            }
            
            if let (Some(next_nom), Some(next_denom)) = (event.next_bar_nom, event.next_bar_denom) {
                println!("   Next bar: {}/{}", next_nom, next_denom);
            } else {
                println!("   Next bar: End of section");
            }
            println!();
        }
        
        // Flutter would poll at its own frame rate (e.g., 60fps = ~16ms)
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Stop the timer
    stop_musical_timer()?;
    println!("⏸️  Timer stopped");
    
    println!("✅ Demo completed successfully!");
    Ok(())
}

/// Example showing how to collect beat events for analysis
pub fn demo_beat_event_collection() -> Result<(), String> {
    println!("\n=== Beat Event Collection Demo ===");

    let bars = vec![(3, 4), (5, 8)]; // Unusual time signatures
    let tempo = 100.0; // Constant tempo

    // Start collection
    start_musical_section_simple(bars, tempo, tempo)?;
    
    let mut collected_events = Vec::<FlutterBeatEvent>::new();
    
    // Collect events for 4 seconds
    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_secs() < 4 {
        let events = get_beat_events();
        collected_events.extend(events);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    
    stop_musical_timer()?;

    // Analyze collected events
    println!("Collected {} beat events", collected_events.len());

    // Group by bar type
    let mut bar_counts = HashMap::new();
    for event in collected_events.iter() {
        let key = (event.current_bar_nom, event.current_bar_denom);
        *bar_counts.entry(key).or_insert(0) += 1;
    }

    println!("Events per bar type:");
    for ((nom, denom), count) in &bar_counts {
        println!("  {}/{}: {} events", nom, denom, count);
    }

    // Show timing analysis
    if let (Some(first), Some(last)) = (collected_events.first(), collected_events.last()) {
        println!("Timing span: {}ms - {}ms", first.current_time_ms, last.current_time_ms);
    }

    println!("✅ Collection demo completed!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flutter_demo() {
        assert!(demo_flutter_interface().is_ok());
    }

    #[test] 
    fn test_collection_demo() {
        assert!(demo_beat_event_collection().is_ok());
    }
}
