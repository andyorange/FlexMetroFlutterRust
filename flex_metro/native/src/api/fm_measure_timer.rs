use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use log::error;
use crate::api::fm_section::FMSection;


pub struct FMMeasureTimer {
    stop: bool,
    cnt_section: usize,
    cnt_measure: usize,
    cnt_beat: usize,
    beat_idx: usize,
    pub sections: FMSection, // Assuming Sections is a struct defined elsewhere
    pub num_measures: usize,
    pub timers: Vec<Duration>,
    current_timer: Option<thread::JoinHandle<()>>,
}

impl FMMeasureTimer {
    pub fn new(sections: FMSection, num_measures: usize, timers: Vec<Duration>) -> Self {
        FMMeasureTimer {
            stop: false,
            cnt_section: 0,
            cnt_measure: 0,
            cnt_beat: 0,
            beat_idx: 0,
            sections,
            num_measures,
            timers,
            current_timer: None,
        }
    }

    pub fn run(&mut self) {
        // Implement the run logic here
    }

    pub fn start(&mut self) {
        let self_arc = Arc::new(Mutex::new(self));
        let self_clone = Arc::clone(&self_arc);

        self.current_timer = Some(thread::spawn(move || {
            let mut self_locked = self_clone.lock().unwrap();
            if let Err(exc) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self_locked.run();
            })) {
                error!("Exception in running periodic thread: {:?}", exc);
            }
            if !self_locked._stop {
                self_locked.schedule_timer();
            }
        }));
    }

    pub fn schedule_timer(&mut self) {
        if self.cnt_section >= self.sections.len() {
            self.cancel();
            return;
        }
        if self.cnt_measure >= self.num_measures {
            self.cnt_section += 1;
            self.cnt_measure = 0;
            return;
        }
        self.tick_start = self.resolve_beat();
        let self_arc = Arc::new(Mutex::new(self));
        let self_clone = Arc::clone(&self_arc);

        self.current_timer = Some(thread::spawn(move || {
            thread::sleep(self_clone.lock().unwrap().timers[self_clone.lock().unwrap().beat_idx]);
            self_clone.lock().unwrap().run();
        }));

        self.cnt_beat += 1;
        self.beat_idx += 1;
        if self.cnt_beat >= self.sections.nom {
            self.cnt_beat = 0;
            self.cnt_measure += 1;
        }
    }

    pub fn cancel(&mut self) {
        self.stop = true;
    }

    fn resolve_beat(&self) -> usize {
        // Implement the resolve_beat logic here
        0
    }
}

// -- unit tests --

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_fm_measure_timer_start() {
        let bar = FMBarElement {
            nom: 4, denom: 4,
            nom_secs: 0.0,
            tempo_bar_start: 0,
            tempo_bar_end: 0,
            base_beat: 0,
        };
        let sections = FMSection { nom: 4 };
        let timers = vec![Duration::from_secs(1); 4];
        let mut timer = FMMeasureTimer::new(sections, 2, timers);

        timer.start();

        assert_eq!(timer.cnt_beat, 1);
        assert_eq!(timer.beat_idx, 1);
        assert_eq!(timer.cnt_measure, 0);
        assert!(!timer.stop);
    }

    #[test]
    fn test_fm_measure_timer_cancel() {
        let sections = FMSection { nom: 4 };
        let timers = vec![Duration::from_secs(1); 4];
        let mut timer = FMMeasureTimer::new(sections, 2, timers);

        timer.cancel();

        assert!(timer.stop);
    }
}
