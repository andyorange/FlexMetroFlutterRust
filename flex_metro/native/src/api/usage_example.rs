// Example: How to use your FMTickerBase in other parts of your project

// Import the types from your module
use crate::api::fm_ticker_base::{
    FMSectionTimer, 
    FMCircleTicker
};

// Now you can use the types
pub fn example_usage() -> Result<(), Box<dyn std::error::Error>> {
    // Create a timer - now uses TimeDelta and returns Result
    let time_delta = chrono::TimeDelta::try_milliseconds(500)
        .ok_or("Failed to create TimeDelta")?;
    let mut timer = FMSectionTimer::new(time_delta)?;
    
    // Set up callback
    timer.set_tick_callback(|beat_event| {
        println!("Beat Event - Bar: {}, Beat: {}/{}, Type: {:?}, Time: {}ms", 
                beat_event.bar_index + 1, beat_event.beat_in_bar + 1, beat_event.subbeat_in_beat + 1,
                beat_event.beat_type, beat_event.time_offset_ms);
    })?;
    
    // Create ticker
    let mut ticker = FMCircleTicker::new();
    
    // Connect timer to ticker
    ticker.connect_timer(timer)?;
    
    // Start the timer
    ticker.start_timer()?;
    
    // Later... stop it
    ticker.stop_timer()?;
    
    Ok(())
}
