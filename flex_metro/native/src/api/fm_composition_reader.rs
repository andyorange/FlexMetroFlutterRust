// YAML Reader for Musical Composition Structure
// Reads YAML files into Composition structures
// Converts between 1-based YAML indexing and 0-based internal representation

use crate::api::fm_composition::{Composition, Movement, Section, Bar, Repetition};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use std::path::Path;

// YAML-specific structures that use 1-based indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct YamlComposition {
    pub composer_name: String,
    pub work_name: String,
    pub movements: Vec<YamlMovement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct YamlMovement {
    pub name: String,
    /// Optional tempo marking for this movement (can be overridden by sections)
    #[serde(default)]
    pub tempo_bpm: Option<f64>,
    pub sections: Vec<YamlSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct YamlSection {
    pub name: String,
    /// Optional tempo marking for this section (overrides movement tempo if present)
    #[serde(default)]
    pub tempo_bpm: Option<f64>,
    pub bars: Vec<Bar>, // Bars don't need conversion, they keep their natural numbering
    #[serde(default)]
    pub repetitions: Vec<YamlRepetition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct YamlRepetition {
    /// Starting section number (1-based) for repetition
    pub start_section: usize,
    /// Ending section number (1-based) for repetition
    pub end_section: usize,
    /// Total number of repetitions to perform
    pub num_repetitions: usize,
    /// Individual ending pattern: either all repetitions have identical-length endings or none do
    #[serde(default)]
    pub individual_ending_pattern: Option<Vec<Bar>>,
}

impl From<YamlComposition> for Composition {
    fn from(yaml: YamlComposition) -> Self {
        Self {
            composer_name: yaml.composer_name,
            work_name: yaml.work_name,
            movements: yaml.movements.into_iter().map(|m| m.into()).collect(),
        }
    }
}

impl From<Composition> for YamlComposition {
    fn from(comp: Composition) -> Self {
        Self {
            composer_name: comp.composer_name,
            work_name: comp.work_name,
            movements: comp.movements.into_iter().map(|m| m.into()).collect(),
        }
    }
}

impl From<YamlMovement> for Movement {
    fn from(yaml: YamlMovement) -> Self {
        Self {
            name: yaml.name,
            tempo_bpm: yaml.tempo_bpm,
            sections: yaml.sections.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl From<Movement> for YamlMovement {
    fn from(movement: Movement) -> Self {
        Self {
            name: movement.name,
            tempo_bpm: movement.tempo_bpm,
            sections: movement.sections.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl From<YamlSection> for Section {
    fn from(yaml: YamlSection) -> Self {
        Self {
            name: yaml.name,
            tempo_bpm: yaml.tempo_bpm,
            bars: yaml.bars,
            repetitions: yaml.repetitions.into_iter().map(|r| r.into()).collect(),
        }
    }
}

impl From<Section> for YamlSection {
    fn from(section: Section) -> Self {
        Self {
            name: section.name,
            tempo_bpm: section.tempo_bpm,
            bars: section.bars,
            repetitions: section.repetitions.into_iter().map(|r| r.into()).collect(),
        }
    }
}

impl From<YamlRepetition> for Repetition {
    fn from(yaml: YamlRepetition) -> Self {
        Self {
            // Convert from 1-based to 0-based indexing
            start_section: yaml.start_section.saturating_sub(1),
            end_section: yaml.end_section.saturating_sub(1),
            num_repetitions: yaml.num_repetitions,
            individual_ending_pattern: yaml.individual_ending_pattern,
        }
    }
}

impl From<Repetition> for YamlRepetition {
    fn from(repetition: Repetition) -> Self {
        Self {
            // Convert from 0-based to 1-based indexing
            start_section: repetition.start_section + 1,
            end_section: repetition.end_section + 1,
            num_repetitions: repetition.num_repetitions,
            individual_ending_pattern: repetition.individual_ending_pattern,
        }
    }
}

#[derive(Debug)]
pub enum CompositionReaderError {
    FileNotFound(String),
    InvalidYaml(String),
    ValidationError(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for CompositionReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompositionReaderError::FileNotFound(path) => write!(f, "File not found: {}", path),
            CompositionReaderError::InvalidYaml(msg) => write!(f, "Invalid YAML: {}", msg),
            CompositionReaderError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            CompositionReaderError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for CompositionReaderError {}

impl From<std::io::Error> for CompositionReaderError {
    fn from(err: std::io::Error) -> Self {
        CompositionReaderError::IoError(err)
    }
}

impl From<serde_yaml::Error> for CompositionReaderError {
    fn from(err: serde_yaml::Error) -> Self {
        CompositionReaderError::InvalidYaml(err.to_string())
    }
}

pub struct CompositionReader;

impl CompositionReader {
    /// Read a composition from a YAML file
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Composition, CompositionReaderError> {
        let path_ref = path.as_ref();
        
        if !path_ref.exists() {
            return Err(CompositionReaderError::FileNotFound(
                path_ref.to_string_lossy().to_string()
            ));
        }

        let content = fs::read_to_string(path_ref)?;
        Self::read_from_string(&content)
    }

    /// Read a composition from a YAML string
    pub fn read_from_string(yaml_content: &str) -> Result<Composition, CompositionReaderError> {
        let yaml_composition: YamlComposition = serde_yaml::from_str(yaml_content)?;
        let composition = Composition::from(yaml_composition);
        Self::validate_composition(&composition)?;
        Ok(composition)
    }

    /// Validate the composition structure
    fn validate_composition(composition: &Composition) -> Result<(), CompositionReaderError> {
        if composition.composer_name.is_empty() {
            return Err(CompositionReaderError::ValidationError(
                "Composer name cannot be empty".to_string()
            ));
        }

        if composition.work_name.is_empty() {
            return Err(CompositionReaderError::ValidationError(
                "Work name cannot be empty".to_string()
            ));
        }

        if composition.movements.is_empty() {
            return Err(CompositionReaderError::ValidationError(
                "Composition must have at least one movement".to_string()
            ));
        }

        for (movement_idx, movement) in composition.movements.iter().enumerate() {
            Self::validate_movement(movement, movement_idx)?;
        }

        Ok(())
    }

    fn validate_movement(movement: &Movement, movement_idx: usize) -> Result<(), CompositionReaderError> {
        if movement.name.is_empty() {
            return Err(CompositionReaderError::ValidationError(
                format!("Movement {} name cannot be empty", movement_idx + 1)
            ));
        }

        if let Some(tempo) = movement.tempo_bpm {
            if tempo <= 0.0 {
                return Err(CompositionReaderError::ValidationError(
                    format!("Movement '{}' tempo must be positive", movement.name)
                ));
            }
        }

        if movement.sections.is_empty() {
            return Err(CompositionReaderError::ValidationError(
                format!("Movement '{}' must have at least one section", movement.name)
            ));
        }

        for (section_idx, section) in movement.sections.iter().enumerate() {
            Self::validate_section(section, section_idx, movement.sections.len())?;
        }

        Ok(())
    }

    fn validate_section(section: &Section, section_idx: usize, total_sections: usize) -> Result<(), CompositionReaderError> {
        if section.name.is_empty() {
            return Err(CompositionReaderError::ValidationError(
                format!("Section {} name cannot be empty", section_idx + 1)
            ));
        }

        if let Some(tempo) = section.tempo_bpm {
            if tempo <= 0.0 {
                return Err(CompositionReaderError::ValidationError(
                    format!("Section '{}' tempo must be positive", section.name)
                ));
            }
        }

        if section.bars.is_empty() {
            return Err(CompositionReaderError::ValidationError(
                format!("Section '{}' must have at least one bar", section.name)
            ));
        }

        // Validate bar numbering
        for (bar_idx, bar) in section.bars.iter().enumerate() {
            Self::validate_bar(bar, bar_idx)?;
        }

        // Validate repetitions
        for (rep_idx, repetition) in section.repetitions.iter().enumerate() {
            Self::validate_repetition(repetition, rep_idx, total_sections)?;
        }

        // Check for overlapping repetitions
        Self::check_repetition_overlaps(&section.repetitions)?;

        Ok(())
    }

    fn validate_bar(bar: &Bar, bar_idx: usize) -> Result<(), CompositionReaderError> {
        if bar.numerator == 0 {
            return Err(CompositionReaderError::ValidationError(
                format!("Bar {} numerator cannot be zero", bar_idx + 1)
            ));
        }

        if bar.denominator == 0 {
            return Err(CompositionReaderError::ValidationError(
                format!("Bar {} denominator cannot be zero", bar_idx + 1)
            ));
        }

        // Validate denominator is a power of 2
        if !bar.denominator.is_power_of_two() {
            return Err(CompositionReaderError::ValidationError(
                format!("Bar {} denominator must be a power of 2", bar_idx + 1)
            ));
        }

        Ok(())
    }

    fn validate_repetition(repetition: &Repetition, rep_idx: usize, total_sections: usize) -> Result<(), CompositionReaderError> {
        // Note: repetition uses 0-based indexing internally after conversion
        if repetition.start_section >= total_sections {
            return Err(CompositionReaderError::ValidationError(
                format!("Repetition {} start_section (internal: {}, YAML: {}) exceeds available sections ({})", 
                       rep_idx + 1, repetition.start_section, repetition.start_section + 1, total_sections)
            ));
        }

        if repetition.end_section >= total_sections {
            return Err(CompositionReaderError::ValidationError(
                format!("Repetition {} end_section (internal: {}, YAML: {}) exceeds available sections ({})", 
                       rep_idx + 1, repetition.end_section, repetition.end_section + 1, total_sections)
            ));
        }

        if repetition.start_section > repetition.end_section {
            return Err(CompositionReaderError::ValidationError(
                format!("Repetition {} start_section ({}) cannot be greater than end_section ({})", 
                       rep_idx + 1, repetition.start_section + 1, repetition.end_section + 1)
            ));
        }

        if repetition.num_repetitions == 0 {
            return Err(CompositionReaderError::ValidationError(
                format!("Repetition {} num_repetitions cannot be zero", rep_idx + 1)
            ));
        }

        // Validate individual ending pattern bars if present
        if let Some(ref ending_pattern) = repetition.individual_ending_pattern {
            for (bar_idx, bar) in ending_pattern.iter().enumerate() {
                Self::validate_bar(bar, bar_idx)?;
            }
        }

        Ok(())
    }

    fn check_repetition_overlaps(repetitions: &[Repetition]) -> Result<(), CompositionReaderError> {
        for (i, rep1) in repetitions.iter().enumerate() {
            for (j, rep2) in repetitions.iter().enumerate() {
                if i != j {
                    // Check if repetitions overlap
                    let overlap = !(rep1.end_section < rep2.start_section || rep2.end_section < rep1.start_section);
                    if overlap {
                        return Err(CompositionReaderError::ValidationError(
                            format!("Repetitions {} and {} have overlapping section ranges", i + 1, j + 1)
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Write a composition to a YAML file
    pub fn write_to_file<P: AsRef<Path>>(composition: &Composition, path: P) -> Result<(), CompositionReaderError> {
        let yaml_composition = YamlComposition::from(composition.clone());
        let yaml_content = serde_yaml::to_string(&yaml_composition)?;
        fs::write(path, yaml_content)?;
        Ok(())
    }

    /// Convert a composition to a YAML string
    pub fn to_yaml_string(composition: &Composition) -> Result<String, CompositionReaderError> {
        let yaml_composition = YamlComposition::from(composition.clone());
        let yaml_content = serde_yaml::to_string(&yaml_composition)?;
        Ok(yaml_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_read_valid_yaml() {
        let yaml_content = r#"
composer_name: "Johann Sebastian Bach"
work_name: "Brandenburg Concerto No. 1"
movements:
  - name: "I. Allegro"
    tempo_bpm: 120.0
    sections:
      - name: "Exposition"
        bars:
          - number: 1
            numerator: 4
            denominator: 4
          - number: 2
            numerator: 4
            denominator: 4
        repetitions: []
      - name: "Development"
        bars:
          - number: 3
            numerator: 4
            denominator: 4
        repetitions:
          - start_section: 1
            end_section: 1
            num_repetitions: 2
            individual_ending_pattern:
              - number: 1
                numerator: 4
                denominator: 4
                name: "first ending"
"#;

        let composition = CompositionReader::read_from_string(yaml_content)
            .expect("Failed to read valid YAML content in test");
        
        assert_eq!(composition.composer_name, "Johann Sebastian Bach");
        assert_eq!(composition.work_name, "Brandenburg Concerto No. 1");
        assert_eq!(composition.movements.len(), 1);
        assert_eq!(composition.movements[0].sections.len(), 2);
    }

    #[test]
    fn test_validation_errors() {
        // Test empty composer name
        let yaml_content = r#"
composer_name: ""
work_name: "Test Work"
movements: []
"#;
        assert!(CompositionReader::read_from_string(yaml_content).is_err());

        // Test invalid time signature
        let yaml_content = r#"
composer_name: "Test Composer"
work_name: "Test Work"
movements:
  - name: "Movement 1"
    sections:
      - name: "Section 1"
        bars:
          - number: 1
            numerator: 0
            denominator: 4
        repetitions: []
"#;
        assert!(CompositionReader::read_from_string(yaml_content).is_err());
    }

    #[test]
    fn test_file_io() {
        let dir = tempdir()
            .expect("Failed to create temporary directory for test");
        let file_path = dir.path().join("test_composition.yaml");

        let mut composition = Composition::new(
            "Test Composer".to_string(),
            "Test Work".to_string(),
        );

        let mut movement = Movement::new("Test Movement".to_string());
        let mut section = Section::new("Test Section".to_string());
        section.add_bar(Bar::new(1, 4, 4));
        movement.add_section(section);
        composition.add_movement(movement);

        // Write to file
        CompositionReader::write_to_file(&composition, &file_path)
            .expect("Failed to write composition to file in test");
        
        // Read back from file
        let read_composition = CompositionReader::read_from_file(&file_path)
            .expect("Failed to read composition from file in test");
        
        assert_eq!(composition.composer_name, read_composition.composer_name);
        assert_eq!(composition.work_name, read_composition.work_name);
    }

    #[test]
    fn test_beethoven_example() {
        let beethoven_path = "examples/beethoven_symphony_5.yaml";
        let composition = CompositionReader::read_from_file(beethoven_path)
            .expect("Failed to read Beethoven YAML file in test");
        
        println!("\n=== ACTUAL Beethoven Symphony 5 File Test ===");
        println!("Composer: {}", composition.composer_name);
        println!("Work: {}", composition.work_name);
        println!("Number of movements: {}", composition.movements.len());
        
        assert_eq!(composition.composer_name, "Ludwig van Beethoven");
        assert_eq!(composition.work_name, "Symphony No. 5 in C minor, Op. 67");
        assert_eq!(composition.movements.len(), 4);
        
        // Print detailed movement information
        for (i, movement) in composition.movements.iter().enumerate() {
            println!("\nMovement {}: {}", i + 1, movement.name);
            println!("  Tempo: {:?}", movement.tempo_bpm);
            println!("  Sections: {}", movement.sections.len());
            
            for (j, section) in movement.sections.iter().enumerate() {
                println!("    Section {}: {} ({} bars, tempo: {:?})", 
                        j + 1, section.name, section.bars.len(), section.tempo_bpm);
                
                // Show first few bars of each section
                for (k, bar) in section.bars.iter().take(3).enumerate() {
                    println!("      Bar {}: {}/{} (number: {})", 
                            k + 1, bar.numerator, bar.denominator, bar.number);
                }
                if section.bars.len() > 3 {
                    println!("      ... and {} more bars", section.bars.len() - 3);
                }
                
                if !section.repetitions.is_empty() {
                    println!("      Repetitions: {}", section.repetitions.len());
                }
            }
        }
        
        // Check movement tempos
        assert_eq!(composition.movements[0].tempo_bpm, Some(108.0));
        assert_eq!(composition.movements[1].tempo_bpm, Some(92.0));
        assert_eq!(composition.movements[2].tempo_bpm, Some(96.0));
        assert_eq!(composition.movements[3].tempo_bpm, Some(84.0));
        
        // Check section tempo override in first movement
        println!("\nChecking section names in first movement:");
        for (i, section) in composition.movements[0].sections.iter().enumerate() {
            println!("  Section {}: '{}'", i, section.name);
        }
        
        // The YAML file shows "Second Theme" as section index 2 (third section)
        assert_eq!(composition.movements[0].sections[2].tempo_bpm, Some(96.0)); // Second Theme
        
        // Check repetition ending names
        let repetition = &composition.movements[0].sections[1].repetitions[0];
        if let Some(ref endings) = repetition.individual_ending_pattern {
            assert_eq!(endings[0].name, Some("first ending".to_string()));
            assert_eq!(endings[1].name, Some("transition".to_string()));
        }
        
        let total_bars = composition.total_bar_count();
        println!("\nTotal bars across all movements: {}", total_bars);
        println!("✅ Beethoven Symphony No. 5 loaded successfully!");
    }
}
