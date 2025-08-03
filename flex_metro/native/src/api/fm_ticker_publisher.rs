// Publisher/Subscriber Pattern for FMTickerBase
// Provides timer control and event notifications for musical timing

use std::sync::{Arc, Mutex, mpsc::{channel, Sender, Receiver}};
use std::collections::HashMap;
use std::thread;

use crate::api::fm_ticker_base::{FMSectionTimer, BeatEvent, AsyncTimer};
use crate::api::fm_tempo_interval::FMTempoSequence;
use crate::api::fm_composition::{Composition, BarReference};

/// Timer control commands that subscribers can send
#[derive(Debug, Clone)]
pub enum TimerCommand {
    Start,
    Pause,
    Stop,
    Resume,
}

/// Composition context information for subscribers
/// This provides essential context without duplicating the full Composition structure
#[derive(Debug, Clone)]
pub struct CompositionContext {
    /// Composer name
    pub composer_name: String,
    /// Composition/work name
    pub composition_name: String,
    /// Movement number (1-based)
    pub movement_number: usize,
    /// Movement name
    pub movement_name: String,
    /// Current section name
    pub section_name: String,
    /// Current bar number in the composition
    pub bar_number: usize,
}

/// Events that can be sent to subscribers
#[derive(Debug, Clone)]
pub enum TimerEvent {
    /// Beat/subbeat event with musical timing information (uses existing BeatEvent)
    Beat(BeatEvent),
    /// Composition context change event
    CompositionContext(CompositionContext),
    /// Timer state change events
    Started,
    Paused,
    Stopped,
    Resumed,
}

/// Subscriber trait for receiving timer events
pub trait TimerSubscriber: Send + Sync {
    /// Called when a timer event occurs
    fn on_timer_event(&mut self, event: TimerEvent);
}

/// Publisher for ticker events with subscriber management
/// This integrates with FMTickerBase to provide publisher/subscriber functionality
#[allow(dead_code)] // Fields are used through the command processor thread
pub struct TickerPublisher {
    /// Internal timer
    timer: Arc<Mutex<FMSectionTimer>>,
    /// Command sender for timer control
    command_sender: Sender<TimerCommand>,
    /// Subscribers receiving timer events
    subscribers: Arc<Mutex<HashMap<usize, Box<dyn TimerSubscriber>>>>,
    /// Next subscriber ID
    next_subscriber_id: Arc<Mutex<usize>>,
    /// Composition for context information
    composition: Option<Arc<Composition>>,
    /// Current bar number in the sequence
    current_bar_number: Arc<Mutex<usize>>,
    /// Last sent composition info to avoid duplicates
    last_composition_info: Arc<Mutex<Option<CompositionContext>>>,
}

impl TickerPublisher {
    /// Create a new timer publisher with a tempo sequence
    pub fn new(tempo_sequence: FMTempoSequence) -> Result<Self, String> {
        let timer = Self::create_timer_from_sequence(tempo_sequence)?;
        let timer = Arc::new(Mutex::new(timer));
        let (command_sender, command_receiver) = channel();
        let subscribers = Arc::new(Mutex::new(HashMap::new()));
        let next_subscriber_id = Arc::new(Mutex::new(1));
        let current_bar_number = Arc::new(Mutex::new(1));
        let last_composition_info = Arc::new(Mutex::new(None));

        let mut publisher = Self {
            timer: timer.clone(),
            command_sender,
            subscribers: subscribers.clone(),
            next_subscriber_id,
            composition: None,
            current_bar_number: current_bar_number.clone(),
            last_composition_info: last_composition_info.clone(),
        };

        // Start command processing thread
        publisher.start_command_processor(command_receiver, timer.clone(), subscribers.clone(), 
                                        current_bar_number, last_composition_info);

        Ok(publisher)
    }

