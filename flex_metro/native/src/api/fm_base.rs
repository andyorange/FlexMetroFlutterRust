use std::collections::HashMap;
use once_cell::sync::Lazy;


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
    let mut m = HashMap::new();
    m.insert(MetricKey::Standard((3, 4)), vec![3]);
    m.insert(MetricKey::Standard((3, 8)), vec![3]);
    m.insert(MetricKey::Standard((2, 4)), vec![2]);
    m.insert(MetricKey::Standard((4, 4)), vec![2, 2]);
    m.insert(MetricKey::Standard((5, 8)), vec![3, 2]);
    m.insert(MetricKey::Standard((5, 4)), vec![3, 2]);
    m.insert(MetricKey::Standard((6, 8)), vec![3, 3]);
    m.insert(MetricKey::Standard((7, 4)), vec![3, 2, 2]);
    m.insert(MetricKey::Standard((8, 8)), vec![3, 3, 2]);
    m.insert(MetricKey::Standard((6, 4)), vec![3, 3]);
    m.insert(MetricKey::Standard((3, 2)), vec![3]);
    m
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

