// FMTempoInterval - Represents a sequence of bars with specific tempo characteristics
// This replaces the single FMSection concept with multiple intervals

use crate::api::fm_bar_element::FMBarElement;

/// Represents a tempo interval containing a sequence of bars with tempo progression
#[derive(Debug, Clone)]
pub struct FMTempoInterval {
    /// List of bars in this interval
    pub bars: Vec<FMBarElement>,
    /// Starting tempo in BPM for this interval
    pub start_tempo_bpm: f64,
    /// Ending tempo in BPM for this interval (can be same as start for constant tempo)
    pub end_tempo_bpm: f64,
    /// Name/label for this interval (obligatory)
    pub name: String,
    /// Repetition ID (0 if not in repetition, otherwise incremental counter starting from 1)
    pub repetition_id: usize,
    /// Position within current repetition (0 if not in repetition, otherwise 0-based position)
    pub position_in_repetition: usize,
    /// Total number of intervals in the current repetition (0 if not in repetition)
    pub total_intervals_in_repetition: usize,
}

impl FMTempoInterval {
    /// Create a new tempo interval
    /// If end_tempo_bpm is None, it defaults to start_tempo_bpm (constant tempo)
    pub fn new(bars: Vec<FMBarElement>, start_tempo_bpm: f64, end_tempo_bpm: Option<f64>, name: String) -> Self {
        let end_tempo = end_tempo_bpm.unwrap_or(start_tempo_bpm);
        Self {
            bars,
            start_tempo_bpm,
            end_tempo_bpm: end_tempo,
            name,
            repetition_id: 0,                        // Default: not in repetition
            position_in_repetition: 0,               // Default: not in repetition
            total_intervals_in_repetition: 0,        // Default: not in repetition
        }
    }

    /// Create a new tempo interval as part of a repetition
    pub fn new_with_repetition(
        bars: Vec<FMBarElement>, 
        start_tempo_bpm: f64, 
        end_tempo_bpm: Option<f64>, 
        name: String,
        repetition_id: usize,
        position_in_repetition: usize,
        total_intervals_in_repetition: usize
    ) -> Self {
        let end_tempo = end_tempo_bpm.unwrap_or(start_tempo_bpm);
        Self {
            bars,
            start_tempo_bpm,
            end_tempo_bpm: end_tempo,
            name,
            repetition_id,
            position_in_repetition,
            total_intervals_in_repetition,
        }
    }

    /// Check if this interval is part of a repetition
    pub fn is_in_repetition(&self) -> bool {
        self.repetition_id > 0
    }
    
    /// Get the total number of bars in this interval
    pub fn bar_count(&self) -> usize {
        self.bars.len()
    }
    
    /// Check if this interval has constant tempo (no tempo change)
    pub fn is_constant_tempo(&self) -> bool {
        (self.start_tempo_bpm - self.end_tempo_bpm).abs() < 0.01
    }
    
    /// Get tempo change percentage for this interval
    pub fn tempo_change_percent(&self) -> f64 {
        if self.start_tempo_bpm == 0.0 {
            0.0
        } else {
            ((self.end_tempo_bpm - self.start_tempo_bpm) / self.start_tempo_bpm) * 100.0
        }
    }
    
    /// Get a human-readable description of this interval
    pub fn description(&self) -> String {
        let tempo_desc = if self.is_constant_tempo() {
            format!("{:.1} BPM", self.start_tempo_bpm)
        } else {
            format!("{:.1} → {:.1} BPM", self.start_tempo_bpm, self.end_tempo_bpm)
        };
        
        format!("{}: {} bars, {}", self.name, self.bar_count(), tempo_desc)
    }
}

/// A sequence of tempo intervals that forms a complete musical section
#[derive(Debug, Clone)]
pub struct FMTempoSequence {
    /// List of tempo intervals in order
    pub intervals: Vec<FMTempoInterval>,
}

impl FMTempoSequence {
    /// Create a new empty tempo sequence
    pub fn new() -> Self {
        Self {
            intervals: Vec::new(),
        }
    }
    
    /// Create a tempo sequence from a single interval
    pub fn from_interval(interval: FMTempoInterval) -> Self {
        Self {
            intervals: vec![interval],
        }
    }
    
    /// Add a tempo interval to the sequence
    pub fn add_interval(&mut self, interval: FMTempoInterval) {
        self.intervals.push(interval);
    }
    
    /// Create a FMSectionTimer from this tempo sequence
    pub fn new_with_tempo_sequence(sequence: FMTempoSequence) -> Result<crate::api::fm_ticker_base::FMSectionTimer, String> {
        crate::api::fm_ticker_base::FMSectionTimer::new_with_tempo_sequence(sequence)
    }
    
    /// Get the total number of bars across all intervals
    pub fn total_bars(&self) -> usize {
        self.intervals.iter().map(|i| i.bar_count()).sum()
    }
    
    /// Get the total number of intervals
    pub fn interval_count(&self) -> usize {
        self.intervals.len()
    }
    
    /// Check if the sequence is empty
    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }
    
    /// Get all bars from all intervals in order
    pub fn all_bars(&self) -> Vec<FMBarElement> {
        self.intervals.iter()
            .flat_map(|interval| interval.bars.iter())
            .cloned()
            .collect()
    }
    
    /// Get a human-readable description of the entire sequence
    pub fn description(&self) -> String {
        if self.intervals.is_empty() {
            return "Empty sequence".to_string();
        }
        
        let mut parts = Vec::new();
        for (i, interval) in self.intervals.iter().enumerate() {
            parts.push(format!("Interval {}: {}", i + 1, interval.description()));
        }
        
        format!("Tempo Sequence ({} intervals, {} bars total):\n{}", 
                self.interval_count(), 
                self.total_bars(),
                parts.join("\n"))
    }
}

impl Default for FMTempoSequence {
    fn default() -> Self {
        Self::new()
    }
}
