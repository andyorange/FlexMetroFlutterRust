//! Minimal Flutter Interface for Musical Timer
//! 
//! This module provides the ONLY interface between Rust and Flutter.
//! It uses a global event queue that Flutter can poll for beat events.

use crate::api::fm_bar_element::FMBarElement;
use crate::api::fm_ticker_base::{FMSectionTimer, BeatType, AsyncTimer, BeatEvent};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use once_cell::sync::Lazy;

/// Beat event data structure for Flutter UI updates
/// This is the ONLY data structure exposed to Flutter
#[derive(Debug, Clone, PartialEq)]
pub struct FlutterBeatEvent {
    /// Current bar metric (e.g., 4 for 4/4, 6 for 6/8, 0 for time-based)
    pub current_bar_nom: i32,
    pub current_bar_denom: i32,
    
    /// Beat type classification: "Major", "Medium", "Minor"
    pub beat_type: String,
    
    /// Subbeat position (1-based): for 6/8 this would be 1-6
    pub subbeat_position: i32,
    
    /// Next bar metric (if available)
    pub next_bar_nom: Option<i32>,
    pub next_bar_denom: Option<i32>,
    
    /// Timing information
    pub current_time_ms: i64,      // Current time within the current bar
    pub bar_duration_ms: Option<i64>, // Duration of current bar
}

/// Global state for timer management
static GLOBAL_TIMER: Lazy<Arc<Mutex<Option<FMSectionTimer>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});

/// Global event queue for Flutter to poll
static EVENT_QUEUE: Lazy<Arc<Mutex<VecDeque<FlutterBeatEvent>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(VecDeque::new()))
});

/// Global section information for next bar calculations
static SECTION_INFO: Lazy<Arc<Mutex<(Vec<FMBarElement>, f64, f64)>>> = Lazy::new(|| {
    Arc::new(Mutex::new((Vec::new(), 60.0, 90.0)))
});

/// Start a musical timer section
/// This is the main interface function that Flutter calls
pub fn start_musical_section_simple(
    bars: Vec<(i32, i32)>, // List of (nom, denom) pairs
    start_tempo_bpm: f64,
    end_tempo_bpm: f64,
) -> Result<(), String> {
    // Convert bar tuples to FMBarElement
    let bar_elements: Vec<FMBarElement> = bars
        .into_iter()
        .map(|(nom, denom)| FMBarElement::new(nom, denom, 0.0, None))
        .collect();

    // Store section info for next bar calculations
    if let Ok(mut section_guard) = SECTION_INFO.lock() {
        *section_guard = (bar_elements.clone(), start_tempo_bpm, end_tempo_bpm);
    } else {
        return Err("Failed to store section info".to_string());
    }

    // Clear any existing events in the queue
    if let Ok(mut queue_guard) = EVENT_QUEUE.lock() {
        queue_guard.clear();
    }

    // Create timer with section
    let mut timer = FMSectionTimer::new_with_section(bar_elements.clone(), start_tempo_bpm, end_tempo_bpm)
        .map_err(|e| format!("Failed to create timer: {}", e))?;

    // Set up the Rust callback that will add events to the queue
    let event_queue_clone = EVENT_QUEUE.clone();
    let section_info_clone = SECTION_INFO.clone();

    let rust_callback = {
        move |beat_event: BeatEvent| {
            // Calculate the current time within the bar and bar duration
            let (current_time_in_bar_ms, bar_duration_ms, next_bar_nom, next_bar_denom) = 
                if let Ok(section_guard) = section_info_clone.lock() {
                    let (ref bars, _start_tempo, _end_tempo) = *section_guard;
                    
                    // Get the current bar
                    let fallback_bar = FMBarElement::new(4, 4, 0.0, None);
                    let current_bar = if beat_event.bar_index < bars.len() {
                        &bars[beat_event.bar_index]
                    } else {
                        // Fallback to the last bar if index is out of bounds
                        bars.last().unwrap_or(&fallback_bar)
                    };

                    // Calculate bar duration based on tempo and time signature
                    let bar_duration = if current_bar.has_signature {
                        // For beat-based bars: calculate from tempo and time signature
                        let quarter_notes_per_bar = (current_bar.nom as f64 * 4.0) / current_bar.denom as f64;
                        let current_tempo_bpm = beat_event.tempo_bpm; // Use precise tempo from beat event
                        let bar_duration_ms = (quarter_notes_per_bar * 60000.0) / current_tempo_bpm;
                        bar_duration_ms as i64
                    } else {
                        // For time-based bars: use fixed duration
                        (current_bar.nom_secs * 1000.0) as i64
                    };

                    // Calculate current position within the bar
                    // Use a simpler calculation based on beat position
                    let total_subbeats_in_bar = current_bar.nom as usize;
                    let beats_per_subbeat = current_bar.nom as usize / total_subbeats_in_bar.max(1);
                    let current_subbeat_index = beat_event.beat_in_bar * beats_per_subbeat + beat_event.subbeat_in_beat;
                    let subbeat_duration_ms = bar_duration / total_subbeats_in_bar as i64;
                    let current_time_in_bar = current_subbeat_index as i64 * subbeat_duration_ms;

                    // Get next bar info
                    let next_bar = if beat_event.bar_index + 1 < bars.len() {
                        let next = &bars[beat_event.bar_index + 1];
                        (Some(next.nom), Some(next.denom))
                    } else {
                        (None, None)
                    };

                    (current_time_in_bar, Some(bar_duration), next_bar.0, next_bar.1)
                } else {
                    // Fallback values if we can't access section info
                    (0, None, None, None)
                };

            // Calculate proper subbeat position (1-based position within the bar)
            let subbeat_position = (beat_event.beat_in_bar * beat_event.nom as usize + beat_event.subbeat_in_beat + 1) as i32;

            // Create Flutter beat event with all the required information
            let flutter_event = FlutterBeatEvent {
                current_bar_nom: beat_event.nom,
                current_bar_denom: beat_event.denom,
                beat_type: match beat_event.beat_type {
                    BeatType::Major => "Major".to_string(),
                    BeatType::Medium => "Medium".to_string(),
                    BeatType::Minor => "Minor".to_string(),
                },
                subbeat_position,
                next_bar_nom,
                next_bar_denom,
                current_time_ms: current_time_in_bar_ms,
                bar_duration_ms,
            };

            // Add event to queue for Flutter to poll
            if let Ok(mut queue_guard) = event_queue_clone.lock() {
                queue_guard.push_back(flutter_event);
                // Keep queue size reasonable (last 100 events)
                while queue_guard.len() > 100 {
                    queue_guard.pop_front();
                }
            }
        }
    };

    timer.set_tick_callback(rust_callback)
        .map_err(|e| format!("Failed to set callback: {}", e))?;

    // Start the timer
    timer.start()
        .map_err(|e| format!("Failed to start timer: {}", e))?;

    // Store timer globally
    if let Ok(mut timer_guard) = GLOBAL_TIMER.lock() {
        *timer_guard = Some(timer);
        Ok(())
    } else {
        Err("Failed to store timer".to_string())
    }
}

