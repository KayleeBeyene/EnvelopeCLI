//! Event handling for the TUI
//!
//! This module handles terminal events (key presses, mouse events, resize)
//! using crossterm's event system.

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Terminal events
#[derive(Debug, Clone)]
pub enum Event {
    /// Key press event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Tick event for periodic updates
    Tick,
}

/// Event handler for terminal events
pub struct EventHandler {
    /// Event sender
    #[allow(dead_code)]
    sender: mpsc::Sender<Event>,
    /// Event receiver
    receiver: mpsc::Receiver<Event>,
    /// Event thread handle
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new event handler with the specified tick rate
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    // Calculate timeout for next tick
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(Duration::ZERO);

                    // Poll for events
                    if event::poll(timeout).expect("Failed to poll events") {
                        match event::read().expect("Failed to read event") {
                            CrosstermEvent::Key(key) => {
                                if sender.send(Event::Key(key)).is_err() {
                                    return;
                                }
                            }
                            CrosstermEvent::Mouse(mouse) => {
                                if sender.send(Event::Mouse(mouse)).is_err() {
                                    return;
                                }
                            }
                            CrosstermEvent::Resize(width, height) => {
                                if sender.send(Event::Resize(width, height)).is_err() {
                                    return;
                                }
                            }
                            _ => {}
                        }
                    }

                    // Send tick event if needed
                    if last_tick.elapsed() >= tick_rate {
                        if sender.send(Event::Tick).is_err() {
                            return;
                        }
                        last_tick = Instant::now();
                    }
                }
            })
        };

        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Get the next event (blocking)
    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.receiver.recv()
    }

    /// Try to get the next event (non-blocking)
    #[allow(dead_code)]
    pub fn try_next(&self) -> Result<Event, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(Duration::from_millis(250))
    }
}
