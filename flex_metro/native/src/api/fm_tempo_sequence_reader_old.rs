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
        for (index, yaml_interval) in yaml_sequence.intervals.iter().enumerate() {
            let interval = Self::convert_interval(yaml_interval.clone(), 0, 0, 0)?;
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
    }

    /// Convert a single interval from YAML data
    fn convert_interval(
        yaml_interval: YamlIntervalData, 
        repetition_id: usize, 
        position_in_repetition: usize, 
        total_intervals_in_repetition: usize
    ) -> Result<FMTempoInterval, String> {
        let mut bars = Vec::new();

        for yaml_bar in yaml_interval.bars {
            let bar = Self::convert_bar_element(yaml_bar)?;
            bars.push(bar);
        }

        let mut interval = FMTempoInterval::new(
            bars,
            yaml_interval.start_tempo_bpm,
            yaml_interval.end_tempo_bpm,
            yaml_interval.name,
        );

        // Set repetition info
        interval.repetition_id = repetition_id;
        interval.position_in_repetition = position_in_repetition;
        interval.total_intervals_in_repetition = total_intervals_in_repetition;

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
        let mut yaml_items = Vec::new();
        let mut i = 0;

        while i < sequence.intervals.len() {
            let interval = &sequence.intervals[i];

            if interval.repetition_id == 0 {
                // Single interval not in repetition
                let yaml_interval_data = Self::convert_interval_to_yaml_data(interval)?;
                yaml_items.push(YamlSequenceItem::Interval(YamlTempoInterval {
                    interval: yaml_interval_data,
                }));
                i += 1;
            } else {
                // Start of a repetition group - we need to collect all intervals in this repetition
                // and determine how many times it was repeated
                let repetition_id = interval.repetition_id;
                let intervals_in_repetition = interval.total_intervals_in_repetition;
                
                // Collect the original intervals (first occurrence)
                let mut original_intervals = Vec::new();
                for j in 0..intervals_in_repetition {
                    if i + j < sequence.intervals.len() {
                        let rep_interval = &sequence.intervals[i + j];
                        let yaml_interval_data = Self::convert_interval_to_yaml_data(rep_interval)?;
                        original_intervals.push(yaml_interval_data);
                    }
                }

                // Count how many times this group was repeated by looking ahead
                let mut num_reps = 1;
                let mut next_start = i + intervals_in_repetition;
                
                while next_start < sequence.intervals.len() {
                    let next_interval = &sequence.intervals[next_start];
                    if next_interval.repetition_id == repetition_id {
                        num_reps += 1;
                        next_start += intervals_in_repetition;
                    } else {
                        break;
                    }
                }

                // Create repetition YAML structure
                let repetition_data = YamlRepetitionData {
                    num_reps,
                    intervals: original_intervals,
                };

                yaml_items.push(YamlSequenceItem::Repetition(YamlTempoRepetition {
                    repetition: repetition_data,
                }));

                // Skip all the repeated intervals
                i += intervals_in_repetition * num_reps;
            }
        }

        Ok(YamlTempoSequence {
            sequence: yaml_items,
        })
    }

    /// Convert a single interval to YAML data
    fn convert_interval_to_yaml_data(interval: &FMTempoInterval) -> Result<YamlIntervalData, String> {
        let mut yaml_bars = Vec::new();

        for bar in &interval.bars {
            let yaml_bar = Self::convert_bar_element_to_yaml(bar)?;
            yaml_bars.push(yaml_bar);
        }

        let end_tempo = if interval.is_constant_tempo() {
            None // Don't include end_tempo if it's the same as start_tempo
        } else {
            Some(interval.end_tempo_bpm)
        };

        Ok(YamlIntervalData {
            name: interval.name.clone(),
            start_tempo_bpm: interval.start_tempo_bpm,
            end_tempo_bpm: end_tempo,
            bars: yaml_bars,
        })
    }

    /// Convert a bar element to YAML
    fn convert_bar_element_to_yaml(bar: &FMBarElement) -> Result<YamlBarElement, String> {
        let beats = if bar.has_signature {
            match &bar.beats {
                crate::api::fm_bar_element::BarBeatData::Int(beats_vec) => {
                    // Use the bar element's method to check if it's using defaults
                    if bar.is_using_default_beats() {
                        None // Skip serializing if same as default
                    } else {
                        Some(beats_vec.clone())
                    }
                },
                crate::api::fm_bar_element::BarBeatData::Float(_) => None, // Don't serialize float beats
            }
        } else {
            None
        };

        Ok(YamlBarElement {
            nom: bar.nom,
            denom: bar.denom,
            nom_secs: bar.nom_secs,
            beats,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sequence_with_repetition() {
        let yaml_content = r#"
Sequence:
  - Interval:
      name: "Solo Start"
      start_tempo_bpm: 90
      bars:
        - nom: 4
          denom: 4
          nom_secs: 0.0
  - Repetition:
      NumReps: 2
      - Interval:
          name: "Repeated"
          start_tempo_bpm: 120
          bars:
            - nom: 4
              denom: 4
              nom_secs: 0.0
"#;

        let result = TempoSequenceReader::read_from_string(yaml_content);
        assert!(result.is_ok(), "Failed to parse YAML: {:?}", result.err());
        
        let sequence = result.unwrap();
        assert_eq!(sequence.intervals.len(), 3); // 1 solo + 2 repetitions

        // Check solo interval
        assert_eq!(sequence.intervals[0].name, "Solo Start");
        assert_eq!(sequence.intervals[0].repetition_id, 0);

        // Check repeated intervals
        assert_eq!(sequence.intervals[1].name, "Repeated");
        assert_eq!(sequence.intervals[1].repetition_id, 1);
        assert_eq!(sequence.intervals[1].position_in_repetition, 0);
        assert_eq!(sequence.intervals[1].total_intervals_in_repetition, 1);

        assert_eq!(sequence.intervals[2].name, "Repeated");
        assert_eq!(sequence.intervals[2].repetition_id, 1);
    }
}
