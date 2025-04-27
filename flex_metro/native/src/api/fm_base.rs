use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::fs;
use std::path::Path;
use yaml_rust::{Yaml, YamlLoader};


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
    println!("Current working directory: {:?}", std::env::current_dir().unwrap());

    let config_path = Path::new("./native/src/api/cfg/beats.yaml");
    let config_str = fs::read_to_string(config_path).expect("Failed to read beats.yaml");
    let docs = YamlLoader::load_from_str(&config_str).expect("Failed to parse beats.yaml");
    let config = docs[0].clone();

    let mut beats = HashMap::new();
    if let Some(hash) = config.as_hash() {
        for (key, value) in hash {
            let num_denom = key.as_str().expect("Failed to convert key to string");
            let (num_str, denom_str) = num_denom.split_once('/').expect("Failed to split key into num and denom");
            let num: i32 = num_str.trim().parse().expect("Failed to parse num");
            let denom: i32 = denom_str.trim().parse().expect("Failed to parse denom");
    
            let metric_key = MetricKey::Standard((num, denom));
            let beat_values = value.as_vec().expect("Failed to convert value to vec").iter().map(|x| x.as_i64().expect("Failed to convert beat value to i64") as i32).collect();
    
            beats.insert(metric_key, beat_values);
        }
    }

    beats
});

fn linspace(start: f64, end: f64, num: usize) -> Vec<f64> {
    if num == 0 {
        return vec![];
    }
    if num == 1 {
        return vec![start];
    }

    let step = (end - start) / (num - 1) as f64;
    (0..num).map(|i| start + i as f64 * step).collect()
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
        let failed_beats = vec![
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