    /// Create a new timer publisher with a tempo sequence and composition
    pub fn new_with_composition(
        tempo_sequence: FMTempoSequence, 
        composition: Composition
    ) -> Result<Self, String> {
        let mut publisher = Self::new(tempo_sequence)?;
        
        // Store composition and create bar reference
        let composition = Arc::new(composition);
        
        // Note: We need to handle lifetime issues with BarReference
        // For now, we'll create it when needed in the context lookup
        publisher.composition = Some(composition);
        
        Ok(publisher)
    }

    /// Helper method to create timer from sequence
    fn create_timer_from_sequence(tempo_sequence: FMTempoSequence) -> Result<FMSectionTimer, String> {
        // Convert sequence to the format expected by existing constructor
        if tempo_sequence.intervals.is_empty() {
            return Err("Tempo sequence must contain at least one interval".to_string());
        }
        
        // For now, just use the first interval's bars and tempo
        let first_interval = &tempo_sequence.intervals[0];
        let start_tempo = first_interval.start_tempo_bpm;
        let end_tempo = if tempo_sequence.intervals.len() == 1 {
            first_interval.end_tempo_bpm
        } else {
            tempo_sequence.intervals.last().unwrap().end_tempo_bpm
        };
        
        FMSectionTimer::new_with_section(
            first_interval.bars.clone(),
            start_tempo,
            end_tempo
        )
    }

    /// Subscribe to timer events and get a subscriber ID for unsubscribing
    pub fn subscribe(&mut self, subscriber: Box<dyn TimerSubscriber>) -> usize {
        let mut subscribers = self.subscribers.lock().unwrap();
        let mut next_id = self.next_subscriber_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        subscribers.insert(id, subscriber);
        id
    }

    /// Unsubscribe from timer events
    pub fn unsubscribe(&mut self, subscriber_id: usize) -> bool {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.remove(&subscriber_id).is_some()
    }

    /// Send a command to the timer (start, pause, stop, resume)
    pub fn send_command(&self, command: TimerCommand) -> Result<(), String> {
        self.command_sender.send(command)
            .map_err(|e| format!("Failed to send command: {}", e))
    }

    /// Start the timer
    pub fn start(&self) -> Result<(), String> {
        self.send_command(TimerCommand::Start)
    }

    /// Pause the timer
    pub fn pause(&self) -> Result<(), String> {
        self.send_command(TimerCommand::Pause)
    }

    /// Stop the timer
    pub fn stop(&self) -> Result<(), String> {
        self.send_command(TimerCommand::Stop)
    }

    /// Resume the timer
    pub fn resume(&self) -> Result<(), String> {
        self.send_command(TimerCommand::Resume)
    }

    /// Internal method to start the command processor thread
    fn start_command_processor(
        &mut self,
        command_receiver: Receiver<TimerCommand>,
        timer: Arc<Mutex<FMSectionTimer>>,
        subscribers: Arc<Mutex<HashMap<usize, Box<dyn TimerSubscriber>>>>,
        current_bar_number: Arc<Mutex<usize>>,
        last_composition_info: Arc<Mutex<Option<CompositionContext>>>,
    ) {
        let composition = self.composition.clone();
        
        thread::spawn(move || {
            loop {
                match command_receiver.recv() {
                    Ok(command) => {
                        match Self::process_command(
                            command, 
                            &timer, 
                            &subscribers, 
                            &composition,
                            &current_bar_number,
                            &last_composition_info
                        ) {
                            Ok(_) => {},
                            Err(e) => eprintln!("Error processing command: {}", e),
                        }
                    }
                    Err(_) => {
                        println!("Command receiver channel closed, stopping command processor");
                        break;
                    }
                }
            }
        });
    }

