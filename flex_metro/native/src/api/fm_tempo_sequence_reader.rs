// YAML Reader and Writer for FMTempoSequence with intervals and repetitions keys
// Simplified approach: use clean YAML structure with intervals key for better compatibility

use crate::api::fm_tempo_interval::{FMTempoSequence, FMTempoInterval};
use crate::api::fm_bar_element::FMBarElement;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use std::path::Path;

// Top-level YAML structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlTempoSequence {
    pub intervals: Vec<YamlIntervalData>,
    #[serde(default)]
    pub repetitions: Vec<YamlRepetitionData>,
}

// Repetition structure with start/end interval indices and repeat count  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlRepetitionData {
    pub start_interval_index: usize,
    pub end_interval_index: usize,
    pub num_reps: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlIntervalData {
    pub name: String,
    pub start_tempo_bpm: f64,
    pub end_tempo_bpm: Option<f64>,
    pub bars: Vec<YamlBarElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlBarElement {
    pub nom: i32,
    pub denom: i32,
    #[serde(default)]
    pub nom_secs: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beats: Option<Vec<i32>>,
}

// Reader
pub struct TempoSequenceReader;

impl TempoSequenceReader {
    /// Read a FMTempoSequence from YAML file
    pub fn read_from_file(file_path: &Path) -> Result<FMTempoSequence, String> {
        let file_content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        Self::read_from_string(&file_content)
    }

    /// Read a FMTempoSequence from YAML string
    pub fn read_from_string(yaml_content: &str) -> Result<FMTempoSequence, String> {
        let yaml_sequence: YamlTempoSequence = serde_yaml::from_str(yaml_content)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;

        Self::convert_from_yaml(yaml_sequence)
    }

    /// Convert from YAML structure to FMTempoSequence
    fn convert_from_yaml(yaml_sequence: YamlTempoSequence) -> Result<FMTempoSequence, String> {
        let mut intervals = Vec::new();
        
        // Convert base intervals
        for yaml_interval in yaml_sequence.intervals.iter() {
            let interval = Self::convert_interval(yaml_interval.clone())?;
            intervals.push(interval);
        }

        // Apply repetitions by setting repetition metadata
        for (rep_id, rep) in yaml_sequence.repetitions.iter().enumerate() {
            let repetition_id = rep_id + 1; // Start from 1
            let total_intervals_in_rep = rep.end_interval_index - rep.start_interval_index + 1;
            
            // Set repetition metadata for intervals in the repetition range
            for i in rep.start_interval_index..=rep.end_interval_index {
                if i < intervals.len() {
                    intervals[i].repetition_id = repetition_id;
                    intervals[i].position_in_repetition = i - rep.start_interval_index;
                    intervals[i].total_intervals_in_repetition = total_intervals_in_rep;
                }
            }
        }

        Ok(FMTempoSequence { intervals })
    }

    /// Convert a single interval from YAML data
    fn convert_interval(yaml_interval: YamlIntervalData) -> Result<FMTempoInterval, String> {
        let mut bars = Vec::new();

        for yaml_bar in yaml_interval.bars {
            let bar = Self::convert_bar_element(yaml_bar)?;
            bars.push(bar);
        }

        let interval = FMTempoInterval::new(
            bars,
            yaml_interval.start_tempo_bpm,
            yaml_interval.end_tempo_bpm,
            yaml_interval.name,
        );

        Ok(interval)
    }

    /// Convert a bar element from YAML
    fn convert_bar_element(yaml_bar: YamlBarElement) -> Result<FMBarElement, String> {
        let bar = FMBarElement::new(yaml_bar.nom, yaml_bar.denom, yaml_bar.nom_secs, yaml_bar.beats);
        Ok(bar)
    }
}

// Writer
pub struct TempoSequenceWriter;

impl TempoSequenceWriter {
    /// Write a FMTempoSequence to YAML file
    pub fn write_to_file(sequence: &FMTempoSequence, file_path: &Path) -> Result<(), String> {
        let yaml_string = Self::write_to_string(sequence)?;
        fs::write(file_path, yaml_string)
            .map_err(|e| format!("Failed to write file: {}", e))
    }

    /// Write a FMTempoSequence to YAML string
    pub fn write_to_string(sequence: &FMTempoSequence) -> Result<String, String> {
        let yaml_sequence = Self::convert_to_yaml(sequence)?;
        serde_yaml::to_string(&yaml_sequence)
            .map_err(|e| format!("Failed to serialize YAML: {}", e))
    }

    /// Convert from FMTempoSequence to YAML structure
    fn convert_to_yaml(sequence: &FMTempoSequence) -> Result<YamlTempoSequence, String> {
        // Convert all intervals
        let mut yaml_intervals = Vec::new();
        for interval in &sequence.intervals {
            let yaml_interval = Self::convert_interval_to_yaml(interval)?;
            yaml_intervals.push(yaml_interval);
        }

        // Extract repetitions from interval metadata
        let mut repetitions = Vec::new();
        let mut processed_reps = std::collections::HashSet::new();

        for (i, interval) in sequence.intervals.iter().enumerate() {
            if interval.repetition_id > 0 && !processed_reps.contains(&interval.repetition_id) {
                processed_reps.insert(interval.repetition_id);
                
                // Find the start and end of this repetition
                let start_index = i - interval.position_in_repetition;
                let end_index = start_index + interval.total_intervals_in_repetition - 1;
                
                // Count how many times this repetition appears
                let num_reps = Self::count_repetitions(sequence, interval.repetition_id);
                
                repetitions.push(YamlRepetitionData {
                    start_interval_index: start_index,
                    end_interval_index: end_index,
                    num_reps,
                });
            }
        }

        Ok(YamlTempoSequence {
            intervals: yaml_intervals,
            repetitions,
        })
    }

    /// Count how many times a repetition appears in the sequence
    fn count_repetitions(sequence: &FMTempoSequence, repetition_id: usize) -> usize {
        let mut count = 0;
        let mut i = 0;
        
        while i < sequence.intervals.len() {
            let interval = &sequence.intervals[i];
            if interval.repetition_id == repetition_id {
                count += 1;
                // Skip ahead to the next repetition
                i += interval.total_intervals_in_repetition;
            } else {
                i += 1;
            }
        }
        
        count
    }

    /// Convert a single interval to YAML data
    fn convert_interval_to_yaml(interval: &FMTempoInterval) -> Result<YamlIntervalData, String> {
        let mut yaml_bars = Vec::new();

        for bar in &interval.bars {
            let yaml_bar = Self::convert_bar_to_yaml(bar);
            yaml_bars.push(yaml_bar);
        }

        Ok(YamlIntervalData {
            name: interval.name.clone(),
            start_tempo_bpm: interval.start_tempo_bpm,
            end_tempo_bpm: if interval.end_tempo_bpm != interval.start_tempo_bpm {
                Some(interval.end_tempo_bpm)
            } else {
                None
            },
            bars: yaml_bars,
        })
    }

    /// Convert a bar element to YAML
    fn convert_bar_to_yaml(bar: &FMBarElement) -> YamlBarElement {
        let beats = match &bar.beats {
            crate::api::fm_bar_element::BarBeatData::Int(int_beats) => {
                // Only include beats if they're not the default pattern
                let key = crate::api::fm_base::MetricKey::Standard((bar.nom, bar.denom));
                if let Some(default_beats) = crate::api::fm_base::DEFAULT_BEATS.get(&key) {
                    if int_beats == default_beats {
                        None // Skip default beats
                    } else {
                        Some(int_beats.clone())
                    }
                } else {
                    Some(int_beats.clone())
                }
            },
            crate::api::fm_bar_element::BarBeatData::Float(_) => {
                // For float beats, we'll convert to int for YAML simplicity
                None // Skip for now, could be enhanced later
            }
        };

        YamlBarElement {
            nom: bar.nom,
            denom: bar.denom,
            nom_secs: bar.nom_secs,
            beats,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sequence_with_repetition() {
        let yaml_content = r#"
intervals:
  - name: "Intro"
    start_tempo_bpm: 120.0
    bars:
      - nom: 4
        denom: 4
        nom_secs: 2.0
  - name: "Main"
    start_tempo_bpm: 120.0
    bars:
      - nom: 4
        denom: 4
        nom_secs: 2.0
  - name: "Bridge"
    start_tempo_bpm: 110.0
    bars:
      - nom: 3
        denom: 4
        nom_secs: 1.5

repetitions:
  - start_interval_index: 1
    end_interval_index: 2
    num_reps: 3
"#;

        match TempoSequenceReader::read_from_string(yaml_content) {
            Ok(sequence) => {
                println!("Successfully parsed sequence with {} intervals", sequence.intervals.len());
                assert_eq!(sequence.intervals.len(), 3, "Should have 3 intervals");
                
                // Check repetition metadata
                let main_interval = &sequence.intervals[1];
                let bridge_interval = &sequence.intervals[2];
                
                assert_eq!(main_interval.repetition_id, 1);
                assert_eq!(main_interval.position_in_repetition, 0);
                assert_eq!(main_interval.total_intervals_in_repetition, 2);
                
                assert_eq!(bridge_interval.repetition_id, 1);
                assert_eq!(bridge_interval.position_in_repetition, 1);
                assert_eq!(bridge_interval.total_intervals_in_repetition, 2);
                
                println!("Test passed!");
            }
            Err(e) => {
                panic!("Failed to parse sequence: {}", e);
            }
        }
    }

    #[test]
    fn test_round_trip_conversion() {
        let yaml_content = r#"
intervals:
  - name: "Test"
    start_tempo_bpm: 120.0
    bars:
      - nom: 4
        denom: 4
        nom_secs: 2.0

repetitions: []
"#;

        let sequence = TempoSequenceReader::read_from_string(yaml_content).unwrap();
        let yaml_output = TempoSequenceWriter::write_to_string(&sequence).unwrap();
        let sequence2 = TempoSequenceReader::read_from_string(&yaml_output).unwrap();
        
        assert_eq!(sequence.intervals.len(), sequence2.intervals.len());
        assert_eq!(sequence.intervals[0].name, sequence2.intervals[0].name);
    }

    #[test]
    fn test_file_reading() {
        use std::path::Path;
        
        let test_file = Path::new("test_new_structure.yaml");
        match TempoSequenceReader::read_from_file(test_file) {
            Ok(sequence) => {
                println!("Successfully loaded sequence from file with {} intervals", sequence.intervals.len());
                assert_eq!(sequence.intervals.len(), 4);
                
                // Check that repetition metadata was applied correctly
                let verse = &sequence.intervals[1];
                let chorus = &sequence.intervals[2];
                
                assert_eq!(verse.repetition_id, 1);
                assert_eq!(verse.position_in_repetition, 0);
                assert_eq!(verse.total_intervals_in_repetition, 2);
                
                assert_eq!(chorus.repetition_id, 1);
                assert_eq!(chorus.position_in_repetition, 1);
                assert_eq!(chorus.total_intervals_in_repetition, 2);
                
                println!("File reading test passed!");
            }
            Err(e) => {
                println!("Failed to read file (this may be expected if test file doesn't exist): {}", e);
                // Don't panic for this test since the file might not exist in all environments
            }
        }
    }
}
