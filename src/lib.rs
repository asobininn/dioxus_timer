use dioxus::prelude::*;
use std::fmt::Display;

#[cfg(target_arch = "wasm32")]
use async_std::task::sleep;
#[cfg(target_arch = "wasm32")]
use instant::{Duration, Instant};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::sleep;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerState {
    Inactive,
    Working,
    Finished,
    Paused,
}

impl Display for TimerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            TimerState::Inactive => "Inactive",
            TimerState::Working => "Working",
            TimerState::Finished => "Finished",
            TimerState::Paused => "Paused",
        };
        write!(f, "{text}")
    }
}

#[derive(Debug, Clone)]
pub struct DioxusTimer {
    preset_duration: Duration,
    target_time: Instant,
    state: TimerState,
    /// Stores Instant::now()
    current_time: Instant,
    /// Stores paused time
    paused_time: Option<Instant>,
}

impl DioxusTimer {
    /// Creates a new `DioxusTimer` instance with default settings.
    pub fn new() -> Self {
        let current_time = Instant::now();
        let target_time = current_time;
        Self {
            preset_duration: Duration::ZERO,
            target_time,
            state: TimerState::Inactive,
            current_time,
            paused_time: None,
        }
    }

    /// Sets the preset duration for the timer.
    pub fn set_preset_time(&mut self, preset_duration: Duration) {
        if self.state == TimerState::Finished {
            return;
        }
        self.preset_duration = preset_duration;
        self.target_time = self
            .current_time
            .checked_add(preset_duration)
            .unwrap_or(self.current_time);
    }

    /// Returns the remaining time on the timer.
    pub fn remaining_time(&self) -> Duration {
        self.target_time
            .checked_duration_since(self.current_time)
            .unwrap_or(Duration::ZERO)
    }

    /// Returns the current state of the timer.
    pub fn state(&self) -> TimerState {
        self.state
    }

    /// Starts the timer if it is in the `Inactive` state.
    ///
    /// If the preset duration is zero, the method does nothing.
    pub fn start(&mut self) {
        match self.state {
            TimerState::Inactive => {
                if self.preset_duration.is_zero() {
                    return;
                }
                self.target_time = self
                    .current_time
                    .checked_add(self.preset_duration)
                    .unwrap_or(self.current_time);
                self.state = TimerState::Working;
            }
            TimerState::Paused => {
                self.state = TimerState::Working;
            }
            _ => {}
        }
    }

    /// Pauses the timer if it is in the `Working` state.
    pub fn pause(&mut self) {
        if let TimerState::Working = self.state {
            self.state = TimerState::Paused;
            self.paused_time = Some(Instant::now());
        }
    }

    /// Resets the timer to its initial state or sets the target time for a new cycle.
    ///
    /// If the timer is in the `Finished` state, it transitions to the `Inactive` state.
    pub fn reset(&mut self) {
        if self.state == TimerState::Finished {
            self.state = TimerState::Inactive;
            return;
        }
        self.target_time = self
            .current_time
            .checked_add(self.preset_duration)
            .unwrap_or(self.current_time);
    }

    /// Updates the timer's current time and checks for state transitions.
    ///
    /// The `Working` state transitions to `Finished` when the target time is reached.
    /// The `Paused` state adjusts the target time based on the time paused.
    /// The `Inactive` state resets the timer.
    pub fn update(&mut self) {
        self.current_time = Instant::now();
        match self.state {
            TimerState::Working => {
                if self
                    .target_time
                    .checked_duration_since(self.current_time)
                    .is_none()
                {
                    self.state = TimerState::Finished;
                }
            }
            TimerState::Paused => {
                self.target_time = self
                    .target_time
                    .checked_add(self.current_time - self.paused_time.unwrap())
                    .unwrap_or(self.current_time);
                self.paused_time = Some(self.current_time);
            }
            TimerState::Inactive => {
                self.reset();
            }
            _ => {}
        }
    }
}

impl Default for DioxusTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for DioxusTimer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rem_time = self.remaining_time().as_secs();
        write!(
            f,
            "{:0>2}:{:0>2}:{:0>2}",
            rem_time / 3600,
            rem_time % 3600 / 60,
            rem_time % 60,
        )
    }
}

/// Manages a DioxusTimer instance within the Dioxus GUI framework.
///
/// # Examples
///
/// ```
///let mut timer = use_timer(Duration::from_millis(16));
///use_effect(move || {
///    spawn(async move {
///        timer.write().set_preset_time(Duration::from_secs(10));
///        timer.write().start();
///    });
///});
///rsx!("{timer}"
/// ```
pub fn use_timer(tick: Duration) -> Signal<DioxusTimer> {
    let mut timer = use_signal(DioxusTimer::new);
    use_future(move || async move {
        loop {
            timer.write().update();
            sleep(tick).await;
        }
    });
    timer
}