/// Get the latest beat events from the event queue
/// Flutter calls this to poll for new events
pub fn get_beat_events() -> Vec<FlutterBeatEvent> {
    if let Ok(mut queue_guard) = EVENT_QUEUE.lock() {
        let events: Vec<FlutterBeatEvent> = queue_guard.drain(..).collect();
        events
    } else {
        Vec::new()
    }
}

/// Stop the current musical timer
pub fn stop_musical_timer() -> Result<(), String> {
    if let Ok(mut timer_guard) = GLOBAL_TIMER.lock() {
        if let Some(ref mut timer) = *timer_guard {
            timer.stop()
                .map_err(|e| format!("Failed to stop timer: {}", e))?;
        }
        *timer_guard = None;
        
        // Clear event queue
        if let Ok(mut queue_guard) = EVENT_QUEUE.lock() {
            queue_guard.clear();
        }
        
        Ok(())
    } else {
        Err("Failed to access timer".to_string())
    }
}

/// Check if timer is currently running
pub fn is_timer_running() -> bool {
    if let Ok(timer_guard) = GLOBAL_TIMER.lock() {
        timer_guard.as_ref().map_or(false, |timer| timer.is_running())
    } else {
        false
    }
}

// Backward compatibility functions for existing tests
// These will be removed once tests are updated to use the new interface

/// Initialize flutter timer (backward compatibility)
pub fn init_flutter_timer() -> Result<(), String> {
    // No initialization needed with new interface
    Ok(())
}

/// Create section (backward compatibility)
pub fn flutter_create_section(
    _bars: Vec<(i32, i32)>,
    _start_tempo_bpm: f64,
    _end_tempo_bpm: f64,
) -> Result<String, String> {
    // This would be handled by start_musical_section in new interface
    Ok("Section created (compatibility mode)".to_string())
}

/// Start timer with config (backward compatibility)
pub fn start_flutter_musical_timer_with_config(
    bars: Vec<(i32, i32)>,
    start_tempo_bpm: f64,
    end_tempo_bpm: f64,
) -> String {
    match start_musical_section_simple(bars, start_tempo_bpm, end_tempo_bpm) {
        Ok(_) => "Timer started successfully".to_string(),
        Err(e) => format!("Failed to start timer: {}", e),
    }
}

/// Stop timer (backward compatibility)
pub fn stop_flutter_musical_timer() -> String {
    match stop_musical_timer() {
        Ok(_) => "Timer stopped successfully".to_string(),
        Err(e) => format!("Failed to stop timer: {}", e),
    }
}
