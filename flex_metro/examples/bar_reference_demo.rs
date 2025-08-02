// Demo: Using BarReference for efficient bar lookup
// This example shows how to use the BarReference class to quickly find
// which movement, section, and repetition a specific bar belongs to

// Note: This would typically be: use rust_lib_flex_metro::api::fm_composition::*;
// For internal development, we use relative paths:
use flex_metro::api::fm_composition::*;

fn main() {
    // Create a simple composition with repetitions
    let mut composition = Composition::new(
        "Demo Composer".to_string(),
        "Reference Demo".to_string(),
    );

    let mut movement = Movement::new("I. Demo Movement".to_string()).with_tempo(120.0);
    
    // Section A: 3 bars
    let mut section_a = Section::new("Theme A".to_string());
    for i in 1..=3 {
        section_a.add_bar(Bar::new(i, 4, 4));
    }
    movement.add_section(section_a);
    
    // Section B: 2 bars with repetition of A
    let mut section_b = Section::new("Theme B".to_string());
    section_b.add_bar(Bar::new(4, 4, 4));
    section_b.add_bar(Bar::new(5, 4, 4));
    
    // Add repetition: repeat A twice with individual endings
    let repetition = Repetition::new(0, 0, 2)
        .with_individual_ending_pattern(vec![
            Bar::new(1, 4, 4).with_name("ending 1".to_string()),
            Bar::new(2, 4, 4).with_name("ending 2".to_string()),
        ]);
    section_b.add_repetition(repetition);
    
    movement.add_section(section_b);
    composition.add_movement(movement);
    
    // Create BarReference for efficient lookups
    let bar_ref = BarReference::new(&composition);
    
    println!("=== Bar Reference Demo ===");
    println!("Total bars in composition: {}", bar_ref.total_bars());
    println!();
    
    // Demonstrate lookups for different types of bars
    let test_bars = vec![1, 3, 6, 8, 10, 13];
    
    for bar_num in test_bars {
        if let Some(ref_info) = bar_ref.get_bar_reference(bar_num) {
            println!("Bar {}: {}", bar_num, format_bar_info(ref_info));
        } else {
            println!("Bar {}: Not found", bar_num);
        }
    }
    
    println!();
    println!("=== Summary Queries ===");
    
    // Show movement-wide analysis
    let movement_bars = bar_ref.get_bars_in_movement(0);
    println!("Movement 0 has {} bars", movement_bars.len());
    
    // Show section-specific analysis
    let section_a_bars = bar_ref.get_bars_in_section(0, 0);
    println!("Section A appears in {} bars (including repetitions)", section_a_bars.len());
    
    // Show repetition analysis
    let repetition_bars = bar_ref.get_repetition_bars();
    println!("Total repetition bars: {}", repetition_bars.len());
    
    let ending_bars = bar_ref.get_individual_ending_bars();
    println!("Individual ending bars: {}", ending_bars.len());
    
    println!("\n✅ BarReference provides O(log n) lookup performance!");
}

fn format_bar_info(info: &BarReferenceInfo) -> String {
    let mut result = format!("Movement '{}', Section '{}'", 
                           info.movement_name, info.section_name);
    
    if let Some(rep_idx) = info.repetition_index {
        if info.is_individual_ending {
            result.push_str(&format!(" (Individual Ending #{})", rep_idx + 1));
        } else {
            result.push_str(&format!(" (Repetition #{})", rep_idx + 1));
        }
    }
    
    result
}
