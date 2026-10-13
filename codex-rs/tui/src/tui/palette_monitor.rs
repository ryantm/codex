//! Follow the attached terminal's colors, including over SSH and multiplexers.
//!
//! Use crossterm's shared parser, not a second raw stdin reader. Its OSC filters
//! keep user input queued while the event stream is stopped. Rendering remains
//! live during queries, and editor/suspend handoffs serialize through the broker.

use std::io;
use std::io::IsTerminal;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use crossterm::style::Color;
use tokio::sync::broadcast;

use super::event_stream::EventBroker;
use crate::terminal_palette::DefaultColors;

const CHECK_INTERVAL: Duration = Duration::from_secs(2);

// Exit paths and the panic hook can restore modes before Tui is dropped.
static ACTIVE_BROKER: Mutex<Weak<EventBroker>> = Mutex::new(Weak::new());

pub(super) fn pause_before_restore() {
    let broker = ACTIVE_BROKER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .upgrade();
    if let Some(broker) = broker {
        broker.pause_events();
    }
}

pub(super) struct PaletteMonitor {
    stopped: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

impl PaletteMonitor {
    pub(super) fn start(broker: &Arc<EventBroker>, draw_tx: broadcast::Sender<()>) -> Option<Self> {
        // Unsupported terminals must not acquire a recurring query timeout.
        if !io::stdout().is_terminal() {
            return None;
        }
        crate::terminal_palette::default_colors()?;
        *ACTIVE_BROKER
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Arc::downgrade(broker);
        let broker = Arc::downgrade(broker);
        let stopped = Arc::new(AtomicBool::new(false));
        let stop = stopped.clone();
        let worker = thread::Builder::new()
            .name("codex-terminal-colors".into())
            .spawn(move || {
                loop {
                    thread::park_timeout(CHECK_INTERVAL);
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    let Some(broker) = broker.upgrade() else {
                        break;
                    };
                    match broker.refresh_colors(&mut query_colors) {
                        Ok(true) => {
                            let _ = draw_tx.send(());
                        }
                        Ok(false) => {}
                        Err(error) => {
                            // A disconnected or changed terminal must not impose repeated
                            // crossterm timeouts. Retain the last complete sample.
                            tracing::debug!(%error, "Stopping terminal palette monitoring");
                            break;
                        }
                    }
                }
            })
            .inspect_err(|error| tracing::warn!(%error, "Failed to start terminal palette monitor"))
            .ok()?;
        Some(Self {
            stopped,
            worker: Some(worker),
        })
    }
}

fn query_colors() -> io::Result<DefaultColors> {
    fn rgb(color: Option<Color>) -> io::Result<(u8, u8, u8)> {
        match color {
            Some(Color::Rgb { r, g, b }) => Ok((r, g, b)),
            _ => Err(io::Error::other(
                "Terminal did not return an RGB default color",
            )),
        }
    }
    Ok(DefaultColors {
        fg: rgb(crossterm::style::query_foreground_color()?)?,
        bg: rgb(crossterm::style::query_background_color()?)?,
    })
}

impl Drop for PaletteMonitor {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            // Do not leave an in-flight reader behind when an external program takes over.
            let _ = worker.join();
        }
    }
}
