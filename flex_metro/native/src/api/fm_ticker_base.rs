//! FMTickerBase - Async Timer Implementation
//! 
//! This module provides an asynchronous timer implementation that matches the Python FMTickerBase interface.
//! 
//! ## Key Components:
//! 
//! 1. **AsyncTimer trait** - Base interface for timer functionality
//! 2. **FMSectionTimer struct** - Concrete timer implementation that inherits from AsyncTimer
//! 3. **FMTickerBase trait** - Abstract interface matching Python implementation
//! 4. **FMCircleTicker struct** - Example concrete ticker implementation
//! 
//! ## Usage:
//! 
//! ```rust
//! use std::time::Duration;
//! 
//! // Create a timer with 1 second intervals
//! let mut timer = FMSectionTimer::new(Duration::from_secs(1));
//! 
//! // Set up a callback function - now returns Result
//! timer.set_tick_callback(|section_info, count, ignore_subbeats| {
//!     println!("Tick #{}: {:?}", count, section_info);
//! })?;
//! 
//! // Create a ticker and connect the timer
//! let mut ticker = FMCircleTicker::new();
//! ticker.connect_timer(timer)?;
//! 
//! // Start the timer - proper error handling
//! ticker.start_timer()?;
//! 
//! // Later... stop the timer - graceful shutdown
//! ticker.stop_timer()?;
//! ```

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::marker::{Send, Sync}; // Import Send and Sync traits
use chrono::TimeDelta;
use crate::api::fm_bar_element::FMBarElement;
use crate::api::fm_tempo_interval::{FMTempoInterval, FMTempoSequence};
// use crate::api::internationalization::{I18n, TKey}; // Temporarily disabled
use crate::{log_info, log_debug, log_warn, log_error};

/// Beat type classification for musical timing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BeatType {
    Major,  // Main beat - starts a bar (always beat 1)
    Medium, // Medium beat - starts a beat group from YAML config
    Minor,  // Minor beat - subdivisions within a beat group
}

/// Musical timing calculator that generates precise beat events based on time signatures and tempos
/// within a sequence of tempo intervals with potentially changing time signatures and tempos
#[derive(Debug)]
pub struct MusicalTiming {
    pub beat_events: Vec<BeatEvent>,
    pub total_duration_ms: f64,
    pub start_tempo_bpm: f64,
    pub end_tempo_bpm: f64,
    pub tempo_sequence: FMTempoSequence,
}

#[derive(Debug, Clone)]
// // #[flutter_rust_bridge::frb(opaque)]
pub struct BeatEvent {
    pub time_offset_ms: f64,
    pub bar_index: usize,
    pub beat_in_bar: usize,
    pub subbeat_in_beat: usize,
    pub tempo_bpm: f64,  // Current tempo at this beat
    pub nom: i32,
    pub denom: i32,
    pub beat_type: BeatType,  // Major, Medium, or Minor beat
}

impl MusicalTiming {
    /// Create a new musical timing calculator for a sequence of tempo intervals
    /// This is the new preferred method for complex tempo sequences
    pub fn new_from_sequence(tempo_sequence: FMTempoSequence) -> Result<Self, String> {
        if tempo_sequence.is_empty() {
            return Err("Tempo sequence must contain at least one interval".to_string());
        }

        let start_tempo_bpm = tempo_sequence.intervals[0].start_tempo_bpm;
        let end_tempo_bpm = tempo_sequence.intervals.last()
            .ok_or("Tempo sequence is unexpectedly empty after non-empty check")?
            .end_tempo_bpm;

        let mut timing = MusicalTiming {
            beat_events: Vec::new(),
            total_duration_ms: 0.0,
            start_tempo_bpm,
            end_tempo_bpm,
            tempo_sequence: tempo_sequence.clone(),
        };

        timing.calculate_beat_events_from_sequence(&tempo_sequence)
            .map_err(|e| format!("Failed to calculate beat events from sequence: {}", e))?;
        Ok(timing)
    }

    /// Create a new musical timing calculator for a section of bars with tempo
    /// start_tempo_bpm and end_tempo_bpm should be in quarter notes per minute
    /// This method is kept for backward compatibility
    // // #[flutter_rust_bridge::frb(ignore)]
    pub fn new(bars: &[FMBarElement], start_tempo_bpm: f64, end_tempo_bpm: f64) -> Result<Self, String> {
        if bars.is_empty() {
            return Err("Section must contain at least one bar".to_string());
        }

        // Convert the old single-section approach to use tempo intervals
        let interval = FMTempoInterval::new(bars.to_vec(), start_tempo_bpm, Some(end_tempo_bpm), "Default Interval".to_string());
        let sequence = FMTempoSequence::from_interval(interval);
        Self::new_from_sequence(sequence)
    }

