// Musical Composition Structure
// Defines a nested hierarchy: Composition -> Movement -> Section -> Bar
// With support for repetitions and individual endings

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    pub composer_name: String,
    pub work_name: String,
    pub movements: Vec<Movement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movement {
    pub name: String,
    /// Optional tempo marking for this movement (can be overridden by sections)
    #[serde(default)]
    pub tempo_bpm: Option<f64>,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub name: String,
    /// Optional tempo marking for this section (overrides movement tempo if present)
    #[serde(default)]
    pub tempo_bpm: Option<f64>,
    pub bars: Vec<Bar>,
    #[serde(default)]
    pub repetitions: Vec<Repetition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repetition {
    /// Starting section index (inclusive) for repetition
    pub start_section: usize,
    /// Ending section index (inclusive) for repetition
    pub end_section: usize,
    /// Total number of repetitions to perform
    pub num_repetitions: usize,
    /// Individual ending pattern: either all repetitions have identical-length endings or none do
    #[serde(default)]
    pub individual_ending_pattern: Option<Vec<Bar>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    pub number: usize,
    /// Time signature numerator (e.g., 4 in 4/4)
    pub numerator: u32,
    /// Time signature denominator (e.g., 4 in 4/4)
    pub denominator: u32,
    /// Optional name for this bar (useful for repetition endings)
    #[serde(default)]
    pub name: Option<String>,
}

impl Composition {
    pub fn new(composer_name: String, work_name: String) -> Self {
        Self {
            composer_name,
            work_name,
            movements: Vec::new(),
        }
    }

    pub fn add_movement(&mut self, movement: Movement) {
        self.movements.push(movement);
    }

    /// Get total number of movements
    pub fn movement_count(&self) -> usize {
        self.movements.len()
    }

    /// Get movement by index (1-based)
    pub fn get_movement(&self, movement_number: usize) -> Option<&Movement> {
        if movement_number > 0 && movement_number <= self.movements.len() {
            Some(&self.movements[movement_number - 1])
        } else {
            None
        }
    }

    /// Calculate total bar count across all movements, accounting for repetitions
    pub fn total_bar_count(&self) -> usize {
        self.movements.iter().map(|m| m.total_bar_count()).sum()
    }
}

impl Movement {
    pub fn new(name: String) -> Self {
        Self {
            name,
            tempo_bpm: None,
            sections: Vec::new(),
        }
    }

    pub fn with_tempo(mut self, tempo_bpm: f64) -> Self {
        self.tempo_bpm = Some(tempo_bpm);
        self
    }

    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    /// Get section by index (1-based)
    pub fn get_section(&self, section_number: usize) -> Option<&Section> {
        if section_number > 0 && section_number <= self.sections.len() {
            Some(&self.sections[section_number - 1])
        } else {
            None
        }
    }

    /// Get all effective tempos for sections in this movement, with inheritance
    pub fn get_section_tempos(&self) -> Vec<Option<f64>> {
        self.sections.iter()
            .map(|section| section.effective_tempo(self.tempo_bpm))
            .collect()
    }

    /// Calculate total bar count for this movement, accounting for repetitions
    pub fn total_bar_count(&self) -> usize {
        // Generate the complete sequence and count bars - this handles all repetitions correctly
        self.generate_bar_sequence().len()
    }

    /// Generate the complete bar sequence for this movement, handling repetitions
    pub fn generate_bar_sequence(&self) -> Vec<ExpandedBar> {
        let mut sequence = Vec::new();
        let mut current_bar_number = 1;
        
        for (section_index, section) in self.sections.iter().enumerate() {
            // Add base section bars
            for bar in &section.bars {
                sequence.push(ExpandedBar {
                    original_bar: bar.clone(),
                    logical_bar_number: current_bar_number,
                    section_index,
                    section_name: section.name.clone(),
                    is_repetition: false,
                    repetition_index: None,
                    is_individual_ending: false,
                });
                current_bar_number += 1;
            }
            
            // Handle repetitions for this section
            for (rep_index, repetition) in section.repetitions.iter().enumerate() {
                for _repeat_num in 0..repetition.num_repetitions {
                    // Reset bar counting for each repetition
                    let mut repetition_bar_number = 1;
                    
                    // Add repeated sections
                    for rep_section_index in repetition.start_section..=repetition.end_section {
                        if let Some(rep_section) = self.sections.get(rep_section_index) {
                            for bar in &rep_section.bars {
                                sequence.push(ExpandedBar {
                                    original_bar: Bar {
                                        number: repetition_bar_number,
                                        ..bar.clone()
                                    },
                                    logical_bar_number: current_bar_number,
                                    section_index: rep_section_index,
                                    section_name: rep_section.name.clone(),
                                    is_repetition: true,
                                    repetition_index: Some(rep_index),
                                    is_individual_ending: false,
                                });
                                current_bar_number += 1;
                                repetition_bar_number += 1;
                            }
                        }
                    }
                    
                    // Add individual ending if pattern is defined
                    if let Some(ref ending_pattern) = repetition.individual_ending_pattern {
                        for bar in ending_pattern {
                            sequence.push(ExpandedBar {
                                original_bar: Bar {
                                    number: repetition_bar_number,
                                    ..bar.clone()
                                },
                                logical_bar_number: current_bar_number,
                                section_index,
                                section_name: format!("{} (Ending {})", section.name, rep_index + 1),
                                is_repetition: true,
                                repetition_index: Some(rep_index),
                                is_individual_ending: true,
                            });
                            current_bar_number += 1;
                            repetition_bar_number += 1;
                        }
                    }
                }
            }
        }
        
        sequence
    }
}

impl Section {
    pub fn new(name: String) -> Self {
        Self {
            name,
            tempo_bpm: None,
            bars: Vec::new(),
            repetitions: Vec::new(),
        }
    }

    pub fn with_tempo(mut self, tempo_bpm: f64) -> Self {
        self.tempo_bpm = Some(tempo_bpm);
        self
    }

    /// Get the effective tempo for this section, inheriting from movement if not set
    pub fn effective_tempo(&self, movement_tempo: Option<f64>) -> Option<f64> {
        self.tempo_bpm.or(movement_tempo)
    }

    pub fn add_bar(&mut self, bar: Bar) {
        self.bars.push(bar);
    }

    pub fn add_repetition(&mut self, repetition: Repetition) {
        self.repetitions.push(repetition);
    }

    /// Get base bar count (without repetitions)
    pub fn base_bar_count(&self) -> usize {
        self.bars.len()
    }
}

impl Bar {
    pub fn new(number: usize, numerator: u32, denominator: u32) -> Self {
        Self {
            number,
            numerator,
            denominator,
            name: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

/// Expanded bar representation showing the complete performance sequence
#[derive(Debug, Clone)]
pub struct ExpandedBar {
    pub original_bar: Bar,
    pub logical_bar_number: usize,
    pub section_index: usize,
    pub section_name: String,
    pub is_repetition: bool,
    pub repetition_index: Option<usize>,
    pub is_individual_ending: bool,
}

impl Repetition {
    pub fn new(start_section: usize, end_section: usize, num_repetitions: usize) -> Self {
        Self {
            start_section,
            end_section,
            num_repetitions,
            individual_ending_pattern: None,
        }
    }

    pub fn with_individual_ending_pattern(mut self, ending_pattern: Vec<Bar>) -> Self {
        self.individual_ending_pattern = Some(ending_pattern);
        self
    }

    /// Get the length of individual endings (0 if no pattern is defined)
    pub fn individual_ending_length(&self) -> usize {
        match &self.individual_ending_pattern {
            Some(pattern) => pattern.len(),
            None => 0,
        }
    }

    /// Check if this repetition has individual endings
    pub fn has_individual_endings(&self) -> bool {
        self.individual_ending_pattern.is_some()
    }
}

/// Internal reference information for a bar within the composition hierarchy
#[derive(Debug, Clone)]
struct BarReferenceInfo {
    movement_index: usize,
    section_index: usize,
    repetition_index: Option<usize>,
    repetition_section_index: Option<usize>, // Which section contains the repetition
    is_individual_ending: bool,
    bar_index_in_section: usize,
}

/// Reference result containing actual object references for a bar
#[derive(Debug)]
pub struct BarReferences<'a> {
    pub movement: &'a Movement,
    pub section: &'a Section,
    pub repetition: Option<&'a Repetition>,
    pub movement_index: usize,
    pub section_index: usize,
    pub repetition_index: Option<usize>,
    pub is_individual_ending: bool,
    pub bar_index_in_section: usize,
}

/// Efficient bar reference lookup system using range-based dictionary
/// Maps bar number ranges to their containing structures for O(log n) lookups
pub struct BarReference<'a> {
    composition: &'a Composition,
    /// Internal mapping from bar ranges to reference information
    bar_ranges: BTreeMap<(usize, usize), BarReferenceInfo>,
}

impl<'a> BarReference<'a> {
    /// Create a new BarReference for the given composition
    pub fn new(composition: &'a Composition) -> Self {
        let mut bar_ranges = BTreeMap::new();
        let mut current_bar_number = 1;
        
        for (movement_index, movement) in composition.movements.iter().enumerate() {
            Self::build_movement_ranges(
                &mut bar_ranges,
                &mut current_bar_number,
                movement,
                movement_index,
            );
        }
        
        Self {
            composition,
            bar_ranges,
        }
    }
    
    /// Main accessor: Get object references for a given bar number
    /// Returns actual references to Movement, Section, and optional Repetition objects
    pub fn get_bar_objects(&self, bar_number: usize) -> Option<BarReferences<'a>> {
        // Find the reference info first
        let ref_info = self.find_bar_reference_info(bar_number)?;
        
        // Get actual object references
        let movement = &self.composition.movements[ref_info.movement_index];
        let section = &movement.sections[ref_info.section_index];
        
        // Find the repetition object if this bar is part of a repetition
        let repetition = if let Some(rep_idx) = ref_info.repetition_index {
            if let Some(rep_section_idx) = ref_info.repetition_section_index {
                if let Some(rep_section) = movement.sections.get(rep_section_idx) {
                    rep_section.repetitions.get(rep_idx)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        Some(BarReferences {
            movement,
            section,
            repetition,
            movement_index: ref_info.movement_index,
            section_index: ref_info.section_index,
            repetition_index: ref_info.repetition_index,
            is_individual_ending: ref_info.is_individual_ending,
            bar_index_in_section: ref_info.bar_index_in_section,
        })
    }
    
    /// Internal method to find reference information for a bar number
    fn find_bar_reference_info(&self, bar_number: usize) -> Option<&BarReferenceInfo> {
        // Look for a range that contains this bar number
        for ((start_bar, end_bar), ref_info) in &self.bar_ranges {
            if bar_number >= *start_bar && bar_number <= *end_bar {
                return Some(ref_info);
            }
        }
        None
    }
    
    /// Build bar ranges for a movement and all its sections/repetitions
    fn build_movement_ranges(
        bar_ranges: &mut BTreeMap<(usize, usize), BarReferenceInfo>,
        current_bar_number: &mut usize,
        movement: &Movement,
        movement_index: usize,
    ) {
        for (section_index, section) in movement.sections.iter().enumerate() {
            // Add base section bars
            if !section.bars.is_empty() {
                for (bar_index, _bar) in section.bars.iter().enumerate() {
                    let bar_num = *current_bar_number + bar_index;
                    bar_ranges.insert(
                        (bar_num, bar_num),
                        BarReferenceInfo {
                            movement_index,
                            section_index,
                            repetition_index: None,
                            repetition_section_index: None,
                            is_individual_ending: false,
                            bar_index_in_section: bar_index,
                        },
                    );
                }
                
                *current_bar_number += section.bars.len();
            }
            
            // Handle repetitions for this section
            for (rep_index, repetition) in section.repetitions.iter().enumerate() {
                for _rep_num in 0..repetition.num_repetitions {
                    // Add repeated sections
                    for rep_section_index in repetition.start_section..=repetition.end_section {
                        if let Some(rep_section) = movement.sections.get(rep_section_index) {
                            if !rep_section.bars.is_empty() {
                                for (bar_index, _bar) in rep_section.bars.iter().enumerate() {
                                    bar_ranges.insert(
                                        (*current_bar_number, *current_bar_number),
                                        BarReferenceInfo {
                                            movement_index,
                                            section_index: rep_section_index,
                                            repetition_index: Some(rep_index),
                                            repetition_section_index: Some(section_index), // The section that contains this repetition
                                            is_individual_ending: false,
                                            bar_index_in_section: bar_index,
                                        },
                                    );
                                    *current_bar_number += 1;
                                }
                            }
                        }
                    }
                    
                    // Add individual ending if pattern is defined
                    if let Some(ref ending_pattern) = repetition.individual_ending_pattern {
                        for (bar_index, _bar) in ending_pattern.iter().enumerate() {
                            bar_ranges.insert(
                                (*current_bar_number, *current_bar_number),
                                BarReferenceInfo {
                                    movement_index,
                                    section_index,
                                    repetition_index: Some(rep_index),
                                    repetition_section_index: Some(section_index), // The section that contains this repetition
                                    is_individual_ending: true,
                                    bar_index_in_section: bar_index,
                                },
                            );
                            *current_bar_number += 1;
                        }
                    }
                }
            }
        }
    }
    
    /// Test if a given bar number is within any range in the composition
    pub fn contains_bar(&self, bar_number: usize) -> bool {
        self.find_bar_reference_info(bar_number).is_some()
    }
    
    /// Get all bar ranges in the composition
    pub fn get_all_ranges(&self) -> Vec<(usize, usize)> {
        self.bar_ranges.keys().cloned().collect()
    }
    
    /// Get the total number of bars in the composition
    pub fn total_bars(&self) -> usize {
        self.composition.total_bar_count()
    }
    
    /// Get a reference to the underlying composition
    pub fn composition(&self) -> &Composition {
        self.composition
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composition_creation() {
        let mut composition = Composition::new(
            "Ludwig van Beethoven".to_string(),
            "Symphony No. 9".to_string(),
        );

        let mut movement = Movement::new("I. Allegro ma non troppo".to_string()).with_tempo(120.0);
        let mut section = Section::new("Exposition".to_string());
        
        section.add_bar(Bar::new(1, 4, 4));
        section.add_bar(Bar::new(2, 4, 4));
        movement.add_section(section);
        
        composition.add_movement(movement);

        assert_eq!(composition.composer_name, "Ludwig van Beethoven");
        assert_eq!(composition.work_name, "Symphony No. 9");
        assert_eq!(composition.movement_count(), 1);
    }

    #[test]
    fn test_repetition_bar_counting() {
        let mut movement = Movement::new("Test Movement".to_string());
        
        // Section 1: 2 bars
        let mut section1 = Section::new("A".to_string());
        section1.add_bar(Bar::new(1, 4, 4));
        section1.add_bar(Bar::new(2, 4, 4));
        movement.add_section(section1);
        
        // Section 2: 2 bars with repetition
        let mut section2 = Section::new("B".to_string());
        section2.add_bar(Bar::new(3, 4, 4));
        section2.add_bar(Bar::new(4, 4, 4));
        
        // Add repetition: repeat sections 0-1 twice with individual ending pattern
        let repetition = Repetition::new(0, 1, 2)
            .with_individual_ending_pattern(vec![Bar::new(1, 4, 4)]);
        section2.add_repetition(repetition);
        
        movement.add_section(section2);
        
        // Base bars: 4, Repetition: (2+2)*2 + 1*2 = 10, Total: 14
        // Let's trace this: A(2) + B(2) + repetition[A(2)+B(2)]*2 + ending(1)*2 = 2+2+8+2 = 14
        assert_eq!(movement.total_bar_count(), 14);
        
        let sequence = movement.generate_bar_sequence();
        assert_eq!(sequence.len(), 14);
        
        // Check that repetition bars have correct numbering
        let repetition_bars: Vec<_> = sequence.iter()
            .filter(|b| b.is_repetition)
            .collect();
        assert_eq!(repetition_bars.len(), 10); // 4 bars repeated twice + 1 ending repeated twice
    }

    #[test]
    fn test_bar_reference_creation() {
        let mut composition = Composition::new(
            "Test Composer".to_string(),
            "Test Work".to_string(),
        );

        let mut movement = Movement::new("I. Allegro".to_string()).with_tempo(120.0);
        
        // Section 1: 2 bars
        let mut section1 = Section::new("Exposition".to_string());
        section1.add_bar(Bar::new(1, 4, 4));
        section1.add_bar(Bar::new(2, 4, 4));
        movement.add_section(section1);
        
        // Section 2: 2 bars with repetition
        let mut section2 = Section::new("Development".to_string());
        section2.add_bar(Bar::new(3, 4, 4));
        section2.add_bar(Bar::new(4, 4, 4));
        
        // Add repetition: repeat section 0 once with individual ending
        let repetition = Repetition::new(0, 0, 1)
            .with_individual_ending_pattern(vec![Bar::new(1, 4, 4).with_name("ending".to_string())]);
        section2.add_repetition(repetition);
        
        movement.add_section(section2);
        composition.add_movement(movement);
        
        let bar_ref = BarReference::new(&composition);
        
        // Test basic lookups
        assert!(bar_ref.contains_bar(1));
        assert!(bar_ref.contains_bar(4));
        assert!(bar_ref.contains_bar(7)); // Individual ending
        assert!(!bar_ref.contains_bar(8)); // Beyond total bars
        
        // Test specific bar references using new API
        let bar1_ref = bar_ref.get_bar_objects(1).expect("Bar 1 should exist");
        assert_eq!(bar1_ref.movement_index, 0);
        assert_eq!(bar1_ref.section_index, 0);
        assert_eq!(bar1_ref.section.name, "Exposition");
        assert_eq!(bar1_ref.repetition_index, None);
        assert!(!bar1_ref.is_individual_ending);
        
        let bar5_ref = bar_ref.get_bar_objects(5).expect("Bar 5 should exist (repetition)");
        assert_eq!(bar5_ref.movement_index, 0);
        assert_eq!(bar5_ref.section_index, 0); // Repeated section
        assert_eq!(bar5_ref.section.name, "Exposition");
        assert_eq!(bar5_ref.repetition_index, Some(0));
        assert!(!bar5_ref.is_individual_ending);
        
        let bar7_ref = bar_ref.get_bar_objects(7).expect("Bar 7 should exist (individual ending)");
        assert_eq!(bar7_ref.movement_index, 0);
        assert_eq!(bar7_ref.section_index, 1); // Development section
        assert_eq!(bar7_ref.repetition_index, Some(0));
        assert!(bar7_ref.is_individual_ending);
        
        // Test utility methods
        assert_eq!(bar_ref.total_bars(), 7);
        assert_eq!(bar_ref.get_all_ranges().len(), 7); // Each bar has its own range
        
        // Test getting bars by movement - check if we can find bars from movement 0 (all bars should be in movement 0)
        let all_ranges = bar_ref.get_all_ranges();
        let movement_0_count = all_ranges.iter()
            .filter(|(start, _)| {
                if let Some(refs) = bar_ref.get_bar_objects(*start) {
                    refs.movement_index == 0
                } else {
                    false
                }
            })
            .count();
        assert_eq!(movement_0_count, 7);
        
        // Test getting bars by section - section 0 should have 4 bars (2 base + 2 repetition)
        let section_0_count = all_ranges.iter()
            .filter(|(start, _)| {
                if let Some(refs) = bar_ref.get_bar_objects(*start) {
                    refs.section_index == 0
                } else {
                    false
                }
            })
            .count();
        assert_eq!(section_0_count, 4);
        
        // Test getting repetition bars - should be 3 bars total
        let repetition_count = all_ranges.iter()
            .filter(|(start, _)| {
                if let Some(refs) = bar_ref.get_bar_objects(*start) {
                    refs.repetition_index.is_some()
                } else {
                    false
                }
            })
            .count();
        assert_eq!(repetition_count, 3);
        
        // Test getting individual ending bars - should be 1 bar
        let ending_count = all_ranges.iter()
            .filter(|(start, _)| {
                if let Some(refs) = bar_ref.get_bar_objects(*start) {
                    refs.is_individual_ending
                } else {
                    false
                }
            })
            .count();
        assert_eq!(ending_count, 1);
    }

    #[test]
    fn test_bar_reference_multiple_movements() {
        let mut composition = Composition::new(
            "Test Composer".to_string(),
            "Multi-Movement Work".to_string(),
        );

        // Movement 1: 2 bars
        let mut movement1 = Movement::new("I. Allegro".to_string());
        let mut section1 = Section::new("Theme".to_string());
        section1.add_bar(Bar::new(1, 4, 4));
        section1.add_bar(Bar::new(2, 4, 4));
        movement1.add_section(section1);
        composition.add_movement(movement1);
        
        // Movement 2: 3 bars
        let mut movement2 = Movement::new("II. Andante".to_string());
        let mut section2 = Section::new("Variation".to_string());
        section2.add_bar(Bar::new(1, 3, 4));
        section2.add_bar(Bar::new(2, 3, 4));
        section2.add_bar(Bar::new(3, 3, 4));
        movement2.add_section(section2);
        composition.add_movement(movement2);
        
        let bar_ref = BarReference::new(&composition);
        
        // Test total bars
        assert_eq!(bar_ref.total_bars(), 5);
        
        // Test movement boundaries using new API
        let bar2_ref = bar_ref.get_bar_objects(2).expect("Bar 2 should exist");
        assert_eq!(bar2_ref.movement_index, 0);
        assert_eq!(bar2_ref.movement.name, "I. Allegro");
        
        let bar3_ref = bar_ref.get_bar_objects(3).expect("Bar 3 should exist");
        assert_eq!(bar3_ref.movement_index, 1);
        assert_eq!(bar3_ref.movement.name, "II. Andante");
        
        // Test movement-specific queries using new API
        let all_ranges = bar_ref.get_all_ranges();
        let movement1_count = all_ranges.iter()
            .filter(|(start, _)| {
                if let Some(refs) = bar_ref.get_bar_objects(*start) {
                    refs.movement_index == 0
                } else {
                    false
                }
            })
            .count();
        assert_eq!(movement1_count, 2);
        
        let movement2_count = all_ranges.iter()
            .filter(|(start, _)| {
                if let Some(refs) = bar_ref.get_bar_objects(*start) {
                    refs.movement_index == 1
                } else {
                    false
                }
            })
            .count();
        assert_eq!(movement2_count, 3);
    }

    #[test]
    fn test_bar_reference_empty_composition() {
        let composition = Composition::new(
            "Empty Composer".to_string(),
            "Empty Work".to_string(),
        );
        
        let bar_ref = BarReference::new(&composition);
        
        assert_eq!(bar_ref.total_bars(), 0);
        assert!(!bar_ref.contains_bar(1));
        assert!(bar_ref.get_bar_objects(1).is_none());
        assert!(bar_ref.get_all_ranges().is_empty());
    }

    #[test]
    fn test_bar_reference_with_beethoven_example() {
        // This test demonstrates BarReference with a more complex composition
        
        // Create a simple composition similar to the Beethoven structure
        let mut composition = Composition::new(
            "Ludwig van Beethoven".to_string(),
            "Symphony No. 5 (simplified)".to_string(),
        );

        let mut movement1 = Movement::new("I. Allegro con brio".to_string()).with_tempo(108.0);
        
        // First Theme: 4 bars
        let mut first_theme = Section::new("First Theme".to_string());
        for i in 1..=4 {
            first_theme.add_bar(Bar::new(i, 2, 4));
        }
        movement1.add_section(first_theme);
        
        // Bridge: 2 bars with repetition
        let mut bridge = Section::new("Bridge".to_string());
        bridge.add_bar(Bar::new(5, 2, 4));
        bridge.add_bar(Bar::new(6, 2, 4));
        
        // Repeat first theme with individual endings
        let repetition = Repetition::new(0, 0, 1)
            .with_individual_ending_pattern(vec![
                Bar::new(1, 2, 4).with_name("first ending".to_string()),
                Bar::new(2, 2, 4).with_name("transition".to_string()),
            ]);
        bridge.add_repetition(repetition);
        movement1.add_section(bridge);
        
        composition.add_movement(movement1);
        
        let bar_ref = BarReference::new(&composition);
        
        // Test total structure
        // Structure: 4 (First Theme) + 2 (Bridge) + 4 (repeated First Theme) + 2 (endings) = 12 bars
        assert_eq!(bar_ref.total_bars(), 12);
        
        // Test specific bar lookups using new API
        let bar1_ref = bar_ref.get_bar_objects(1).expect("Bar 1 should exist");
        assert_eq!(bar1_ref.section.name, "First Theme");
        assert_eq!(bar1_ref.repetition_index, None);
        
        let bar7_ref = bar_ref.get_bar_objects(7).expect("Bar 7 should exist (repeated first theme)");
        assert_eq!(bar7_ref.section.name, "First Theme");
        assert_eq!(bar7_ref.repetition_index, Some(0));
        assert!(!bar7_ref.is_individual_ending);
        
        let bar11_ref = bar_ref.get_bar_objects(11).expect("Bar 11 should exist (individual ending)");
        assert_eq!(bar11_ref.repetition_index, Some(0));
        assert!(bar11_ref.is_individual_ending);
        
        // Test queries using new API
        let all_ranges = bar_ref.get_all_ranges();
        
        // Count bars in first theme section (movement 0, section 0) - should be 8 (4 original + 4 repeated)
        let first_theme_count = all_ranges.iter()
            .filter(|(start, _)| {
                if let Some(refs) = bar_ref.get_bar_objects(*start) {
                    refs.movement_index == 0 && refs.section_index == 0
                } else {
                    false
                }
            })
            .count();
        assert_eq!(first_theme_count, 8);
        
        // Count repetition bars - should be 6 (4 repeated theme bars + 2 endings)
        let repetition_count = all_ranges.iter()
            .filter(|(start, _)| {
                if let Some(refs) = bar_ref.get_bar_objects(*start) {
                    refs.repetition_index.is_some()
                } else {
                    false
                }
            })
            .count();
        assert_eq!(repetition_count, 6);
        
        // Count individual ending bars - should be 2
        let ending_count = all_ranges.iter()
            .filter(|(start, _)| {
                if let Some(refs) = bar_ref.get_bar_objects(*start) {
                    refs.is_individual_ending
                } else {
                    false
                }
            })
            .count();
        assert_eq!(ending_count, 2);
        
        println!("✅ BarReference works correctly with complex composition structure!");
    }

    #[test]
    fn test_bar_reference_with_real_beethoven_yaml() {
        use crate::api::fm_composition_reader::CompositionReader;
        
        // Load the actual Beethoven YAML file
        let beethoven_path = "examples/beethoven_symphony_5.yaml";
        let composition = CompositionReader::read_from_file(beethoven_path)
            .expect("Failed to read Beethoven YAML file for BarReference test");
        
        let bar_ref = BarReference::new(&composition);
        
        println!("\n=== BarReference Test with Real Beethoven YAML ===");
        println!("Total bars in composition: {}", bar_ref.total_bars());
        
        // Let's first examine the structure by printing out the first 20 bars
        for bar_num in 1..=20.min(bar_ref.total_bars()) {
            if let Some(refs) = bar_ref.get_bar_objects(bar_num) {
                println!("Bar {}: Movement {} '{}' - Section {} '{}' - Rep: {:?} - Ending: {}", 
                         bar_num, 
                         refs.movement_index, 
                         refs.movement.name,
                         refs.section_index,
                         refs.section.name,
                         refs.repetition_index,
                         refs.is_individual_ending);
            }
        }
        
        // Based on the actual structure observed:
        // Movement 1 "I. Allegro con brio":
        //   Section 0 "First Theme (Fate Motif)": bars 1-4
        //   Section 1 "Bridge": bars 5-6
        //   Bridge repetitions: repeat section 0 (First Theme) twice with individual endings
        //     - Repetition 1: First Theme (bars 7-10) + endings (bars 11-12)
        //     - Repetition 2: First Theme (bars 13-16) + endings (bars 17-18)
        //   Section 2 "Second Theme": bars 19-22
        //   Section 3 "Development": bars 23-26
        //   Development repetitions: repeat sections 2-3 once with individual ending
        
        // Test bars from different sections in the first movement
        
        // Bar 2: First Theme (no repetition)
        let bar2_ref = bar_ref.get_bar_objects(2).expect("Bar 2 should exist in First Theme");
        assert_eq!(bar2_ref.movement.name, "I. Allegro con brio");
        assert_eq!(bar2_ref.section.name, "First Theme (Fate Motif)");
        assert_eq!(bar2_ref.movement_index, 0);
        assert_eq!(bar2_ref.section_index, 0);
        assert_eq!(bar2_ref.repetition_index, None);
        assert!(!bar2_ref.is_individual_ending);
        println!("✓ Bar 2: {} - {}", bar2_ref.movement.name, bar2_ref.section.name);
        
        // Bar 6: Bridge (original section, no repetition)
        let bar6_ref = bar_ref.get_bar_objects(6).expect("Bar 6 should exist in Bridge");
        assert_eq!(bar6_ref.movement.name, "I. Allegro con brio");
        assert_eq!(bar6_ref.section.name, "Bridge");
        assert_eq!(bar6_ref.movement_index, 0);
        assert_eq!(bar6_ref.section_index, 1);
        assert_eq!(bar6_ref.repetition_index, None);
        assert!(!bar6_ref.is_individual_ending);
        println!("✓ Bar 6: {} - {}", bar6_ref.movement.name, bar6_ref.section.name);
        
        // Bar 8: First repetition of First Theme section (called by Bridge repetition)
        let bar8_ref = bar_ref.get_bar_objects(8).expect("Bar 8 should exist in repeated First Theme");
        assert_eq!(bar8_ref.movement.name, "I. Allegro con brio");
        assert_eq!(bar8_ref.section.name, "First Theme (Fate Motif)");
        assert_eq!(bar8_ref.movement_index, 0);
        assert_eq!(bar8_ref.section_index, 0); // Refers to First Theme section 
        assert_eq!(bar8_ref.repetition_index, Some(0));
        assert!(!bar8_ref.is_individual_ending);
        println!("✓ Bar 8: {} - {} (repetition {})", 
                 bar8_ref.movement.name, bar8_ref.section.name, bar8_ref.repetition_index.unwrap());
        
        // Bar 12: Individual ending from first Bridge repetition
        let bar12_ref = bar_ref.get_bar_objects(12).expect("Bar 12 should exist as individual ending");
        assert_eq!(bar12_ref.movement.name, "I. Allegro con brio");
        assert_eq!(bar12_ref.section.name, "Bridge");
        assert_eq!(bar12_ref.movement_index, 0);
        assert_eq!(bar12_ref.section_index, 1); // Part of Bridge repetition
        assert_eq!(bar12_ref.repetition_index, Some(0));
        assert!(bar12_ref.is_individual_ending);
        println!("✓ Bar 12: {} - {} (individual ending from repetition {})", 
                 bar12_ref.movement.name, bar12_ref.section.name, bar12_ref.repetition_index.unwrap());
        
        // Bar 20: Second Theme (original section)
        let bar20_ref = bar_ref.get_bar_objects(20).expect("Bar 20 should exist in Second Theme");
        assert_eq!(bar20_ref.movement.name, "I. Allegro con brio");
        assert_eq!(bar20_ref.section.name, "Second Theme");
        assert_eq!(bar20_ref.movement_index, 0);
        assert_eq!(bar20_ref.section_index, 2);
        assert_eq!(bar20_ref.repetition_index, None);
        assert!(!bar20_ref.is_individual_ending);
        println!("✓ Bar 20: {} - {}", bar20_ref.movement.name, bar20_ref.section.name);
        
        // Bar 25: Development (original section)  
        let bar25_ref = bar_ref.get_bar_objects(25).expect("Bar 25 should exist in Development");
        assert_eq!(bar25_ref.movement.name, "I. Allegro con brio");
        assert_eq!(bar25_ref.section.name, "Development");
        assert_eq!(bar25_ref.movement_index, 0);
        assert_eq!(bar25_ref.section_index, 3);
        assert_eq!(bar25_ref.repetition_index, None);
        assert!(!bar25_ref.is_individual_ending);
        println!("✓ Bar 25: {} - {}", bar25_ref.movement.name, bar25_ref.section.name);
        
        // Test a bar from the second movement
        let second_movement_bars = bar_ref.get_all_ranges().iter()
            .filter_map(|(start, _)| {
                bar_ref.get_bar_objects(*start).and_then(|refs| {
                    if refs.movement_index == 1 {
                        Some(*start)
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();
        
        if let Some(&first_bar_of_second_movement) = second_movement_bars.first() {
            let second_mv_ref = bar_ref.get_bar_objects(first_bar_of_second_movement)
                .expect("First bar of second movement should exist");
            assert_eq!(second_mv_ref.movement.name, "II. Andante con moto");
            assert_eq!(second_mv_ref.movement_index, 1);
            println!("✓ Bar {}: {} - {}", 
                     first_bar_of_second_movement, 
                     second_mv_ref.movement.name, 
                     second_mv_ref.section.name);
        }
        
        // Verify total structure consistency
        let total_bars = bar_ref.total_bars();
        let all_ranges = bar_ref.get_all_ranges();
        assert_eq!(all_ranges.len(), total_bars);
        
        // Count bars by movement to verify structure
        let movement_counts: Vec<usize> = (0..composition.movements.len())
            .map(|mov_idx| {
                all_ranges.iter()
                    .filter(|(start, _)| {
                        if let Some(refs) = bar_ref.get_bar_objects(*start) {
                            refs.movement_index == mov_idx
                        } else {
                            false
                        }
                    })
                    .count()
            })
            .collect();
        
        println!("Movement bar counts: {:?}", movement_counts);
        println!("Total bars across all movements: {}", movement_counts.iter().sum::<usize>());
        assert_eq!(movement_counts.iter().sum::<usize>(), total_bars);
        
        println!("✅ BarReference correctly handles real Beethoven YAML with complex repetition structure!");
    }
}