    /// Process a timer command
    fn process_command(
        command: TimerCommand,
        timer: &Arc<Mutex<FMSectionTimer>>,
        subscribers: &Arc<Mutex<HashMap<usize, Box<dyn TimerSubscriber>>>>,
        composition: &Option<Arc<Composition>>,
        current_bar_number: &Arc<Mutex<usize>>,
        last_composition_info: &Arc<Mutex<Option<CompositionContext>>>,
    ) -> Result<(), String> {
        match command {
            TimerCommand::Start => {
                println!("Processing Start command");
                
                // Set up beat callback before starting
                {
                    let mut timer_guard = timer.lock().unwrap();
                    let subscribers_clone = subscribers.clone();
                    let composition_clone = composition.clone();
                    let current_bar_number_clone = current_bar_number.clone();
                    let last_composition_info_clone = last_composition_info.clone();
                    
                    timer_guard.set_tick_callback(move |beat_event| {
                        Self::handle_beat_event(
                            &beat_event, 
                            &subscribers_clone, 
                            &composition_clone,
                            &current_bar_number_clone,
                            &last_composition_info_clone
                        );
                    })?;
                }
                
                // Send composition context when starting
                if let Some(comp) = composition {
                    Self::send_composition_context(
                        1, // Start at bar 1
                        comp, 
                        subscribers, 
                        current_bar_number,
                        last_composition_info
                    );
                }
                
                // Start the timer
                let mut timer_guard = timer.lock().unwrap();
                timer_guard.start()?;
                drop(timer_guard);
                
                // Notify subscribers
                Self::notify_subscribers(subscribers, TimerEvent::Started);
                
                println!("Timer started successfully");
            }
            
            TimerCommand::Pause => {
                println!("Processing Pause command");
                let mut timer_guard = timer.lock().unwrap();
                timer_guard.pause()?;
                drop(timer_guard);
                Self::notify_subscribers(subscribers, TimerEvent::Paused);
                println!("Timer paused");
            }
            
            TimerCommand::Stop => {
                println!("Processing Stop command");
                let mut timer_guard = timer.lock().unwrap();
                timer_guard.stop()?;
                drop(timer_guard);
                
                // Reset bar number
                *current_bar_number.lock().unwrap() = 1;
                *last_composition_info.lock().unwrap() = None;
                
                Self::notify_subscribers(subscribers, TimerEvent::Stopped);
                println!("Timer stopped");
            }
            
            TimerCommand::Resume => {
                println!("Processing Resume command");
                
                // Send composition context when resuming
                if let Some(comp) = composition {
                    let current_bar = *current_bar_number.lock().unwrap();
                    Self::send_composition_context(
                        current_bar,
                        comp, 
                        subscribers,
                        current_bar_number,
                        last_composition_info
                    );
                }
                
                let mut timer_guard = timer.lock().unwrap();
                timer_guard.resume()?;
                drop(timer_guard);
                Self::notify_subscribers(subscribers, TimerEvent::Resumed);
                println!("Timer resumed");
            }
        }
        
        Ok(())
    }

    /// Handle a beat event and notify subscribers
    fn handle_beat_event(
        beat_event: &BeatEvent,
        subscribers: &Arc<Mutex<HashMap<usize, Box<dyn TimerSubscriber>>>>,
        composition: &Option<Arc<Composition>>,
        current_bar_number: &Arc<Mutex<usize>>,
        last_composition_info: &Arc<Mutex<Option<CompositionContext>>>,
    ) {
        // Update current bar number
        *current_bar_number.lock().unwrap() = beat_event.bar_index + 1;
        
        // Send composition context if we have a composition and bar changed
        if let Some(comp) = composition {
            Self::send_composition_context(
                beat_event.bar_index + 1,
                comp,
                subscribers,
                current_bar_number,
                last_composition_info
            );
        }
        
        // Send the beat event directly to subscribers
        Self::notify_subscribers(subscribers, TimerEvent::Beat(beat_event.clone()));
    }

