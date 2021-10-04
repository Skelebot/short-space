use eyre::{eyre::eyre, Result};
use legion::{Resources, World};

// Marker component for entities which get removed when on_stop() of their containing state gets removed
#[derive(Clone, Copy)]
pub struct Scoped {
    pub id: std::any::TypeId,
}

#[derive(Debug)]
pub enum CustomEvent {
    Exit,
}

pub enum Transition {
    None,
    Pop,
    Push(Box<dyn State>),
    Switch(Box<dyn State>),
}
use winit::event::Event;

pub trait State {
    fn on_start(&mut self, _world: &mut World, _resources: &mut Resources) {}
    fn on_stop(&mut self, _world: &mut World, _resources: &mut Resources) {}
    fn on_pause(&mut self, _world: &mut World, _resources: &mut Resources) {}
    fn on_resume(&mut self, _world: &mut World, _resources: &mut Resources) {}
    fn handle_event(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        _event: Event<CustomEvent>,
    ) -> Transition {
        Transition::None
    }
    fn update(&mut self, _world: &mut World, _resources: &mut Resources) -> Transition {
        Transition::None
    }
    fn update_inactive(&mut self, _world: &mut World, _resources: &mut Resources) {}
}

pub struct StateMachine<'s> {
    stack: Vec<Box<dyn State + 's>>,
}

impl<'s> StateMachine<'s> {
    pub fn new<S: State + 's>(init: S) -> Self {
        StateMachine {
            stack: vec![Box::new(init)],
        }
    }

    pub fn start(&mut self, world: &mut World, resources: &mut Resources) -> Result<()> {
        let state = self
            .stack
            .last_mut()
            .ok_or_else(|| eyre!("No states present"))?;
        state.on_start(world, resources);
        Ok(())
    }

    pub fn handle_event(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
        event: Event<CustomEvent>,
    ) {
        let trans = match self.stack.last_mut() {
            Some(state) => state.handle_event(world, resources, event),
            None => Transition::None,
        };

        self.transition(trans, world, resources);
    }

    pub fn update(&mut self, world: &mut World, resources: &mut Resources) {
        let trans = match self.stack.last_mut() {
            Some(state) => state.update(world, resources),
            None => Transition::None,
        };
        for state in self.stack.iter_mut() {
            state.update_inactive(world, resources)
        }
        self.transition(trans, world, resources)
    }

    fn transition(&mut self, trans: Transition, world: &mut World, resources: &mut Resources) {
        match trans {
            Transition::None => (),
            Transition::Pop => self.pop(world, resources),
            Transition::Push(state) => self.push(state, world, resources),
            Transition::Switch(state) => self.switch(state, world, resources),
        }
    }

    fn pop(&mut self, world: &mut World, resources: &mut Resources) {
        if let Some(mut state) = self.stack.pop() {
            state.on_stop(world, resources)
        }
        if let Some(state) = self.stack.last_mut() {
            state.on_resume(world, resources);
        }
    }

    fn push(&mut self, state: Box<dyn State>, world: &mut World, resources: &mut Resources) {
        if let Some(state) = self.stack.last_mut() {
            state.on_pause(world, resources)
        }
        self.stack.push(state);
        let new_state = self.stack.last_mut().unwrap();
        new_state.on_start(world, resources)
    }

    fn switch(&mut self, state: Box<dyn State>, world: &mut World, resources: &mut Resources) {
        if let Some(mut state) = self.stack.pop() {
            state.on_stop(world, resources)
        }
        self.stack.push(state);
        let new_state = self.stack.last_mut().unwrap();
        new_state.on_start(world, resources)
    }

    pub fn stop(&mut self, world: &mut World, resources: &mut Resources) {
        while let Some(mut state) = self.stack.pop() {
            state.on_stop(world, resources)
        }
    }
}
