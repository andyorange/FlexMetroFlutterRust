use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::fs;
use std::path::Path;
use yaml_rust::{YamlLoader};


#[derive(Debug, PartialEq, Eq, Hash, Clone)] 
pub enum MetricKey {
    Standard((i32, i32)),
    Composite((Vec<i32>, i32)),
}

impl From<(i32, i32)> for MetricKey {
    fn from(pair: (i32, i32)) -> Self {
        MetricKey::Standard(pair)
    }
}

impl From<(Vec<i32>, i32)> for MetricKey {
    fn from(pair: (Vec<i32>, i32)) -> Self {
        MetricKey::Composite(pair)
    }
}

pub static DEFAULT_BEATS: Lazy<HashMap<MetricKey, Vec<i32>>> = Lazy::new(|| {
    match std::env::current_dir() {
        Ok(dir) => println!("Current working directory: {:?}", dir),
        Err(e) => eprintln!("Failed to get current directory: {}", e),
    }

    let config_path = Path::new("./native/src/api/cfg/beats.yaml");
    let config_str = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read beats.yaml: {}", e);
            return HashMap::new(); // Return empty HashMap on error
        }
    };
    
    let docs = match YamlLoader::load_from_str(&config_str) {
        Ok(docs) => docs,
        Err(e) => {
            eprintln!("Failed to parse beats.yaml: {}", e);
            return HashMap::new(); // Return empty HashMap on error
        }
    };
    
    if docs.is_empty() {
        eprintln!("No documents found in beats.yaml");
        return HashMap::new();
    }
    
    let config = docs[0].clone();

    let mut beats = HashMap::new();
    if let Some(hash) = config.as_hash() {
        for (key, value) in hash {
            let processing_result = (|| -> Result<(), String> {
                let num_denom = key.as_str().ok_or("Failed to convert key to string")?;
                let (num_str, denom_str) = num_denom.split_once('/').ok_or("Failed to split key into num and denom")?;
                let num: i32 = num_str.trim().parse()
                    .map_err(|e| format!("Failed to parse numerator '{}': {}", num_str, e))?;
                let denom: i32 = denom_str.trim().parse()
                    .map_err(|e| format!("Failed to parse denominator '{}': {}", denom_str, e))?;
        
                let metric_key = MetricKey::Standard((num, denom));
                let value_vec = value.as_vec().ok_or("Failed to convert value to vec")?;
                let beat_values: Result<Vec<i32>, String> = value_vec.iter().map(|x| {
                    x.as_i64().ok_or_else(|| "Failed to convert beat value to i64".to_string()).map(|v| v as i32)
                }).collect();
        
                beats.insert(metric_key, beat_values?);
                Ok(())
            })();
            
            if let Err(e) = processing_result {
                eprintln!("Error processing beat configuration entry: {}", e);
            }
        }
    }

    beats
});


/// Load beat configuration from YAML file
pub fn load_beat_config() -> Result<HashMap<MetricKey, Vec<i32>>, String> {
    let config_path = Path::new("cfg/metrum_default_ticks.yaml");
    let config_str = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    let docs = YamlLoader::load_from_str(&config_str)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;
    let config = docs[0].clone();

    let mut beats = HashMap::new();
    if let Some(hash) = config.as_hash() {
        for (key, value) in hash {
            let num_denom = key.as_str().ok_or("Failed to convert key to string")?;
            let (num_str, denom_str) = num_denom.split_once('/').ok_or("Failed to split key into num and denom")?;
            let num: i32 = num_str.trim().parse()
                .map_err(|e| format!("Failed to parse numerator '{}': {}", num_str, e))?;
            let denom: i32 = denom_str.trim().parse()
                .map_err(|e| format!("Failed to parse denominator '{}': {}", denom_str, e))?;
    
            let metric_key = MetricKey::Standard((num, denom));
            
            let beat_values = value.as_vec()
                .ok_or("Failed to convert value to vec")?
                .iter()
                .map(|x| x.as_i64()
                    .ok_or_else(|| "Failed to convert beat value to i64".to_string())
                    .map(|v| v as i32))
                .collect::<Result<Vec<i32>, String>>()?;
            
            beats.insert(metric_key, beat_values);
        }
    }

    Ok(beats)
}

// -- unit tests --

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_beats() {
        let expected_beats = vec![
            (MetricKey::Standard((3, 4)), vec![3]),
            (MetricKey::Standard((3, 8)), vec![3]),
            (MetricKey::Standard((2, 4)), vec![2]),
            (MetricKey::Standard((4, 4)), vec![2, 2]),
            (MetricKey::Standard((5, 8)), vec![3, 2]),
            (MetricKey::Standard((5, 4)), vec![3, 2]),
            (MetricKey::Standard((6, 8)), vec![3, 3]),
            (MetricKey::Standard((7, 4)), vec![3, 2, 2]),
            (MetricKey::Standard((8, 8)), vec![3, 3, 2]),
            (MetricKey::Standard((3, 2)), vec![3]),
        ];
        let _failed_beats = vec![
            (MetricKey::Standard((6, 11)), vec![3, 3]),
            (MetricKey::Standard((5, 4)), vec![2, 3])
        ];

        /*
        It's a bit of a subtle distinction, but unwrap is essentially "unwrapping" the Lazy wrapper around the HashMap instance,
        allowing us to use it as a regular HashMap instance.
        */ 
        // for (metric_key, expected_beats) in expected_beats.iter() {
        //     if let Some(beats_value) = DEFAULT_BEATS.get(metric_key) {
        //         assert_eq!(beats_value, expected_beats);
        //     }
        // }
        for (metric_key, expected_beats) in expected_beats.iter() {
            println!("metric {:?} -> beats {:?}", metric_key, expected_beats);
            if let Some(beats_value) = DEFAULT_BEATS.get(metric_key) {
                assert_eq!(beats_value, expected_beats);
            } else {
                panic!("MetricKey {:?} not found in or incorrect beat {:?}", metric_key, expected_beats);
            }
        }
    }
}