    /// Calculate all beat events from a tempo sequence with precise timing and tempo changes
    /// This implementation handles multiple tempo intervals with different tempo progressions
    fn calculate_beat_events_from_sequence(&mut self, tempo_sequence: &FMTempoSequence) -> Result<(), String> {
        self.beat_events.clear();
        
        let mut current_time_ms = 0.0;
        let mut global_bar_index = 0;

        for (interval_index, interval) in tempo_sequence.intervals.iter().enumerate() {
            log_debug(&format!("Processing interval {}: {}", interval_index + 1, interval.description()));
            
            // Calculate total quarter notes for this interval
            let total_quarter_notes = self.calculate_total_quarter_notes(&interval.bars)
                .map_err(|e| format!("Failed to calculate quarter notes for interval {}: {}", interval_index, e))?;
            
            let mut quarter_note_position = 0.0; // Track position within this interval

            for (local_bar_index, bar) in interval.bars.iter().enumerate() {
                let beat_groups = self.extract_beats(bar)
                    .map_err(|e| format!("Failed to extract beats from bar {} in interval {}: {}", local_bar_index, interval_index, e))?;
                
                let quarter_notes_per_bar = if bar.has_signature {
                    (bar.nom as f64 * 4.0) / bar.denom as f64
                } else {
                    // For time-based bars, estimate based on interval's average tempo
                    let avg_tempo = (interval.start_tempo_bpm + interval.end_tempo_bpm) / 2.0;
                    let duration_minutes = bar.nom_secs as f64 / 60.0;
                    avg_tempo * duration_minutes
                };
                
                // Calculate the duration of each individual note unit in the bar
                let total_note_units: i32 = beat_groups.iter().sum();
                let quarter_notes_per_note_unit = quarter_notes_per_bar / total_note_units as f64;
                
                let mut is_first_beat_in_bar = true;
                let mut absolute_beat_in_bar = 0; // Track absolute beat position in bar

                // Iterate through each beat group
                for (group_index, &group_size) in beat_groups.iter().enumerate() {
                    // Iterate through each note in this beat group
                    for note_in_group in 0..group_size {
                        // Calculate progress within this interval for tempo interpolation
                        let interval_progress = if total_quarter_notes > 0.0 {
                            quarter_note_position / total_quarter_notes
                        } else {
                            0.0
                        };
                        let current_tempo = interval.start_tempo_bpm + 
                            (interval.end_tempo_bpm - interval.start_tempo_bpm) * interval_progress;

                        // Determine beat type dynamically based on position in beat structure
                        let beat_type = if is_first_beat_in_bar {
                            BeatType::Major  // First beat of the bar is always Major
                        } else if note_in_group == 0 && group_index > 0 {
                            BeatType::Medium // First beat of a group (after the first group) is Medium
                        } else {
                            BeatType::Minor  // All other beats are Minor
                        };

                        // Create beat event with global bar index
                        self.beat_events.push(BeatEvent {
                            time_offset_ms: current_time_ms,
                            bar_index: global_bar_index,
                            beat_in_bar: group_index,
                            subbeat_in_beat: absolute_beat_in_bar, // Use absolute position in bar
                            tempo_bpm: current_tempo,
                            nom: bar.nom,
                            denom: bar.denom,
                            beat_type,
                        });

                        // Calculate the time duration for this note unit with changing tempo within the interval
                        let note_duration_ms = self.calculate_note_duration_with_interval_tempo_change(
                            quarter_note_position, 
                            quarter_notes_per_note_unit, 
                            total_quarter_notes,
                            interval.start_tempo_bpm,
                            interval.end_tempo_bpm
                        );
                        
                        // Advance time and quarter note position
                        current_time_ms += note_duration_ms;
                        quarter_note_position += quarter_notes_per_note_unit;
                        absolute_beat_in_bar += 1; // Increment absolute beat position
                        is_first_beat_in_bar = false;
                    }
                }
                global_bar_index += 1; // Increment global bar index
            }
        }

        self.total_duration_ms = current_time_ms;
        log_debug(&format!("Generated {} beat events across {} intervals, total duration: {:.1}ms", 
                          self.beat_events.len(), tempo_sequence.intervals.len(), self.total_duration_ms));
        Ok(())
    }

    /// Calculate all beat events with their precise timing and tempo changes
    /// This implementation uses smooth tempo interpolation throughout the entire section
    fn calculate_beat_events(&mut self, bars: &[FMBarElement]) -> Result<(), String> {
        self.beat_events.clear();
        
        // First, calculate the total number of quarter note units in the section
        let total_quarter_notes = self.calculate_total_quarter_notes(bars).map_err(|e| format!("Failed to calculate quarter notes: {}", e))?;
        
        let mut current_time_ms = 0.0;
        let mut quarter_note_position = 0.0; // Track position in quarter notes

        for (bar_index, bar) in bars.iter().enumerate() {
            let beat_groups = self.extract_beats(bar).map_err(|e| format!("Failed to extract beats from bar {}: {}", bar_index, e))?;
            let quarter_notes_per_bar = if bar.has_signature {
                (bar.nom as f64 * 4.0) / bar.denom as f64
            } else {
                // For time-based bars, estimate based on average tempo
                let avg_tempo = (self.start_tempo_bpm + self.end_tempo_bpm) / 2.0;
                let duration_minutes = bar.nom_secs as f64 / 60.0;
                avg_tempo * duration_minutes
            };
            
            // Calculate the duration of each individual note unit in the bar
            let total_note_units: i32 = beat_groups.iter().sum();
            let quarter_notes_per_note_unit = quarter_notes_per_bar / total_note_units as f64;
            
            let mut is_first_beat_in_bar = true;
            let mut absolute_beat_in_bar = 0; // Track absolute beat position in bar

            // Iterate through each beat group
            for (group_index, &group_size) in beat_groups.iter().enumerate() {
                // Iterate through each note in this beat group
                for note_in_group in 0..group_size {
                    // Calculate section progress for tempo interpolation
                    let section_progress = quarter_note_position / total_quarter_notes;
                    let current_tempo = self.start_tempo_bpm + (self.end_tempo_bpm - self.start_tempo_bpm) * section_progress;

                    // Determine beat type dynamically based on position in beat structure
                    let beat_type = if is_first_beat_in_bar {
                        BeatType::Major  // First beat of the bar is always Major
                    } else if note_in_group == 0 && group_index > 0 {
                        BeatType::Medium // First beat of a group (after the first group) is Medium
                    } else {
                        BeatType::Minor  // All other beats are Minor
                    };

                    // Create beat event
                    self.beat_events.push(BeatEvent {
                        time_offset_ms: current_time_ms,
                        bar_index,
                        beat_in_bar: group_index,
                        subbeat_in_beat: absolute_beat_in_bar, // Use absolute position in bar
                        tempo_bpm: current_tempo,
                        nom: bar.nom,
                        denom: bar.denom,
                        beat_type,
                    });

                    // Calculate the time duration for this note unit with changing tempo
                    let note_duration_ms = self.calculate_note_duration_with_tempo_change(
                        quarter_note_position, 
                        quarter_notes_per_note_unit, 
                        total_quarter_notes
                    );
                    
                    // Advance time and quarter note position
                    current_time_ms += note_duration_ms;
                    quarter_note_position += quarter_notes_per_note_unit;
                    absolute_beat_in_bar += 1; // Increment absolute beat position
                    is_first_beat_in_bar = false;
                }
            }
        }

        self.total_duration_ms = current_time_ms;
        Ok(())
    }

