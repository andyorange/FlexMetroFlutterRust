use crate::api::fm_base::{DEFAULT_BEATS, MetricKey};

#[derive(Debug, Clone, PartialEq)]
pub enum BarBeatData {
    Int(Vec<i32>),
    Float(Vec<f32>),
}

#[derive(Debug, Clone)]
pub struct FMBarElement {
    pub nom: i32,
    pub denom: i32,
    pub nom_secs: f32,  // either nom/denom is set or nom_secs and denom=nom=0
    pub beats: BarBeatData,  // int for time signature based (nom, denom > 0) measure, float otherwise
    pub has_signature: bool,
    pub sub_beats: BarBeatData, // int if signature based, float else
}

impl FMBarElement {
    pub fn new(nom: i32, denom: i32, nom_secs: f32, beats: Option<Vec<i32>>) -> Self {
        assert!([0, 1, 2, 4, 8, 16, 32].contains(&denom));

        let has_signature = (denom > 0) && (nom > 0);
        
        let sub_beats = if has_signature {
            BarBeatData::Int(vec![0])
        } else {
            BarBeatData::Float(vec![0.0])
        };

        let beats = if has_signature {
            // Handle Option<Vec<i32>> properly
            match beats {
                None => {
                    // Use default beats when None
                    let key = MetricKey::Standard((nom, denom));
                    let default_beats = match DEFAULT_BEATS.get(&key) {
                        Some(beats) => beats,
                        None => {
                            eprintln!("Warning: No default beats found for {}/{}, using simple subdivision", nom, denom);
                            // Fallback to simple even subdivision
                            &vec![1; nom as usize]
                        }
                    };
                    BarBeatData::Int(default_beats.iter().map(|&b| b as i32).collect())
                },
                Some(beats_vec) if beats_vec.is_empty() => {
                    // Use default beats when empty
                    let key = MetricKey::Standard((nom, denom));
                    let default_beats = match DEFAULT_BEATS.get(&key) {
                        Some(beats) => beats,
                        None => {
                            eprintln!("Warning: No default beats found for {}/{}, using simple subdivision", nom, denom);
                            // Fallback to simple even subdivision
                            &vec![1; nom as usize]
                        }
                    };
                    BarBeatData::Int(default_beats.iter().map(|&b| b as i32).collect())
                },
                Some(beats_vec) => {
                    // Validate that beats sum equals nom
                    let sum: i32 = beats_vec.iter().sum();
                    assert!(sum == nom, "Beats sum ({}) must equal nom ({})", sum, nom);
                    BarBeatData::Int(beats_vec)
                }
            }
        } else {
            // For non-signature based (nom_secs > 0), convert to float
            assert!(nom_secs > 0.0, "nom_secs must be positive for non-signature based measures");
            match beats {
                None => BarBeatData::Float(vec![]),
                Some(beats_vec) => {
                    assert!(!beats_vec.is_empty(), "beats cannot be empty for non-signature based measures");
                    // Convert i32 to f32 properly
                    let float_beats: Vec<f32> = beats_vec.into_iter().map(|b| b as f32).collect();
                    BarBeatData::Float(float_beats)
                }
            }
        };

        use crate::api::fm_base::{DEFAULT_BEATS, MetricKey};

#[derive(Debug, Clone, PartialEq)]
pub enum BarBeatData {
    Int(Vec<i32>),
    Float(Vec<f32>),
}

#[derive(Debug, Clone)]
pub struct FMBarElement {
    pub nom: i32,
    pub denom: i32,
    pub nom_secs: f32,  // either nom/denom is set or nom_secs and denom=nom=0
    pub beats: BarBeatData,  // int for time signature based (nom, denom > 0) measure, float otherwise
    pub has_signature: bool,
    pub sub_beats: BarBeatData, // int if signature based, float else
}

impl FMBarElement {
    pub fn new(nom: i32, denom: i32, nom_secs: f32, beats: Option<Vec<i32>>) -> Self {
        assert!([0, 1, 2, 4, 8, 16, 32].contains(&denom));

        let has_signature = (denom > 0) && (nom > 0);
        
        let sub_beats = if has_signature {
            BarBeatData::Int(vec![0])
        } else {
            BarBeatData::Float(vec![0.0])
        };

        let beats = if has_signature {
            // Handle Option<Vec<i32>> properly
            match beats {
                None => {
                    // Use default beats when None
                    let key = MetricKey::Standard((nom, denom));
                    let default_beats = match DEFAULT_BEATS.get(&key) {
                        Some(beats) => beats,
                        None => {
                            eprintln!("Warning: No default beats found for {}/{}, using simple subdivision", nom, denom);
                            // Fallback to simple even subdivision
                            &vec![1; nom as usize]
                        }
                    };
                    BarBeatData::Int(default_beats.iter().map(|&b| b as i32).collect())
                },
                Some(beats_vec) if beats_vec.is_empty() => {
                    // Use default beats when empty
                    let key = MetricKey::Standard((nom, denom));
                    let default_beats = match DEFAULT_BEATS.get(&key) {
                        Some(beats) => beats,
                        None => {
                            eprintln!("Warning: No default beats found for {}/{}, using simple subdivision", nom, denom);
                            // Fallback to simple even subdivision
                            &vec![1; nom as usize]
                        }
                    };
                    BarBeatData::Int(default_beats.iter().map(|&b| b as i32).collect())
                },
                Some(beats_vec) => {
                    // Validate that beats sum equals nom
                    let sum: i32 = beats_vec.iter().sum();
                    assert!(sum == nom, "Beats sum ({}) must equal nom ({})", sum, nom);
                    BarBeatData::Int(beats_vec)
                }
            }
        } else {
            // For non-signature based (nom_secs > 0), convert to float
            assert!(nom_secs > 0.0, "nom_secs must be positive for non-signature based measures");
            match beats {
                None => BarBeatData::Float(vec![]),
                Some(beats_vec) => {
                    assert!(!beats_vec.is_empty(), "beats cannot be empty for non-signature based measures");
                    // Convert i32 to f32 properly
                    let float_beats: Vec<f32> = beats_vec.into_iter().map(|b| b as f32).collect();
                    BarBeatData::Float(float_beats)
                }
            }
        };

        FMBarElement {
            nom,
            denom,
            nom_secs,
            beats,
            has_signature,
            sub_beats,
        }
    }

    pub fn add_sub_beat(&mut self, value: f32) {
        match &mut self.sub_beats {
            BarBeatData::Int(vec) => vec.push(value as i32),
            BarBeatData::Float(vec) => vec.push(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fm_bar_element_creation() {
        let beats = vec![1, 2, 1];  // Sum = 4, matches nom
        let mut bar_element = FMBarElement::new(4, 4, 0.0, Some(beats));
        bar_element.add_sub_beat(1.5);
        
        assert_eq!(bar_element.nom, 4);
        assert_eq!(bar_element.denom, 4);
        assert!(bar_element.has_signature);
        
        println!("FMBarElement created: {:?}", bar_element);
    }

    #[test]
    fn test_fm_bar_element_with_none_beats() {
        let bar_element = FMBarElement::new(4, 4, 0.0, None);
        
        assert_eq!(bar_element.nom, 4);
        assert_eq!(bar_element.denom, 4);
        assert!(bar_element.has_signature);
        
        // Should use default beats
        match &bar_element.beats {
            BarBeatData::Int(beats) => assert!(!beats.is_empty()),
            _ => panic!("Expected Int beats for signature-based measure"),
        }
    }

    #[test]
    fn test_fm_bar_element_non_signature() {
        let beats = vec![1, 2, 3];
        let bar_element = FMBarElement::new(0, 0, 2.5, Some(beats));
        
        assert_eq!(bar_element.nom, 0);
        assert_eq!(bar_element.denom, 0);
        assert!(!bar_element.has_signature);
        assert_eq!(bar_element.nom_secs, 2.5);
        
        // Should convert to float beats
        match &bar_element.beats {
            BarBeatData::Float(beats) => {
                assert_eq!(beats.len(), 3);
                assert_eq!(beats[0], 1.0);
                assert_eq!(beats[1], 2.0);
                assert_eq!(beats[2], 3.0);
            },
            _ => panic!("Expected Float beats for non-signature-based measure"),
        }
    }
}
    }

    pub fn add_sub_beat(&mut self, value: f32) {
        match &mut self.sub_beats {
            BarBeatData::Int(vec) => vec.push(value as i32),
            BarBeatData::Float(vec) => vec.push(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fm_bar_element_creation() {
        let beats = vec![1, 2, 1];  // Sum = 4, matches nom
        let mut bar_element = FMBarElement::new(4, 4, 0.0, Some(beats));
        bar_element.add_sub_beat(1.5);
        
        assert_eq!(bar_element.nom, 4);
        assert_eq!(bar_element.denom, 4);
        assert!(bar_element.has_signature);
        
        println!("FMBarElement created: {:?}", bar_element);
    }

    #[test]
    fn test_fm_bar_element_with_none_beats() {
        let bar_element = FMBarElement::new(4, 4, 0.0, None);
        
        assert_eq!(bar_element.nom, 4);
        assert_eq!(bar_element.denom, 4);
        assert!(bar_element.has_signature);
        
        // Should use default beats
        match &bar_element.beats {
            BarBeatData::Int(beats) => assert!(!beats.is_empty()),
            _ => panic!("Expected Int beats for signature-based measure"),
        }
    }

    #[test]
    fn test_fm_bar_element_non_signature() {
        let beats = vec![1, 2, 3];
        let bar_element = FMBarElement::new(0, 0, 2.5, Some(beats));
        
        assert_eq!(bar_element.nom, 0);
        assert_eq!(bar_element.denom, 0);
        assert!(!bar_element.has_signature);
        assert_eq!(bar_element.nom_secs, 2.5);
        
        // Should convert to float beats
        match &bar_element.beats {
            BarBeatData::Float(beats) => {
                assert_eq!(beats.len(), 3);
                assert_eq!(beats[0], 1.0);
                assert_eq!(beats[1], 2.0);
                assert_eq!(beats[2], 3.0);
            },
            _ => panic!("Expected Float beats for non-signature-based measure"),
        }
    }
}
