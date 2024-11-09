// FMBase.rs
use std::collections::HashMap;
use serde::{Deserialize};
use yaml_rust::YamlLoader;
use env::os

#[derive(Debug, Deserialize)]
struct MetricTicks {
    key: (i32, i32),
    value: Vec<i32>,
}



/*
    nom: int|float
    denom: int  # = 0 -> nom
    tempo_bar_start: int
    tempo_bar_end: int
    base_beat: int  # base metronom beat.
    beats: Tuple[int] | Tuple[float] | None  # absolute timepoints if denom == 0
    time_beats: Tuple[float] | None = None   # list of major, medium, minor beats as exact timepoints. Set to None if denom > 0

    default_beats: ClassVar[dict] = {
        (3, 4): (0),
        (3, 8): (0),
        (2, 4): (2),
        (4, 4): (2, 2),
        (5, 8): (3, 2),
        (5, 4): (3, 2),
        (6, 8): (3, 3),
        (7, 4): (3, 2, 2),
        (8, 8): (3, 3, 2),
        (6, 4): (3, 3),
        (3, 2): (3)
    }
    
    def __post_init__(self):
        assert(self.denom in [0, 1, 2, 4, 8, 16, 32])
        self.nom = int(self.nom) if self.denom > 0 else self.nom
        if self.beats is None:
            if self.denom > 0:
                self.beats = FMBarElement.default_beats.get(tuple((self.nom, self.denom)), (0))
                
            else:
                self.beats= (0)
        else:
            self.beats = [int(beat) if self.denom > 0 else beat for beat in self.beats]
        self.subbeats = [0]
        if self.denom > 0:
            self.time_beats = None
            assert((self.beats == (0)) or (sum(self.beats) == self.nom))
            for beat in self.beats[0:-1]:
                if beat > 0:
                    self.subbeats.append(self.subbeats[-1] + beat)
            self.beat_tempo_factor = self.base_beat * 1.0 / self.nom
        else:
            # no tick outside time windows
            self.beats = [beat for beat in self.beats if beat < self.nom]
            self.subbeats = self.beats  # when time is given, beats are absolute time points and thus are the subbeats, actually
            self.beat_tempo_factor = 1
            assert(len(self.time_beats) == len(self.subbeats))
            assert(sum([1 for x in self.time_beats if x not in [FMTickPositions.T_major, FMTickPositions.T_medium, FMTickPositions.T_minor]]))

    
     */

pub final struct FMTicks {
    ticks: HashMap<MetricTicks>
}

impl FMTicks {
    pub fn new(cfg_path: Option<String>) -> Self {
        // Read the YAML file
        let cfg_file = os.path.join(cfg_path, "metrum_default_ticks.yaml");
        let file_content = std::fs::read_to_string(cfg_file.to_string()).expect("Unable to read file");
    
        // Parse the YAML content into a YamlLoader
        let docs = YamlLoader::load_from_str(&file_content).expect("Unable to parse YAML");
    
        // Deserialize the YAML content into a Vec<Data>
        let data: Vec<Data> = docs[0]
        .into_vec()
        .iter()
        .map(|doc| {
            let key = (
                doc["key"][0].as_i64().unwrap() as i32,
                doc["key"][1].as_i64().unwrap() as i32
            );
            let value = doc["value"].as_vec().unwrap().iter().map(|v| v.as_i64().unwrap() as i32).collect();
            Data { key, value }
        })
        .collect();

        // Create a HashMap from the deserialized data
        let mut Self.ticks: HashMap<MetricTicks> = HashMap::new();
        for entry in data {
            tuple_map.insert(entry.key, entry.value);
        }

        // Access values using tuples as keys
        if let Some(value) = Self.ticks.get(&(5, 8)) {
            println!("Values for key (5, 8): {:?}", value);
        }

        // Iterate over the HashMap
        for (key, value) in Self.ticks {
            println!("Key: {:?}, Values: {:?}", key, value);
        }
    }

pub struct FMBarElement {
    pub nom: i32,
    pub denom: i32,
    pub nom_secs: f32,
    pub tempo_bar_start: i32,
    pub tempo_bar_end: i32,
    pub base_beat: i32,
    pub beats: Vec<i32>,
    pub time_beats: Vec<f32>,
    pub default_beats: HashMap<MetricTicks>
}



impl FMBarElement {
    static const default_beats = FMBarElement::read_default_beats(os.path.join(cfg_dir, "metrum_default_ticks.yaml"));
    pub fn new(nom: i32, denom: i32, nom_secs: f32,
        tempo_bar_start: i32, tempo_bar_end: i32, base_beat: i32,
        beats: Vec<i32>, time_beats: Vec<f32>, ref default_beats) -> Self {
            FMBarElement {
                nom, denom, nom_secs,
                tempo_bar_start, tempo_bar_end,
                base_beat, beats, time_beats, default_beats
            }
            assert(self.denom in [0, 1, 2, 4, 8, 16, 32])
            if self.beats is None:
                if self.denom > 0:
                    self.beats = FMBarElement.default_beats.get(tuple((self.nom, self.denom)), (0))
                    
                else:
                    self.beats = (0)
            else:
                self.beats = [int(beat) if self.denom > 0 else beat for beat in self.beats]
            self.subbeats = mut Vec<i32>[0]
            if self.denom > 0:
                self.time_beats = None
                assert((self.beats == (0)) or (sum(self.beats) == self.nom))
                for beat in self.beats[0:-1]:
                    if beat > 0:
                        self.subbeats.append(self.subbeats[-1] + beat)
                self.beat_tempo_factor = self.base_beat * 1.0 / self.nom
            else:
                # no tick outside time windows
                self.beats = [beat for beat in self.beats if beat < self.nom]
                self.subbeats = self.beats  # when time is given, beats are absolute time points and thus are the subbeats, actually
                self.beat_tempo_factor = 1
                assert(len(self.time_beats) == len(self.subbeats))
                assert(sum([1 for x in self.time_beats if x not in [FMTickPositions.T_major, FMTickPositions.T_medium, FMTickPositions.T_minor]]))
    
        
    }



    // Add more methods as needed
}