    /// Convert tempo notation (notes_per_minute, base_note) to BPM for quarter notes
    #[allow(dead_code)]
    fn convert_tempo_to_bpm(&self, notes_per_minute: i32, base_note: i32) -> f64 {
        // Convert to quarter note BPM: (notes_per_minute * 4) / base_note
        // Example: (128, 8) = 128 eighth notes/min = (128 * 4) / 8 = 64 quarter notes/min
        (notes_per_minute as f64 * 4.0) / base_note as f64
    }

    /// Calculate the total number of quarter notes in the section
    fn calculate_total_quarter_notes(&self, bars: &[FMBarElement]) -> Result<f64, String> {
        let mut total_quarter_notes = 0.0;
        for bar in bars {
            if bar.has_signature {
                // Convert to quarter notes: (nom * 4) / denom
                total_quarter_notes += (bar.nom as f64 * 4.0) / bar.denom as f64;
            } else {
                // For time-based bars, estimate based on average tempo
                let avg_tempo = (self.start_tempo_bpm + self.end_tempo_bpm) / 2.0;
                let duration_minutes = bar.nom_secs as f64 / 60.0;
                total_quarter_notes += avg_tempo * duration_minutes;
            }
        }
        Ok(total_quarter_notes)
    }

    /// Calculate the duration of a bar in milliseconds based on its time signature and current tempo
    #[allow(dead_code)]
    fn calculate_bar_duration_ms(&self, bar: &FMBarElement, bar_index: usize, total_bars: usize) -> Result<f64, String> {
        if bar.has_signature {
            // Time signature based: calculate from nom/denom and interpolated tempo
            // Calculate the tempo at the start of this bar
            let bar_progress = if total_bars > 1 {
                bar_index as f64 / (total_bars - 1) as f64
            } else {
                0.0
            };
            let bar_tempo_bpm = self.start_tempo_bpm + (self.end_tempo_bpm - self.start_tempo_bpm) * bar_progress;
            
            // Duration = (beats_per_bar / quarter_notes_per_beat) * (60000 ms/min) / (quarter_notes/min)
            let quarter_notes_per_bar = (bar.nom as f64 * 4.0) / bar.denom as f64;
            let duration_ms = (quarter_notes_per_bar * 60000.0) / bar_tempo_bpm;
            Ok(duration_ms)
        } else {
            // Time-based: use nom_secs directly
            Ok(bar.nom_secs as f64 * 1000.0)
        }
    }

    /// Extract beat durations from a bar element
    fn extract_beats(&self, bar: &FMBarElement) -> Result<Vec<i32>, String> {
        match &bar.beats {
            crate::api::fm_bar_element::BarBeatData::Int(beats) => Ok(beats.clone()),
            crate::api::fm_bar_element::BarBeatData::Float(beats) => {
                Ok(beats.iter().map(|&b| b as i32).collect())
            }
        }
    }

    /// Calculate total beat units for tempo progression calculation
    #[allow(dead_code)]
    fn calculate_total_beat_units(&self, beats: &[i32], subbeats: &[i32]) -> i32 {
        beats.iter().sum::<i32>() + subbeats.iter().sum::<i32>()
    }

    /// Get the next beat event at or after the given time
    pub fn get_next_beat_event(&self, current_time_ms: f64) -> Option<BeatEvent> {
        self.beat_events.iter()
            .find(|event| event.time_offset_ms >= current_time_ms)
            .cloned()
    }

    /// Get all beat events for iteration
    pub fn get_beat_events(&self) -> &[BeatEvent] {
        &self.beat_events
    }

    /// Get the total duration of the musical section in milliseconds
    pub fn get_total_duration_ms(&self) -> f64 {
        self.total_duration_ms
    }

