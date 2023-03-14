//! Experiments to test EC (Entity Component) in Smithay.
//!
//! # EC
//!
//! > Wait surely that is a typo, shouldn't that be ECS (Entity component system) instead?
//!
//! TODO: Describe the specifics of the compromise between EC and ECS that was chosen here.
//!
//!

pub mod compositor;

use std::fmt::Debug;

pub use hecs;
use hecs::{Entity, Query, QueryItem, QueryOneError};
pub use wayland_server;

pub struct Ecs {
    world: hecs::World,
}

impl Ecs {
    pub fn world(&mut self) -> &mut hecs::World {
        &mut self.world
    }
}

impl Debug for Ecs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ecs").finish_non_exhaustive()
    }
}

pub trait EcsAccess {
    fn ecs(&mut self) -> &mut Ecs;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityData(Entity);

impl From<EntityData> for Entity {
    fn from(value: EntityData) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypedEntity<T>(Entity, T);

impl<T> TypedEntity<T> {
    pub const fn entity(&self) -> Entity {
        self.0
    }

    pub const fn value(&self) -> &T {
        &self.1
    }
}
