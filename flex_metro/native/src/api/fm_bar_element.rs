use crate::api::fm_base::{DEFAULT_BEATS, MetricKey};


pub enum BeatType {
    Int(Vec<i32>),
    Float(Vec<f32>),
}

pub struct FMBarElement {
    pub nom: i32,
    pub denom: i32,
    pub nom_secs: f32,  // either nom/denom is set or nom_secs and denom=nom=0
    pub tempo_bar_start: i32,
    pub tempo_bar_end: i32,
    pub base_beat: i32,
    pub beats: BeatType,  // int for time signature based (nom, denom > 0) measure, float otherwise
    pub has_signature: bool,
    //pub sub_beats: BeatType, // int if signature based, float else
}

impl FMBarElement {
    pub fn new(nom: i32, denom: i32, nom_secs: f32,
               tempo_bar_start: i32, tempo_bar_end: i32, base_beat: i32,
               beats: Option<Vec<i32>>) -> Self {
        assert!([0, 1, 2, 4, 8, 16, 32].contains(&denom));

        let has_signature = (denom > 0) && (nom > 0);
        /*
        let sub_beats = if has_signature {
            BeatType::Int(vec![0])
        } else {
            BeatType::Float(vec![0.0])
        };
        */

        let beats = if has_signature {
            if beats.is_empty() {
                let key = MetricKey::Standard((nom, denom));
                let default_beats = DEFAULT_BEATS.get(&key)
                    .expect("DEFAULT_BEATS should contain the key");
                BeatType::Int(default_beats.iter().map(|&b| b as i32).collect())
            } else {
                assert!(beats.iter().sum::<i32>() == nom);
                BeatType::Int(beats)
            }
        } else {
            assert!(nom_secs > 0.0 && !beats.is_empty());
            BeatType::Float(beats.into_iter().map(|b| b as f32).collect())
        };

        FMBarElement {
            nom,
            denom,
            nom_secs,
            tempo_bar_start,
            tempo_bar_end,
            base_beat,
            beats,
            has_signature,
            //sub_beats,
        }
    }

    pub fn add_sub_beat(&mut self, value: f32) {
        match &mut self.sub_beats {
            BeatType::Int(vec) => vec.push(value as i32),
            BeatType::Float(vec) => vec.push(value),
        }
    }
}

fn main() {
    let beats = vec![1, 2, 3];
    let mut bar_element = FMBarElement::new(4, 4, 0.0, 120, 120, beats);
    bar_element.add_sub_beat(1.5);
    println!("FMBarElement created: {:?}", bar_element);
}
