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
pub mod shm;
pub mod xdg_shell;

use std::fmt::Debug;

pub use hecs;
use hecs::{Entity, Query, QueryItem, QueryOneError};
pub use wayland_protocols;
pub use wayland_server;
use wayland_server::Resource;

pub struct Ecs {
    world: hecs::World,
}

impl Ecs {
    pub fn new() -> Self {
        Self {
            world: hecs::World::new(),
        }
    }

    /// Low level access to the world.
    pub fn world(&mut self) -> &mut hecs::World {
        &mut self.world
    }

    pub fn query_one_mut<Q: Query, I: Resource>(
        &mut self,
        resource: &I,
    ) -> Result<QueryItem<'_, Q>, QueryOneError> {
        let entity = resource
            .data::<EntityData>()
            .expect("Cannot query resource that does not use EntityData")
            .0;
        self.world().query_one_mut::<Q>(entity)
    }
}

impl Debug for Ecs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ecs").finish_non_exhaustive()
    }
}

pub trait EcsAccess: 'static {
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
