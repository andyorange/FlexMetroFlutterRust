// Integration test for Composition structure and YAML reader
// Demonstrates the complete functionality with a real musical example

#[cfg(test)]
mod composition_integration_tests {
    use crate::api::fm_composition::{Composition, Movement, Section, Bar, Repetition};
    use crate::api::fm_composition_reader::CompositionReader;

    #[test]
    fn test_composition_with_repetitions() {
        println!("\n=== Composition Structure Integration Test ===");
        
        // Create a composition programmatically
        let mut composition = Composition::new(
            "Wolfgang Amadeus Mozart".to_string(),
            "Piano Sonata No. 16 in C major, K. 545".to_string(),
        );

        // First movement: Allegro
        let mut movement1 = Movement::new("I. Allegro".to_string()).with_tempo(120.0);
        
        // Exposition section
        let mut exposition = Section::new("Exposition".to_string());
        exposition.add_bar(Bar::new(1, 4, 4).with_name("A".to_string()));
        exposition.add_bar(Bar::new(2, 4, 4));
        exposition.add_bar(Bar::new(3, 4, 4));
        exposition.add_bar(Bar::new(4, 4, 4));
        movement1.add_section(exposition);
        
        // Development section with repetition
        let mut development = Section::new("Development".to_string());
        development.add_bar(Bar::new(5, 4, 4).with_name("B".to_string()));
        development.add_bar(Bar::new(6, 4, 4));
        
        // Add repetition: repeat Exposition twice with individual ending pattern
        let repetition = Repetition::new(0, 0, 2)
            .with_individual_ending_pattern(vec![
                Bar::new(1, 4, 4).with_name("forte".to_string()),
                Bar::new(2, 4, 4),
            ]);
        development.add_repetition(repetition);
        movement1.add_section(development);
        
        // Recapitulation
        let mut recapitulation = Section::new("Recapitulation".to_string());
        recapitulation.add_bar(Bar::new(7, 4, 4).with_name("A'".to_string()));
        recapitulation.add_bar(Bar::new(8, 4, 4));
        movement1.add_section(recapitulation);
        
        composition.add_movement(movement1);

        // Analyze the structure
        println!("Composition: {} - {}", composition.composer_name, composition.work_name);
        println!("Total movements: {}", composition.movement_count());
        
        let movement = composition.get_movement(1)
            .expect("Movement 1 should exist in test composition");
        println!("\nMovement 1: {}", movement.name);
        println!("Total bars (with repetitions): {}", movement.total_bar_count());
        
        // Expected: 4 (Exposition) + 2 (Development) + 2 (Recapitulation) + 4*2 (repeated Exposition) + 2*2 (individual endings) = 20
        // Analysis: Exposition(4) + Development(2) + Recapitulation(2) + [Exposition(4)*2 + ending(2)*2] = 4+2+2+8+4 = 20
        assert_eq!(movement.total_bar_count(), 20);
        
        // Generate complete bar sequence
        let sequence = movement.generate_bar_sequence();
        println!("Complete performance sequence:");
        
        for (i, expanded_bar) in sequence.iter().enumerate() {
            let rep_info = if expanded_bar.is_repetition {
                if expanded_bar.is_individual_ending {
                    format!(" [Ending {}]", expanded_bar.repetition_index
                        .expect("Repetition index should be present for individual ending") + 1)
                } else {
                    format!(" [Rep {}]", expanded_bar.repetition_index
                        .expect("Repetition index should be present for repetition") + 1)
                }
            } else {
                String::new()
            };
            
            println!("  {:2}. Bar {} | {}/{} | {} | Logical #{}{}", 
                    i + 1,
                    expanded_bar.original_bar.number,
                    expanded_bar.original_bar.numerator, 
                    expanded_bar.original_bar.denominator,
                    expanded_bar.section_name,
                    expanded_bar.logical_bar_number,
                    rep_info);
        }
        
        assert_eq!(sequence.len(), 20);
        
        // Verify section distribution
        let exposition_bars = sequence.iter().filter(|b| b.section_name == "Exposition").count();
        let development_bars = sequence.iter().filter(|b| b.section_name == "Development").count();
        let recapitulation_bars = sequence.iter().filter(|b| b.section_name == "Recapitulation").count();
        let ending_bars = sequence.iter().filter(|b| b.is_individual_ending).count();
        
        println!("\nSection distribution:");
        println!("  Exposition: {} bars", exposition_bars);
        println!("  Development: {} bars", development_bars);
        println!("  Recapitulation: {} bars", recapitulation_bars);
        println!("  Individual endings: {} bars", ending_bars);
        
        assert_eq!(exposition_bars, 12); // 4 original + 4*2 repeated = 12
        assert_eq!(development_bars, 2);
        assert_eq!(recapitulation_bars, 2);
        assert_eq!(ending_bars, 4); // 2*2 ending bars
        
        println!("\n✅ Composition structure test passed!");
    }