    /// Calculate the total duration of the section with linearly changing tempo
    /// Uses numerical integration to account for tempo change throughout the section
    #[allow(dead_code)]
    fn calculate_section_duration_with_tempo_change(&self, total_quarter_notes: f64) -> f64 {
        // For linear tempo change from start_tempo to end_tempo over total_quarter_notes:
        // tempo(t) = start_tempo + (end_tempo - start_tempo) * (t / total_quarter_notes)
        // time = integral of (60000 / tempo(t)) dt from 0 to total_quarter_notes
        
        if (self.end_tempo_bpm - self.start_tempo_bpm).abs() < 1e-6 {
            // Constant tempo case
            return (total_quarter_notes * 60000.0) / self.start_tempo_bpm;
        }
        
        // For linear tempo change: integral of 1/tempo(t) dt
        // Let a = start_tempo, b = end_tempo, n = total_quarter_notes
        // tempo(t) = a + (b-a) * t/n
        // integral of 1/(a + (b-a)*t/n) dt from 0 to n
        // = (n/(b-a)) * ln((a + (b-a)*n/n) / (a + (b-a)*0/n))
        // = (n/(b-a)) * ln(b/a)
        
        let tempo_ratio = self.end_tempo_bpm / self.start_tempo_bpm;
        let duration_ms = (total_quarter_notes / (self.end_tempo_bpm - self.start_tempo_bpm)) * 
                         tempo_ratio.ln() * 60000.0;
        
        duration_ms
    }

    /// Calculate the duration of a single note unit with changing tempo within a specific interval
    /// Uses numerical integration for the portion of the interval this note occupies
    fn calculate_note_duration_with_interval_tempo_change(
        &self, 
        start_quarter_note_position: f64, 
        quarter_notes_in_note: f64,
        total_quarter_notes_in_interval: f64,
        interval_start_tempo: f64,
        interval_end_tempo: f64
    ) -> f64 {
        if (interval_end_tempo - interval_start_tempo).abs() < 1e-6 {
            // Constant tempo case
            return (quarter_notes_in_note * 60000.0) / interval_start_tempo;
        }
        
        // For a small segment with linear tempo change within this interval:
        // Use numerical integration with multiple steps for accuracy
        let num_steps = 10;
        let step_size = quarter_notes_in_note / num_steps as f64;
        let mut total_time = 0.0;
        
        for i in 0..num_steps {
            let t = start_quarter_note_position + (i as f64 + 0.5) * step_size;
            let progress = if total_quarter_notes_in_interval > 0.0 {
                t / total_quarter_notes_in_interval
            } else {
                0.0
            };
            let tempo_at_t = interval_start_tempo + (interval_end_tempo - interval_start_tempo) * progress;
            
            // Duration for this step: (quarter_notes * 60000ms/min) / (quarter_notes/min)
            let step_duration = (step_size * 60000.0) / tempo_at_t;
            total_time += step_duration;
        }
        
        total_time
    }

    /// Calculate the duration of a single note unit with changing tempo
    /// Uses numerical integration for the portion of the section this note occupies
    fn calculate_note_duration_with_tempo_change(
        &self, 
        start_quarter_note_position: f64, 
        quarter_notes_in_note: f64,
        total_quarter_notes: f64
    ) -> f64 {
        if (self.end_tempo_bpm - self.start_tempo_bpm).abs() < 1e-6 {
            // Constant tempo case
            return (quarter_notes_in_note * 60000.0) / self.start_tempo_bpm;
        }
        
        // For a small segment with linear tempo change:
        // Use numerical integration with multiple steps for accuracy
        let num_steps = 10;
        let step_size = quarter_notes_in_note / num_steps as f64;
        let mut total_time = 0.0;
        
        for i in 0..num_steps {
            let t = start_quarter_note_position + (i as f64 + 0.5) * step_size;
            let progress = t / total_quarter_notes;
            let tempo_at_t = self.start_tempo_bpm + (self.end_tempo_bpm - self.start_tempo_bpm) * progress;
            
            // Duration for this step: (quarter_notes * 60000ms/min) / (quarter_notes/min)
            let step_duration = (step_size * 60000.0) / tempo_at_t;
            total_time += step_duration;
        }
        
        total_time
    }
}

/*
## **Understanding Arc, Send, and Sync:**

### **Arc (Atomically Reference Counted)**:
- **Purpose**: Allows multiple owners of the same data across threads
- **Why needed here**: The callback needs to be shared between:
  1. The main thread (where it's stored in FMSectionTimer)  
  2. The timer thread (where it's called on each tick)
- **Alternative**: We could use `Rc` but it's not thread-safe
- **Memory safety**: Automatically cleans up when last reference is dropped

### **Send Trait**:
- **Purpose**: Marks types that can be safely transferred between threads
- **Why needed**: The callback will be moved to the timer thread
- **Requirement**: All captured variables in the closure must be Send

### **Sync Trait**:
- **Purpose**: Marks types that can be safely shared between threads (via Arc)
- **Why needed**: The callback will be accessed from both main and timer threads
- **Requirement**: The callback function itself must be Sync

### **Could we avoid Arc?**:
Yes, but with trade-offs:
1. **No Arc**: Callback couldn't be shared between threads
2. **No Send/Sync**: Timer couldn't run in separate thread
3. **Simpler alternative**: Single-threaded timer, but blocks main thread

The current design enables true async behavior without blocking the main thread.
*/

/*
## **Simpler Alternative Design (if Arc complexity is unwanted)**:

If you prefer to avoid Arc/Send/Sync complexity, here's a simpler approach:

```rust
pub struct SimpleFMSectionTimer {
    callback: Option<Box<dyn Fn(FMBarElement, i32, Option<bool>)>>, // No Send/Sync
    interval_duration: Duration,
    // No background thread - timer runs when explicitly called
}

impl SimpleFMSectionTimer {
    pub fn tick(&mut self, section_info: FMBarElement, count: i32) {
        if let Some(ref callback) = self.callback {
            callback(section_info, count, None);
        }
    }
}
```

**Trade-offs**:
- ✅ Simpler: No Arc, no Send/Sync, no thread management
- ✅ Easier to understand and debug
- ❌ Not truly async: Caller must drive the timer manually
- ❌ Can block main thread if callback is slow

Choose based on your use case: true async (current) vs. simplicity (alternative).
*/

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FMTickPositions {
    TNone,
    TSection,
    TRest,
    TMedium,
    TMinor,
}

