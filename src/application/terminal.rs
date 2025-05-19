use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{Clear, ClearType};
use crossterm::execute;

pub trait TerminalControl {
    fn start(&self);
    fn is_running(&self) -> bool;
}

pub struct TerminalController {
    running: Arc<AtomicBool>,
}

impl TerminalController {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl TerminalControl for TerminalController {
    fn start(&self) {
        let running = self.running.clone();
        
        println!("Press 'Q<CR>' to quit, 'Ctrl+L<CR>' to clear screen");
        
        thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                if let Ok(Event::Key(key)) = event::read() {
                    match key.code {
                        KeyCode::Char('q') => {
                            running.store(false, Ordering::Relaxed);
                            break;
                        }
                        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            execute!(std::io::stdout(), Clear(ClearType::All)).unwrap();
                        }
                        _ => {}
                    }
                }
            }
        });
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
} 