    #[test]
    fn test_yaml_round_trip() {
        println!("\n=== YAML Read/Write Integration Test ===");
        
        // Create a test composition
        let mut composition = Composition::new(
            "Test Composer".to_string(),
            "Test Symphony".to_string(),
        );

        let mut movement = Movement::new("I. Allegro".to_string()).with_tempo(120.0);
        let mut section = Section::new("Theme".to_string());
        section.add_bar(Bar::new(1, 4, 4));
        section.add_bar(Bar::new(2, 4, 4));
        
        let repetition = Repetition::new(0, 0, 1);
        section.add_repetition(repetition);
        
        movement.add_section(section);
        composition.add_movement(movement);

        // Convert to YAML
        let yaml_string = CompositionReader::to_yaml_string(&composition)
            .expect("Failed to convert composition to YAML in test");
        println!("Generated YAML:");
        println!("{}", yaml_string);
        
        // Read back from YAML
        let read_composition = CompositionReader::read_from_string(&yaml_string)
            .expect("Failed to read composition from generated YAML in test");
        
        // Verify it matches
        assert_eq!(composition.composer_name, read_composition.composer_name);
        assert_eq!(composition.work_name, read_composition.work_name);
        assert_eq!(composition.movements.len(), read_composition.movements.len());
        
        let original_movement = &composition.movements[0];
        let read_movement = &read_composition.movements[0];
        assert_eq!(original_movement.name, read_movement.name);
        assert_eq!(original_movement.total_bar_count(), read_movement.total_bar_count());
        
        println!("✅ YAML round-trip test passed!");
    }

    #[test]
    fn test_complex_repetition_structure() {
        println!("\n=== Complex Repetition Structure Test ===");
        
        let mut movement = Movement::new("Complex Movement".to_string());
        
        // Section A: 2 bars
        let mut section_a = Section::new("A".to_string());
        section_a.add_bar(Bar::new(1, 4, 4));
        section_a.add_bar(Bar::new(2, 4, 4));
        movement.add_section(section_a);
        
        // Section B: 2 bars
        let mut section_b = Section::new("B".to_string());
        section_b.add_bar(Bar::new(3, 4, 4));
        section_b.add_bar(Bar::new(4, 4, 4));
        movement.add_section(section_b);
        
        // Section C: 1 bar with complex repetition
        let mut section_c = Section::new("C".to_string());
        section_c.add_bar(Bar::new(5, 4, 4));
        
        // Repeat A and B together, three times, with identical ending pattern each time
        let repetition = Repetition::new(0, 1, 3)
            .with_individual_ending_pattern(vec![
                Bar::new(1, 3, 4), // Different time signature for ending
            ]);
        section_c.add_repetition(repetition);
        movement.add_section(section_c);
        
        println!("Movement structure:");
        println!("- Section A: 2 bars");
        println!("- Section B: 2 bars"); 
        println!("- Section C: 1 bar + (A+B)*3 + ending*3");
        println!("- Total expected: 2 + 2 + 1 + (2+2)*3 + 1*3 = 20 bars");
        
        let total_bars = movement.total_bar_count();
        println!("Calculated total: {} bars", total_bars);
        assert_eq!(total_bars, 20);
        
        let sequence = movement.generate_bar_sequence();
        assert_eq!(sequence.len(), 20);
        
        // Verify repetition bar numbering restarts
        let repetition_bars: Vec<_> = sequence.iter()
            .filter(|b| b.is_repetition)
            .collect();
        
        println!("Repetition bars analysis:");
        for (i, bar) in repetition_bars.iter().enumerate() {
            println!("  Rep bar {:2}: Logical #{}, Internal #{}, Section: {}", 
                    i + 1,
                    bar.logical_bar_number,
                    bar.original_bar.number,
                    bar.section_name);
        }
        
        // Check that each repetition cycle restarts bar numbering
        let first_repetition_bars: Vec<_> = repetition_bars.iter()
            .filter(|b| !b.is_individual_ending)
            .take(4) // First repetition: 4 bars (A+B)
            .collect();
        
        // In each repetition, bar numbers should restart from 1
        assert_eq!(first_repetition_bars[0].original_bar.number, 1); // A.1
        assert_eq!(first_repetition_bars[1].original_bar.number, 2); // A.2
        assert_eq!(first_repetition_bars[2].original_bar.number, 3); // B.1 
        assert_eq!(first_repetition_bars[3].original_bar.number, 4); // B.2
        
        println!("✅ Complex repetition structure test passed!");
    }

