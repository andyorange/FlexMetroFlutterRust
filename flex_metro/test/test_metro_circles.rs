// test_metro.rs

//mod rust::api::FMCircleTicker;
// mod flex_timer;
// mod fm_base;

use std::thread::Timer;
use rust::api::FMBase::{FMTicks, FMBarElement};
use rust::api::{FMTickerBase, FMCircleTicker};
use flex_timer::{FMMeasureTimer, ctest};
use fm_base::FMBarElement;
use std::os;


fn main() {
    let cfg_dir = os.path.dirname(os.path.realpath(__file__))?;
    cfg_dir = os.path.join(cfg_dir, "cfg");
    let ticks = FMTicks::new(cfg_dir);
    //let bar01 = FMBarElement::new(nom=7, denom=8, nom_secs=-1, tempo_bar_start=52, tempo_bar_end=96, base_beat=8, beats=[2, 3, 2], time_beats=None, ref &ticks);

    let mut si = FMBarElement {
        nom: 5,
        denom: 8,
        nom_secs: 0,
        tempo_bar_start: 52,
        tempo_bar_end: 96,
        base_beat: 8,
        beats: vec![2, 3],
    };

    si = FMBarElement {
        nom: 2,
        denom: 4,
        nom_secs: 0,
        tempo_bar_start: 96,
        tempo_bar_end: 96,
        base_beat: 4,
        beats: vec![],
    };

    let mut page = flet::Page::new();
    page.title = "FlexMetro".to_string();
    page.update();

    let ticker = FMCircleTicker::new(page);
    let tmr = FMMeasureTimer::new(si, 2);
    ticker.set_tick_duration(tmr.timers.iter().min().unwrap());
    ticker.connect_timer(tmr);
    page.add(ticker);
    // ticker.update();
    page.update();
    ticker.start_timer();
    // ticker.stop_timer();
}

//flet::app(target=main);
// si = FMBarElement(nom=7, denom=8, tempo_bar_start=52, tempo_bar_end=96, base_beat=8, beats=[2, 3, 2]);
// let tmr = FMMeasureTimer::new(si, 2, ctest);
// tmr.start();