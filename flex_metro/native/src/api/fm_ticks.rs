use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
use serde::de::{self, Visitor, SeqAccess};
use yaml_rust::YamlLoader;
use std::fmt;
use std::path::Path;
use crate::api::fm_base::{DEFAULT_BEATS, MetricKey};


enum FMTickPositions {
    None = -1,
    Major = 0,
    Medium,
    Minor
}


type MetricDict = HashMap<MetricKey, Vec<i32>>;

impl<'de> Deserialize<'de> for MetricKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MetricKeyVisitor;

        impl<'de> Visitor<'de> for MetricKeyVisitor {
            type Value = MetricKey;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tuple (i32, i32) or a tuple (Vec<i32>, i32)")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec_key = vec![];
                while let Some(element) = seq.next_element()? {
                    vec_key.push(element);
                }
                if vec_key.len() < 2 {
                    return Err(de::Error::invalid_length(vec_key.len(), &self));
                }
                let last = vec_key.pop().unwrap();
                if ![1, 2, 4, 8, 16, 32, 64, 128, 256].contains(&last) {
                    return Err(de::Error::custom("The last element must be a power of two less than 512"));
                }
                if vec_key.len() == 1 {
                    Ok(MetricKey::Standard((vec_key[0], last)))
                } else {
                    Ok(MetricKey::Composite((vec_key, last)))
                }
            }
        }

        deserializer.deserialize_seq(MetricKeyVisitor)
    }
}

pub fn load_ticks(cfg_path: Option<String>) -> Result<HashMap<MetricKey, Vec<i32>>, String> {
    let mut ticks: HashMap<MetricKey, Vec<i32>> = DEFAULT_BEATS.clone();

    let cfg_file = match cfg_path {
        Some(path) => Path::new(&path).to_path_buf(),
        None => Path::new("../../../../cfg/metrum_default_ticks.yaml").to_path_buf(),
    };

    let file_content = std::fs::read_to_string(cfg_file).map_err(|e| e.to_string())?;
    let docs = YamlLoader::load_from_str(&file_content).map_err(|e| e.to_string())?;

    for doc in &docs {
        if let Some(entries) = doc.as_vec() {
            for entry in entries {
                if let (Some(key), Some(value)) = (entry["key"].as_vec(), entry["value"].as_vec()) {
                    /*
                    The reason for using as_i64 and then unwrapping it as i32 is due to the way the yaml_rust library handles numeric values. 
                    The as_i64 method is used to safely extract an integer value from a YAML node, which is internally represented as an i64. 
                    After extracting the value as an i64, it is then cast to an i32 to match the expected type in the MetricKey
                     */

                    let key: MetricKey = if key.iter().all(|v| v.as_i64().is_some()) {
                        if let [first, second] = &key[..] {
                            MetricKey::Standard((first.as_i64().unwrap() as i32, second.as_i64().unwrap() as i32))
                        } else {
                            let vec_key: Vec<i32> = key.iter().map(|v| v.as_i64().unwrap() as i32).collect();
                            let last = vec_key.last().copied().unwrap();
                            MetricKey::Composite((vec_key, last))
                        }
                    } else {
                        return Err("Invalid key format".to_string());
                    };

                    let value: Vec<i32> = value.iter().map(|v| v.as_i64().unwrap() as i32).collect();

                    match key {
                        MetricKey::Composite((vec_key, last)) => {
                            ticks.insert(MetricKey::Composite((vec_key.clone(), last)), vec_key);
                        }
                        MetricKey::Standard((first, second)) => {
                            if value.iter().sum::<i32>() == first {
                                ticks.insert(MetricKey::Standard((first, second)), value);
                            } else {
                                return Err(format!("The sum of the integers in the value must be equal to the first element of the key: {:?}", key));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(ticks)
}

// -- unit test --

// -- unit test --

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_load_ticks() {
        let cfg_path = Some(String::from("path/to/config"));
        let ticks = load_ticks(cfg_path);

        // Assuming the YAML file contains the following data:
        // - key: [1, 2]
        //   value: [3, 4, 5]
        // - key: [[1, 2, 3], 4]
        //   value: [8, 9, 10]
        match ticks {
            Ok(ticks) => {
                let mut expected_ticks = HashMap::new();
                expected_ticks.insert(MetricKey::Standard((1, 2)), vec![1]);
                expected_ticks.insert(MetricKey::Composite((vec![1, 2, 3], 4)), vec![1, 2, 3]);
            }
            Err(err) => {
                panic!("Error loading ticks: {}", err);
            }
        }
    }
}