    #[test]
    fn test_tempo_inheritance() {
        println!("\n=== Tempo Inheritance Test ===");
        
        let mut movement = Movement::new("Tempo Inheritance Movement".to_string()).with_tempo(120.0);
        
        // Section A: inherits movement tempo (no own tempo)
        let mut section_a = Section::new("Section A (inherits)".to_string());
        section_a.add_bar(Bar::new(1, 4, 4));
        section_a.add_bar(Bar::new(2, 4, 4));
        movement.add_section(section_a);
        
        // Section B: overrides with own tempo
        let mut section_b = Section::new("Section B (overrides)".to_string()).with_tempo(96.0);
        section_b.add_bar(Bar::new(3, 4, 4));
        section_b.add_bar(Bar::new(4, 4, 4));
        movement.add_section(section_b);
        
        // Section C: no tempo, inherits from movement
        let mut section_c = Section::new("Section C (inherits)".to_string());
        section_c.add_bar(Bar::new(5, 3, 4));
        movement.add_section(section_c);
        
        println!("Movement tempo: {:?} BPM", movement.tempo_bpm);
        
        let section_tempos = movement.get_section_tempos();
        println!("Section effective tempos:");
        for (idx, &tempo) in section_tempos.iter().enumerate() {
            let section = &movement.sections[idx];
            let effective_str = match tempo {
                Some(t) => format!("{:.1} BPM", t),
                None => "None".to_string(),
            };
            let source = if section.tempo_bpm.is_some() { "own" } else { "inherited" };
            println!("  Section {}: {} ({} tempo)", idx + 1, effective_str, source);
        }
        
        // Verify tempo inheritance
        assert_eq!(section_tempos[0], Some(120.0)); // Section A inherits
        assert_eq!(section_tempos[1], Some(96.0));  // Section B overrides
        assert_eq!(section_tempos[2], Some(120.0)); // Section C inherits
        
        // Test individual section effective tempo method
        assert_eq!(movement.sections[0].effective_tempo(movement.tempo_bpm), Some(120.0));
        assert_eq!(movement.sections[1].effective_tempo(movement.tempo_bpm), Some(96.0));
        assert_eq!(movement.sections[2].effective_tempo(movement.tempo_bpm), Some(120.0));
        
        // Test with no movement tempo
        let mut movement_no_tempo = Movement::new("No Tempo Movement".to_string());
        let mut section_no_tempo = Section::new("No Tempo Section".to_string());
        section_no_tempo.add_bar(Bar::new(1, 4, 4));
        movement_no_tempo.add_section(section_no_tempo);
        
        let no_tempo_result = movement_no_tempo.get_section_tempos();
        assert_eq!(no_tempo_result[0], None); // Both movement and section have no tempo
        
        println!("✅ Tempo inheritance test passed!");
    }
}