    /// Send composition context to subscribers if it has changed
    fn send_composition_context(
        bar_number: usize,
        composition: &Arc<Composition>,
        subscribers: &Arc<Mutex<HashMap<usize, Box<dyn TimerSubscriber>>>>,
        _current_bar_number: &Arc<Mutex<usize>>,
        last_composition_info: &Arc<Mutex<Option<CompositionContext>>>,
    ) {
        // Create bar reference for lookup
        let bar_ref = BarReference::new(composition);
        
        if let Some(bar_refs) = bar_ref.get_bar_objects(bar_number) {
            let composition_context = CompositionContext {
                composer_name: composition.composer_name.clone(),
                composition_name: composition.work_name.clone(),
                movement_number: bar_refs.movement_index + 1, // Convert to 1-based
                movement_name: bar_refs.movement.name.clone(),
                section_name: bar_refs.section.name.clone(),
                bar_number,
            };
            
            // Check if composition context has changed
            let mut last_info_guard = last_composition_info.lock().unwrap();
            let should_send = match last_info_guard.as_ref() {
                None => true,
                Some(last_info) => {
                    last_info.composer_name != composition_context.composer_name ||
                    last_info.composition_name != composition_context.composition_name ||
                    last_info.movement_number != composition_context.movement_number ||
                    last_info.movement_name != composition_context.movement_name ||
                    last_info.section_name != composition_context.section_name
                }
            };
            
            if should_send {
                *last_info_guard = Some(composition_context.clone());
                drop(last_info_guard);
                Self::notify_subscribers(subscribers, TimerEvent::CompositionContext(composition_context));
            }
        }
    }

    /// Notify all subscribers of an event
    fn notify_subscribers(
        subscribers: &Arc<Mutex<HashMap<usize, Box<dyn TimerSubscriber>>>>,
        event: TimerEvent,
    ) {
        let mut subscribers_guard = subscribers.lock().unwrap();
        for (_id, subscriber) in subscribers_guard.iter_mut() {
            subscriber.on_timer_event(event.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::fm_bar_element::FMBarElement;
    use crate::api::fm_tempo_interval::FMTempoInterval;
    use std::sync::Arc;
    use std::time::Duration;

    // Test subscriber implementation
    struct TestSubscriber {
        events: Arc<Mutex<Vec<TimerEvent>>>,
    }

    impl TestSubscriber {
        fn new() -> (Self, Arc<Mutex<Vec<TimerEvent>>>) {
            let events = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    events: events.clone(),
                },
                events,
            )
        }
    }

    impl TimerSubscriber for TestSubscriber {
        fn on_timer_event(&mut self, event: TimerEvent) {
            let mut events = self.events.lock().unwrap();
            events.push(event);
        }
    }

    #[test]
    fn test_publisher_subscriber_basic() {
        // Create a simple tempo sequence
        let bars = vec![
            FMBarElement::new(4, 4, 0.0, None),
            FMBarElement::new(3, 4, 0.0, None),
        ];
        let interval = FMTempoInterval::new(bars, 120.0, Some(120.0), "Test".to_string());
        let sequence = FMTempoSequence::from_interval(interval);
        
        let mut publisher = TickerPublisher::new(sequence).unwrap();
        
        // Create test subscriber
        let (subscriber, events) = TestSubscriber::new();
        let subscriber_id = publisher.subscribe(Box::new(subscriber));
        
        // Send some commands
        publisher.start().unwrap();
        
        // Wait a bit for events
        std::thread::sleep(Duration::from_millis(100));
        
        publisher.stop().unwrap();
        
        // Wait a bit more to ensure stop event is processed
        std::thread::sleep(Duration::from_millis(50));
        
        // Check that we received some events
        let events_guard = events.lock().unwrap();
        println!("Received {} events:", events_guard.len());
        for (i, event) in events_guard.iter().enumerate() {
            println!("  Event {}: {:?}", i, event);
        }
        
        assert!(!events_guard.is_empty(), "Should have received some events");
        
        // Should have at least start and stop events
        let has_started = events_guard.iter().any(|e| matches!(e, TimerEvent::Started));
        let has_stopped = events_guard.iter().any(|e| matches!(e, TimerEvent::Stopped));
        
        assert!(has_started, "Should have received Started event");
        assert!(has_stopped, "Should have received Stopped event");
        
        // Unsubscribe
        assert!(publisher.unsubscribe(subscriber_id), "Should successfully unsubscribe");
        
        println!("✅ Publisher/Subscriber basic test passed!");
    }
}