// Base timer trait that FMSectionTimer will inherit from
// #[flutter_rust_bridge::frb(ignore)]
pub trait AsyncTimer {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn pause(&mut self) -> Result<(), String>;
    fn resume(&mut self) -> Result<(), String>;
    fn set_interval(&mut self, duration: std::time::Duration);  // Keep using Duration internally
    fn is_running(&self) -> bool;
    fn is_paused(&self) -> bool;
}

// FMSectionTimer struct that inherits from async timer functionality
pub struct FMSectionTimer {
    // Member variable: pointer to the virtual method callback (_tick_callback)
    _tick_callback: Arc<Mutex<Option<Box<dyn Fn(BeatEvent) + Send + Sync>>>>,

    // Timer state and control
    interval_duration: Duration,
    is_running: bool,
    is_paused: bool,
    timer_handle: Option<thread::JoinHandle<()>>,
    stop_sender: Option<Sender<()>>,
    pause_sender: Option<Sender<()>>,
    resume_sender: Option<Sender<()>>,
    
    // Musical timing for sections
    musical_timing: Option<MusicalTiming>,
    #[allow(dead_code)]
    section_bars: Vec<FMBarElement>,
}

impl FMSectionTimer {
    /// FRB-compatible constructor that accepts TimeDelta and converts to Duration
    // #[flutter_rust_bridge::frb(sync)]
    pub fn new(interval_duration: TimeDelta) -> Result<Self, String> {
        // Convert TimeDelta to std::time::Duration
        let duration = match interval_duration.to_std() {
            Ok(dur) => dur,
            Err(e) => return Err(format!("Invalid duration: {}", e)),
        };
        
        Ok(Self::new_internal(duration))
    }
    
    /// Internal constructor that uses std::time::Duration
    // #[flutter_rust_bridge::frb(ignore)]
    pub fn new_internal(interval_duration: Duration) -> Self {
        FMSectionTimer {
            _tick_callback: Arc::new(Mutex::new(None)),
            interval_duration,
            is_running: false,
            is_paused: false,
            timer_handle: None,
            stop_sender: None,
            pause_sender: None,
            resume_sender: None,
            musical_timing: None,
            section_bars: Vec::new(),
        }
    }

    /// Create a timer for a musical section with bars and tempo changes
    /// start_tempo_bpm and end_tempo_bpm should be in quarter notes per minute
    /// This method is kept for backward compatibility
    pub fn new_with_section(section_bars: Vec<FMBarElement>, start_tempo_bpm: f64, end_tempo_bpm: f64) -> Result<Self, String> {
        // Convert single section to tempo sequence
        let interval = FMTempoInterval::new(section_bars.clone(), start_tempo_bpm, Some(end_tempo_bpm), "Default Section".to_string());
        let sequence = FMTempoSequence::from_interval(interval);
        Self::new_with_tempo_sequence(sequence)
    }

    /// Create a timer for a tempo sequence with multiple intervals
    /// This is the new preferred method for complex musical arrangements
    pub fn new_with_tempo_sequence(tempo_sequence: FMTempoSequence) -> Result<Self, String> {
        let musical_timing = MusicalTiming::new_from_sequence(tempo_sequence.clone())
            .map_err(|e| format!("Failed to create musical timing from sequence: {}", e))?;
        
        // Use the shortest beat interval as the base timer interval
        // This ensures we don't miss any beat events
        let min_interval_ms = musical_timing.get_beat_events()
            .windows(2)
            .map(|pair| pair[1].time_offset_ms - pair[0].time_offset_ms)
            .fold(f64::INFINITY, f64::min);
        
        let interval_duration = Duration::from_millis(
            (min_interval_ms / 2.0).max(1.0) as u64  // Use half the minimum to ensure precision
        );

        let all_bars = tempo_sequence.all_bars();

        Ok(FMSectionTimer {
            _tick_callback: Arc::new(Mutex::new(None)),
            interval_duration,
            is_running: false,
            is_paused: false,
            timer_handle: None,
            stop_sender: None,
            pause_sender: None,
            resume_sender: None,
            musical_timing: Some(musical_timing),
            section_bars: all_bars,
        })
    }

    // Virtual method to set the tick callback (pointer to callback function)
    // #[flutter_rust_bridge::frb(ignore)]
    pub fn set_tick_callback<F>(&mut self, callback: F) -> Result<(), String>
    where
        F: Fn(BeatEvent) + Send + Sync + 'static,
    {
        match self._tick_callback.lock() {
            Ok(mut cb_guard) => {
                *cb_guard = Some(Box::new(callback));
                Ok(())
            }
            Err(e) => Err(format!("Failed to acquire callback lock: {}", e))
        }
    }

    /// Set interval using TimeDelta - FRB-compatible version
    // #[flutter_rust_bridge::frb(sync)]
    pub fn set_interval_duration(&mut self, duration: TimeDelta) -> Result<(), String> {
        // Convert TimeDelta to std::time::Duration
        let duration = match duration.to_std() {
            Ok(dur) => dur,
            Err(e) => return Err(format!("Invalid duration: {}", e)),
        };
        
        AsyncTimer::set_interval(self, duration);
        Ok(())
    }
    
