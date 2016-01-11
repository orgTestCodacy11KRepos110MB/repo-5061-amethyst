//! Utilities for game state management.

use super::timing::Duration;

/// Types of state transitions.
pub enum Trans {
    None,
    Pop,
    Push(Box<State>),
    Switch(Box<State>),
    Quit,
}

/// A trait which defines game states that can be used by the state machine.
pub trait State {
    /// Executed when the game state begins.
    fn on_start(&mut self) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self) {}

    /// Executed on every frame before updating, for use in reacting to events.
    // TODO: Replace i32 with an actual Event type of some kind.
    fn handle_events(&mut self, _events: &Vec<i32>) {}

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, _delta: Duration) -> Trans { Trans::None }

    /// Executed on every frame immediately, as fast as the engine will allow.
    fn update(&mut self, _delta: Duration) -> Trans { Trans::Pop }
}

/// A simple stack-based state machine.
pub struct StateMachine {
    running: bool,
    state_stack: Vec<Box<State>>,
}

impl StateMachine {
    pub fn new<T: 'static>(initial_state: T) -> StateMachine
        where T: State
    {
        StateMachine {
            running: false,
            state_stack: vec![Box::new(initial_state)],
        }
    }

    /// Retrieves the currently active state.
    pub fn current(&mut self) -> Option<&mut Box<State>> {
        self.state_stack.last_mut()
    }

    /// Initializes the state machine.
    pub fn start(&mut self) {
        if !self.running {
            self.current().unwrap().on_start();
            self.running = true;
        }
    }

    /// Passes a vector of events to the active state to handle.
    // TODO: Replace i32 with an actual Event type of some kind.
    pub fn handle_events(&mut self, events: &Vec<i32>) {
        if self.running {
            if let Some(state) = self.current() {
                state.handle_events(events);
            }
        }
    }

    /// Updates the currently active state at a steady, fixed interval.
    pub fn fixed_update(&mut self, delta_time: Duration) {
        if self.running {
            let mut trans = Trans::None;
            if let Some(state) = self.state_stack.last_mut() {
                trans = state.fixed_update(delta_time);
            }
            self.transition(trans);
        }
    }

    /// Updates the currently active state immediately.
    pub fn update(&mut self, delta_time: Duration) {
        if self.running {
            let mut trans = Trans::None;
            if let Some(state) = self.state_stack.last_mut() {
                trans = state.update(delta_time);
            }
            self.transition(trans);
        }
    }

    /// Performs a state transition, if requested by either update() or
    /// fixed_update().
    fn transition(&mut self, request: Trans) {
        if self.running {
            match request {
                Trans::None => (),
                Trans::Pop => self.pop(),
                Trans::Push(state) => self.push(state),
                Trans::Switch(state) => self.switch(state),
                Trans::Quit => self.stop(),
            }
        }
    }

    /// Sets the currently active state.
    pub fn switch<T: 'static>(&mut self, state: T)
        where T: State
    {
        if self.running {
            if !self.state_stack.is_empty() {
                self.current().unwrap().on_stop();
                self.state_stack.pop();
            }

            self.state_stack.push(Box::new(state));
            self.current().unwrap().on_start();
        }
    }

    /// Pauses the active state (if any) and pushes a new state onto the state
    /// stack.
    pub fn push<T: 'static>(&mut self, state: T)
        where T: State
    {
        if self.running {
            if let Some(state) = self.current() {
                state.on_pause();
            }

            self.state_stack.push(Box::new(state));
            self.current().unwrap().on_start();
        }
    }

    /// Stops and removes the active state and un-pauses the next state on the
    /// stack (if any).
    pub fn pop(&mut self) {
        if self.running {
            if !self.state_stack.is_empty() {
                self.current().unwrap().on_stop();
                self.state_stack.pop();
            }

            if let Some(state) = self.current() {
                state.on_resume();
            }
        }
    }

    /// Shuts the state machine down.
    pub fn stop(&mut self) {
        if self.running {
            for state in self.state_stack.iter_mut() {
                state.on_stop();
            }

            self.running = false;
        }
    }
}
