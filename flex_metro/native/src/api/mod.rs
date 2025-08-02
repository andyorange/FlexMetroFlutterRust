// Temporarily disable Flutter bridge modules until we work on Flutter interface
// pub mod simple;
pub mod fm_base;
pub mod fm_ticker_base;       // Core timer functionality
pub mod fm_bar_element;       // Bar element structures  
pub mod fm_tempo_interval;    // Tempo interval and sequence structures
pub mod fm_composition;       // Musical composition structure (independent from tempo)
pub mod fm_composition_reader; // YAML reader for composition structure
pub mod composition_demo;     // Demonstration of composition functionality
pub mod usage_example;        // Example of how to use the imports
pub mod log_msg_handling;     // Logging macros and message handling
// Temporarily disable Flutter interface modules
// pub mod flutter_interface;    
// pub mod flutter_example;      
// pub mod flutter_demo;         
