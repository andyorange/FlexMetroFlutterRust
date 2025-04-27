use crate::api::fm_bar_element::FMBarElement;


pub struct FMSection {
    pub name: String,
    pub bars: FMBarElement,
    pub num_bars: u32,
    pub start: Option<u32>,
    pub end: Option<u32> // int if signature based, float else
}

impl FMSection {
    pub fn new(name: String, bars: FMBarElement, num_bars: u32, start: Option<u32>) -> Self {
        let end = start.map(|s| s + num_bars - 1);
        FMSection {
            name, bars, num_bars, start, end
        }
    }

    pub fn update(&mut self, num_bars: u32) {
        self.num_bars = num_bars;
        self.end = self.start.map(|start| start + num_bars - 1);
    }

    pub fn len(&self) -> usize {
        self.num_bars
    }
}

pub struct FMSectionDict {
    pub name: String,
    pub section: FMSection
}

impl FMSectionDict {
    pub fn new(name: String, section: FMSection) -> Self {
        FMSectionDict {
            name, section
        }
    }
}