    // Internal method to trigger the callback
    fn trigger_callback(&self, beat_event: BeatEvent) {
        // Use proper error handling instead of unwrap
        if let Ok(callback_guard) = self._tick_callback.lock() {
            if let Some(ref callback) = *callback_guard {
                callback(beat_event);
            }
        }
        // If lock fails, we silently continue - the callback simply won't be called
        // This prevents the timer thread from panicking
    }

    // Public method to manually trigger a callback (useful for testing or manual ticks)
    pub fn manual_tick(&self, beat_event: BeatEvent) {
        self.trigger_callback(beat_event);
    }

    // Debug method to get beat events information
    pub fn get_beat_events_debug(&self) -> Option<Vec<String>> {
        if let Some(ref timing) = self.musical_timing {
            let events = timing.get_beat_events();
            let debug_info: Vec<String> = events.iter().enumerate().map(|(i, event)| {
                format!("Event {}: Bar {} Beat {} Sub {} | {}/{} | Time: {:.1}ms | Tempo: {:.1} BPM | Type: {:?}", 
                    i + 1, event.bar_index + 1, event.beat_in_bar + 1, event.subbeat_in_beat + 1,
                    event.nom, event.denom, event.time_offset_ms, event.tempo_bpm, event.beat_type)
            }).collect();
            Some(debug_info)
        } else {
            None
        }
    }

    /// Get all beat events from the event queue as a vector
    pub fn get_all_beat_events(&self) -> Vec<BeatEvent> {
        if let Some(ref timing) = self.musical_timing {
            timing.get_beat_events().to_vec()
        } else {
            Vec::new()
        }
    }

    /// Get the total duration of the musical section in milliseconds
    pub fn get_total_duration_ms(&self) -> f64 {
        if let Some(ref timing) = self.musical_timing {
            timing.get_total_duration_ms()
        } else {
            0.0
        }
    }
}

// Implement the AsyncTimer trait for FMSectionTimer
impl AsyncTimer for FMSectionTimer {
    fn start(&mut self) -> Result<(), String> {
        if self.is_running {
            return Ok(());
        }

        let (stop_tx, stop_rx): (Sender<()>, Receiver<()>) = channel();
        let (pause_tx, pause_rx): (Sender<()>, Receiver<()>) = channel();
        let (resume_tx, resume_rx): (Sender<()>, Receiver<()>) = channel();
        
        self.stop_sender = Some(stop_tx);
        self.pause_sender = Some(pause_tx);
        self.resume_sender = Some(resume_tx);

        let callback = self._tick_callback.clone();
        let duration = self.interval_duration;

        // Check if we have musical timing for precise beat scheduling
        if let Some(ref musical_timing) = self.musical_timing {
            let beat_events = musical_timing.get_beat_events().to_vec();
            
            // Start the musical timer thread
            let handle = thread::spawn(move || {
                let start_time = std::time::Instant::now();
                let mut last_triggered_event = 0;
                let mut paused = false;
                let mut pause_start_time = std::time::Instant::now();
                let mut total_pause_duration = std::time::Duration::ZERO;

                loop {
                    // Check for stop signal (non-blocking)
                    if stop_rx.try_recv().is_ok() {
                        break;
                    }

                    // Check for pause signal
                    if pause_rx.try_recv().is_ok() {
                        paused = true;
                        pause_start_time = std::time::Instant::now();
                        log_debug("Timer thread received pause signal");
                    }

                    // Check for resume signal
                    if resume_rx.try_recv().is_ok() {
                        if paused {
                            total_pause_duration += pause_start_time.elapsed();
                            paused = false;
                            log_debug("Timer thread received resume signal");
                        }
                    }

                    // Skip beat processing if paused
                    if paused {
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }

                    // Calculate effective elapsed time (subtract pause durations)
                    let elapsed_ms = (start_time.elapsed() - total_pause_duration).as_millis() as f64;
                    
                    // Find next beat event(s) to trigger
                    for (event_index, event) in beat_events.iter().enumerate().skip(last_triggered_event) {
                        if event.time_offset_ms <= elapsed_ms {
                            // Trigger callback with complete BeatEvent
                            if let Ok(callback_guard) = callback.lock() {
                                if let Some(ref cb) = *callback_guard {
                                    cb(event.clone());
                                }
                            }
                            
                            last_triggered_event = event_index + 1;
                        } else {
                            break; // Events are in chronological order
                        }
                    }

                    // Check if we've processed all events
                    if last_triggered_event >= beat_events.len() {
                        break; // Section complete
                    }

                    thread::sleep(Duration::from_millis(1)); // Small sleep to prevent busy waiting
                }
            });
            
            self.timer_handle = Some(handle);
        } else {
            // Fall back to simple timer for non-musical use cases
            let handle = thread::spawn(move || {
                let mut tick_count = 0i32;
                let mut paused = false;

                loop {
                    // Check for stop signal (non-blocking)
                    if stop_rx.try_recv().is_ok() {
                        break;
                    }

                    // Check for pause signal
                    if pause_rx.try_recv().is_ok() {
                        paused = true;
                        log_debug("Simple timer thread received pause signal");
                    }

                    // Check for resume signal
                    if resume_rx.try_recv().is_ok() {
                        paused = false;
                        log_debug("Simple timer thread received resume signal");
                    }

                    // Skip tick if paused
                    if paused {
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }

                    thread::sleep(duration);
                    tick_count += 1;

                    // Create a default BeatEvent for simple timer
                    let beat_event = BeatEvent {
                        time_offset_ms: (tick_count as f64 - 1.0) * duration.as_millis() as f64,
                        bar_index: 0,
                        beat_in_bar: ((tick_count - 1) % 4) as usize,
                        subbeat_in_beat: 0,
                        tempo_bpm: 60.0, // Default tempo
                        nom: 4,
                        denom: 4,
                        beat_type: BeatType::Major,
                    };
                    
                    // Call callback
                    if let Ok(callback_guard) = callback.lock() {
                        if let Some(ref cb) = *callback_guard {
                            cb(beat_event);
                        }
                    }
                }
            });
            
            self.timer_handle = Some(handle);
        }

        self.is_running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Ok(());
        }

