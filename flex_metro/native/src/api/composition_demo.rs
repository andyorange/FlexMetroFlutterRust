// Demonstration program for the Composition structure
// Shows how to load and analyze a musical composition from YAML

use crate::api::fm_composition_reader::CompositionReader;

pub fn demonstrate_composition_loading() {
    println!("=== Musical Composition Structure Demonstration ===\n");
    
    // Example 1: Load from the Beethoven YAML file
    let beethoven_path = "examples/beethoven_symphony_5.yaml";
    
    match CompositionReader::read_from_file(beethoven_path) {
        Ok(composition) => {
            analyze_composition(&composition);
        },
        Err(e) => {
            println!("Could not load Beethoven example: {}", e);
            println!("Creating a programmatic example instead...\n");
            demonstrate_programmatic_composition();
        }
    }
}

fn analyze_composition(composition: &crate::api::fm_composition::Composition) {
    println!("📜 Composition Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎼 Composer: {}", composition.composer_name);
    println!("🎵 Work: {}", composition.work_name);
    println!("📝 Movements: {}\n", composition.movement_count());
    
    let total_bars = composition.total_bar_count();
    println!("📊 Total bars across all movements: {}\n", total_bars);
    
    // Analyze each movement
    for (i, movement) in composition.movements.iter().enumerate() {
        println!("🎼 Movement {}: {}", i + 1, movement.name);
        
        let movement_tempo_str = movement.tempo_bpm
            .map(|t| format!("{:.1} BPM", t))
            .unwrap_or_else(|| "No tempo".to_string());
        println!("   Movement tempo: {}", movement_tempo_str);
        println!("   Sections: {}", movement.sections.len());
        println!("   Total bars: {}", movement.total_bar_count());
        
        let section_tempos = movement.get_section_tempos();
        
        // Show section breakdown
        for (j, section) in movement.sections.iter().enumerate() {
            let effective_tempo = section_tempos[j];
            let tempo_str = effective_tempo
                .map(|t| format!("{:.1} BPM", t))
                .unwrap_or_else(|| "No tempo".to_string());
            let tempo_source = if section.tempo_bpm.is_some() { "own" } else { "inherited" };
            
            let rep_info = if section.repetitions.is_empty() {
                String::new()
            } else {
                format!(" (+ {} repetitions)", section.repetitions.len())
            };
            println!("     §{}: {} - {} bars{} @ {} ({})", 
                    j + 1, section.name, section.base_bar_count(), rep_info, tempo_str, tempo_source);
            
            // Show repetition details
            for (k, repetition) in section.repetitions.iter().enumerate() {
                let ending_info = if !repetition.has_individual_endings() {
                    String::new()
                } else {
                    format!(" + {} ending bars", repetition.individual_ending_length())
                };
                println!("       Rep {}: Sections {}-{} × {}{}", 
                        k + 1, 
                        repetition.start_section + 1, 
                        repetition.end_section + 1, 
                        repetition.num_repetitions,
                        ending_info);
            }
        }
        
        // Generate and show a sample of the performance sequence
        let sequence = movement.generate_bar_sequence();
        println!("   Performance sequence preview (first 10 bars):");
        for (idx, bar) in sequence.iter().take(10).enumerate() {
            let rep_marker = if bar.is_repetition {
                if bar.is_individual_ending {
                    " [End]"
                } else {
                    " [Rep]"
                }
            } else {
                ""
            };
            
            let name_info = if let Some(ref name) = bar.original_bar.name {
                format!(" [{}]", name)
            } else {
                String::new()
            };
            
            println!("     {:2}. Bar {} | {}/{}{}{}", 
                    idx + 1,
                    bar.original_bar.number,
                    bar.original_bar.numerator, 
                    bar.original_bar.denominator,
                    name_info,
                    rep_marker);
        }
        
        if sequence.len() > 10 {
            println!("     ... ({} more bars)", sequence.len() - 10);
        }
        println!();
    }
}

fn demonstrate_programmatic_composition() {
    use crate::api::fm_composition::{Composition, Movement, Section, Bar, Repetition};
    
    println!("🔨 Creating Programmatic Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut composition = Composition::new(
        "Demo Composer".to_string(),
        "Sample Musical Form".to_string(),
    );

    let mut movement = Movement::new("I. Example in Classical Form".to_string()).with_tempo(120.0);
    
    // Theme A
    let mut theme_a = Section::new("Theme A".to_string());
    theme_a.add_bar(Bar::new(1, 4, 4).with_name("A".to_string()));
    theme_a.add_bar(Bar::new(2, 4, 4));
    theme_a.add_bar(Bar::new(3, 4, 4));
    theme_a.add_bar(Bar::new(4, 4, 4));
    movement.add_section(theme_a);
    
    // Theme B
    let mut theme_b = Section::new("Theme B".to_string()).with_tempo(100.0);
    theme_b.add_bar(Bar::new(5, 3, 4).with_name("B".to_string()));
    theme_b.add_bar(Bar::new(6, 3, 4));
    movement.add_section(theme_b);
    
    // Development with complex repetition
    let mut development = Section::new("Development".to_string());
    development.add_bar(Bar::new(7, 4, 4).with_name("Dev".to_string()));
    
    // Create an ABA form: repeat A, then B, then A again, with a coda pattern
    let repetition = Repetition::new(0, 1, 1) // A and B together, once
        .with_individual_ending_pattern(vec![
            Bar::new(1, 4, 4).with_name("Coda".to_string()),
            Bar::new(2, 4, 4),
        ]);
    development.add_repetition(repetition);
    movement.add_section(development);
    
    composition.add_movement(movement);
    
    analyze_composition(&composition);
    
    // Show YAML representation
    match CompositionReader::to_yaml_string(&composition) {
        Ok(yaml) => {
            println!("📄 YAML Representation:");
            println!("━━━━━━━━━━━━━━━━━━━━━━");
            println!("{}", yaml);
        },
        Err(e) => {
            println!("❌ Could not serialize to YAML: {}", e);
        }
    }
}

pub fn run_composition_demonstration() {
    demonstrate_composition_loading();
}
