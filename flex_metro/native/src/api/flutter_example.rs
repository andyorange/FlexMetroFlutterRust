//! Example: How to use the minimal Flutter Interface
//! 
//! This example demonstrates the complete usage of the minimal Flutter-Rust
//! interface for the musical timer using the event queue approach.

use crate::api::flutter_interface::{start_musical_section_simple, stop_musical_timer, get_beat_events, FlutterBeatEvent};

/// Example of how Flutter would process beat events
/// This simulates how Flutter would handle beat events from the queue
pub fn example_flutter_event_processor(event: FlutterBeatEvent) {
    println!("🎵 Flutter UI Update:");
    println!("   Current Bar: {}/{}", event.current_bar_nom, event.current_bar_denom);
    println!("   Beat Type: {}", event.beat_type);
    println!("   Subbeat Position: {}", event.subbeat_position);
    
    if let (Some(next_nom), Some(next_denom)) = (event.next_bar_nom, event.next_bar_denom) {
        println!("   Next Bar: {}/{}", next_nom, next_denom);
    } else {
        println!("   Next Bar: (last bar)");
    }
    
    println!("   Time: {}ms", event.current_time_ms);
    
    if let Some(duration) = event.bar_duration_ms {
        println!("   Bar Duration: {}ms", duration);
    }
    
    println!("   ---");
}

/// Example of how Flutter would start a musical timer and poll for events
pub fn example_start_musical_timer() -> Result<(), String> {
    println!("🚀 Starting Musical Timer Example");
    
    // Define the musical section: 4/4, 6/8, 4/4 bars
    let bars = vec![(4, 4), (6, 8), (4, 4)];
    let start_tempo = 60.0; // BPM (quarter notes per minute)
    let end_tempo = 90.0;   // BPM (quarter notes per minute)
    
    println!("📊 Configuration:");
    println!("   Bars: {:?}", bars);
    println!("   Tempo: {} → {} BPM", start_tempo, end_tempo);
    
    // Start the timer - this creates the timer and starts adding events to the queue
    start_musical_section_simple(bars, start_tempo, end_tempo)?;
    
    println!("✅ Musical timer started successfully!");
    println!("   Beat events will be queued for Flutter UI...");
    
    // Simulate Flutter polling for events
    println!("📱 Simulating Flutter polling loop:");
    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_secs() < 3 {
        // Get events from the queue
        let events = get_beat_events();
        
        // Process each event (Flutter UI updates)
        for event in events {
            example_flutter_event_processor(event);
        }
        
        // Flutter would typically poll at 60fps (16ms intervals)
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    
    Ok(())
}

/// Example of stopping the timer
pub fn example_stop_musical_timer() -> Result<(), String> {
    println!("🛑 Stopping musical timer...");
    stop_musical_timer()?;
    println!("✅ Musical timer stopped successfully!");
    Ok(())
}

/// Complete example demonstrating the full lifecycle
pub fn run_complete_example() -> Result<(), String> {
    println!("\n=== Complete Flutter Musical Timer Example ===");
    
    // Start the timer
    example_start_musical_timer()?;
    
    // In a real Flutter app, the timer would run and send beat events
    // to the Flutter UI. Here we simulate that by waiting a bit.
    println!("🕐 Running timer for 5 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    // Stop the timer
    example_stop_musical_timer()?;
    
    println!("🎯 Example completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_complete_example() {
        println!("\n=== Testing Complete Flutter Interface Example ===");
        
        match run_complete_example() {
            Ok(_) => println!("✅ Complete example test passed!"),
            Err(e) => panic!("❌ Complete example test failed: {}", e),
        }
    }
}