        // Send stop signal - handle potential send failure gracefully
        if let Some(ref stop_sender) = self.stop_sender {
            if let Err(e) = stop_sender.send(()) {
                // Thread might have already exited, which is fine
                log_warn(&format!("Failed to send stop signal: {}", e));
            }
        }

        // Wait for the timer thread to finish
        if let Some(handle) = self.timer_handle.take() {
            match handle.join() {
                Ok(_) => log_debug("Timer stopped successfully"),
                Err(e) => {
                    log_error(&format!("Timer thread panicked: {:?}", e));
                    return Err("Thread panic".into()); // I18n::t(TKey::ThreadPanic).into());
                }
            }
        }

        self.stop_sender = None;
        self.pause_sender = None;
        self.resume_sender = None;
        self.is_running = false;
        self.is_paused = false;
        Ok(())
    }

    fn set_interval(&mut self, duration: Duration) {
        self.interval_duration = duration;
    }

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn pause(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Err("Timer is not running".to_string());
        }
        
        if self.is_paused {
            return Ok(()); // Already paused
        }

        // Send pause signal if we have a pause sender
        if let Some(ref pause_sender) = self.pause_sender {
            if let Err(e) = pause_sender.send(()) {
                log_warn(&format!("Failed to send pause signal: {}", e));
            }
        }
        
        self.is_paused = true;
        log_debug("Timer paused");
        Ok(())
    }

    fn resume(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Err("Timer is not running".to_string());
        }
        
        if !self.is_paused {
            return Ok(()); // Already resumed
        }

        // Send resume signal if we have a resume sender
        if let Some(ref resume_sender) = self.resume_sender {
            if let Err(e) = resume_sender.send(()) {
                log_warn(&format!("Failed to send resume signal: {}", e));
            }
        }
        
        self.is_paused = false;
        log_debug("Timer resumed");
        Ok(())
    }

    fn is_paused(&self) -> bool {
        self.is_paused
    }
}



// FMTickerBase trait - equivalent to Python abstract base class
pub trait FMTickerBase {
    fn tick_callback(&mut self, beat_event: BeatEvent);
    
    // Static method equivalent from Python - implemented as default trait method
    fn fix_subbeat(tick_value: i32, ignore_subbeats: Option<bool>) -> i32 {
        if tick_value == FMTickPositions::TMedium as i32 && ignore_subbeats.unwrap_or(false) {
            FMTickPositions::TMinor as i32
        } else {
            tick_value
        }
    }
}

// Example concrete implementation
pub struct FMCircleTicker {
    pub timer: Option<FMSectionTimer>,
    pub state: FMTickPositions,
    #[allow(dead_code)]
    pub tick_positions: FMTickPositions,
}

impl FMCircleTicker {
    pub fn new() -> Self {
        Self {
            timer: None,
            state: FMTickPositions::TNone,
            tick_positions: FMTickPositions::TNone,
        }
    }

    pub fn connect_timer(&mut self, mut timer: FMSectionTimer) -> Result<(), String> {
        // For now, set up a simple callback that logs
        // In a more complex implementation, you'd use channels or other mechanisms
        // to communicate back to the ticker's tick_callback method
        let callback = |beat_event: BeatEvent| {
            // This demonstrates the callback mechanism working
            // In practice, this would trigger the ticker's tick_callback via some communication mechanism
            log_info(&format!("Timer tick: bar={}, beat={}/{}, beat_type={:?}, time={}ms", 
                     beat_event.bar_index + 1, beat_event.beat_in_bar + 1, beat_event.subbeat_in_beat + 1, 
                     beat_event.beat_type, beat_event.time_offset_ms));
        };
        
        timer.set_tick_callback(callback)?;
        self.timer = Some(timer);
        Ok(())
    }

    pub fn start_timer(&mut self) -> Result<(), String> {
        if let Some(ref mut timer) = self.timer {
            timer.start()?;
        }
        Ok(())
    }

    pub fn stop_timer(&mut self) -> Result<(), String> {
        if let Some(ref mut timer) = self.timer {
            timer.stop()?;
        }
        Ok(())
    }
}

impl FMTickerBase for FMCircleTicker {
    fn tick_callback(&mut self, beat_event: BeatEvent) {
        // Apply fix_subbeat logic if needed (using beat_in_bar instead of cnt)
        let adjusted_tick = Self::fix_subbeat(self.state as i32, None);
        
        // Update state based on tick
        self.state = match adjusted_tick {
            x if x == FMTickPositions::TSection as i32 => FMTickPositions::TSection,
            x if x == FMTickPositions::TRest as i32 => FMTickPositions::TRest,
            _ => FMTickPositions::TNone,
        };
        
        // Handle the tick logic here
        log_debug(&format!("FMCircleTicker tick_callback: bar={}, beat={}/{}, state={:?}, beat_type={:?}", 
                  beat_event.bar_index + 1, beat_event.beat_in_bar + 1, beat_event.subbeat_in_beat + 1,
                  self.state, beat_event.beat_type));
    }
}

// Flutter-integrated ticker that sends beat events to Flutter UI
// Temporarily disabled while focusing on Rust core functionality
/*
/// Flutter-integrated ticker that sends beat events to Flutter UI
// #[flutter_rust_bridge::frb(opaque)]
pub struct FlutterTicker {
    timer: Option<FMSectionTimer>,
    state: FMTickPositions,
    section_bars: Vec<FMBarElement>,
    beat_config: std::collections::HashMap<crate::api::fm_base::MetricKey, Vec<i32>>,
    start_time: Option<std::time::Instant>,
    last_beat_time: u64,
}

impl FlutterTicker {
    pub fn new() -> Result<Self, String> {
        let beat_config = match crate::api::fm_base::load_beat_config() {
            Ok(config) => config,
            Err(e) => {
                log_error(&format!("Failed to load beat config: {}", e));
                // Return error instead of using empty HashMap
                return Err(format!("Beat configuration loading failed: {}", e));
            }
        };

        Ok(Self {
            timer: None,
            state: FMTickPositions::TNone,
            section_bars: Vec::new(),
            beat_config,
            start_time: None,
            last_beat_time: 0,
        })
    }

    pub fn create_section(
        &mut self,
        section_bars: Vec<FMBarElement>,
        start_tempo_bpm: f64,
        end_tempo_bpm: f64,
    ) -> Result<(), String> {
        let timer = match FMSectionTimer::new_with_section(section_bars.clone(), start_tempo_bpm, end_tempo_bpm) {
            Ok(mut timer) => {
                // Set up simple callback for FlutterTicker
                if let Err(e) = timer.set_tick_callback(move |_beat_event| {
                    // Beat events are handled by the individual timer's callback mechanism
                    // This FlutterTicker is just a container
                }) {
                    return Err(format!("Failed to set tick callback: {}", e));
                }
                timer
            }
            Err(e) => return Err(format!("Failed to create timer: {}", e)),
        };

        self.timer = Some(timer);
        self.section_bars = section_bars;
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), String> {
        self.start_time = Some(std::time::Instant::now());
        self.last_beat_time = 0;
        
        if let Some(ref mut timer) = self.timer {
            timer.start().map_err(|e| format!("Failed to start timer: {}", e))
        } else {
            Err("No section configured".to_string())
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(ref mut timer) = self.timer {
            timer.stop().map_err(|e| format!("Failed to stop timer: {}", e))
        } else {
            Ok(())
        }
    }
}

impl FMTickerBase for FlutterTicker {
    fn tick_callback(&mut self, beat_event: BeatEvent) {
        // Apply fix_subbeat logic if needed (using beat_in_bar instead of cnt)
        let adjusted_tick = Self::fix_subbeat(self.state as i32, None);
        
        // Update state based on tick
        self.state = match adjusted_tick {
            x if x == FMTickPositions::TSection as i32 => FMTickPositions::TSection,
            x if x == FMTickPositions::TRest as i32 => FMTickPositions::TRest,
            _ => FMTickPositions::TNone,
        };
        
        log_debug(&format!("FlutterTicker tick_callback: bar={}, beat={}/{}, state={:?}, beat_type={:?}", 
                  beat_event.bar_index + 1, beat_event.beat_in_bar + 1, beat_event.subbeat_in_beat + 1,
                  self.state, beat_event.beat_type));
    }
}
*/

/*
## **Removed `.unwrap()` Calls and Improved Error Handling:**

### **1. `set_tick_callback()` Method:**
- **Before**: Used `.unwrap()` which could panic if mutex was poisoned
- **After**: Returns `Result<(), Box<dyn std::error::Error>>` with proper error message
- **Benefit**: Caller can handle lock failures gracefully

### **2. Timer Thread Callback Execution:**
- **Before**: Used `.unwrap()` for callback lock
- **After**: Uses `if let Ok()` pattern - silently skips tick if lock fails
- **Benefit**: Timer thread continues running even if callback lock is temporarily unavailable

### **3. `stop()` Method Improvements:**
- **Before**: Used `let _ = stop_sender.send(())` (ignoring errors)
- **After**: Proper error handling with logging for send failures
- **Benefit**: Can detect if thread already exited (which is fine)

### **4. Thread Join Handling:**
- **Before**: Used `let _ = handle.join()` (ignoring panics)
- **After**: Handles thread panics explicitly and returns error
- **Benefit**: Caller knows if timer thread crashed

### **5. `connect_timer()` Method:**
- **Before**: No error handling for callback setup
- **After**: Returns `Result` and propagates errors using `?` operator
- **Benefit**: Initialization failures are properly reported

## **Why This is Better:**

1. **No Panics**: The application won't crash if there are temporary lock contention issues
2. **Graceful Degradation**: If callback lock fails during a tick, we just skip that tick
3. **Proper Error Reporting**: Callers get meaningful error messages instead of panics
4. **Robust Shutdown**: Timer shutdown handles edge cases like thread panics
5. **Idiomatic Rust**: Uses `Result` types and the `?` operator for error propagation

## **Error Handling Strategy:**

- **Fatal Errors**: Return `Result::Err` to caller (e.g., mutex poisoning during setup)
- **Recoverable Errors**: Log and continue (e.g., temporary lock contention during tick)
- **Expected Conditions**: Handle gracefully (e.g., thread already stopped)

This approach follows Rust best practices for error handling and makes the timer much more robust in production scenarios!
*/
