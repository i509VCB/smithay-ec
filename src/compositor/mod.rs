//! Protocol implementations for surfaces, subsurfaces and regions.
//!
//! # Surface roles
//!
//! The wayland protocol specifies that a surface needs to be assigned a role before displaying the surface.
//! Furthermore, a surface can only have a single role during its whole lifetime[^1]. Smithay represents this
//! role as a `&'static str` identifier, that can only be set once on a surface.
//!
//! Role data is stored in a [`Role`] which can be queried from a [`WlSurface`].
//!
//! # Accessing the buffers
//!
//! When a buffer is commited the surface may become visible. The attached buffer and related data is stored in
//! a [`Buffer`] which can be queried from a [`WlSurface`].
//!
//! # Subsurfaces
//!
//! This module provides an implementation for [`WlSubsurface`]. A subsurface has a [`role`](Subsurface::ROLE)
//! and a [`Subsurface`] can be queried from a [`WlSurface`] if the surface is a subsurface.
//!
//! TODO: Query to get the next subsurface for a surface with some child subsurfaces.
//!
//! [^1]: Some surface roles, such as xdg-surface can be turned into an xdg-popup or xdg-toplevel.
//! [`Role::replace_role`] is available for those types of surface roles.

mod dispatch;

use std::{collections::HashMap, sync::Mutex};

use hecs::Entity;
use smithay::utils::{Logical, Point, Rectangle};
use wayland_backend::server::ObjectId;
use wayland_server::protocol::{
    wl_buffer, wl_output, wl_subsurface::WlSubsurface, wl_surface::WlSurface,
};

use crate::EcsAccess;

// TODO: Way to allow components to be notified that a surface was pre and post committed. This kind of acts
// like a system. But it's per object type.

pub struct Compositor {
    surfaces: HashMap<ObjectId, WlSurface>,
}

pub trait CompositorHandler: EcsAccess {
    // TODO: Remove this Compositor type
    fn compositor(&mut self) -> &mut Compositor;

    fn new_surface(&mut self, surface: WlSurface);

    fn commit(&mut self, surface: &WlSurface);

    fn destroy(&mut self, surface: &WlSurface) {
        let _ = surface;
    }
}

/// A system that is run before applying a pending surface state.
///
/// This is typically used by protocol extensions that add state to a surface and need to check on commit that
/// the client did not request an illegal state before it is applied on commit.
pub type SurfacePreCommit<State> = fn(state: &mut State, surface: &WlSurface);

/// A system that is run after commiting the current surface state.
///
/// This is typically used by abstractions that further process the state.
pub type SurfacePostCommit<State> = fn(state: &mut State, surface: &WlSurface);

/// A system that is run when a surface is destroyed.
///
/// This may be useful for cleaning up state introduced by extension protocol objects.
pub type SurfaceDestroy<State> = fn(state: &mut State, surface: &WlSurface);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Damage {
    Surface(Rectangle<i32, Logical>),
    Buffer(Rectangle<i32, smithay::utils::Buffer>),
}

/// The role of a [`WlSurface`].
///
/// This can always be queried if the surface is alive.
#[derive(Debug, Default)]
pub struct Role(Option<&'static str>);

impl Role {
    /// Get the current role.
    ///
    /// Returns [`None`] if there is no currently set role.
    pub fn role(&self) -> Option<&'static str> {
        self.0
    }

    /// Sets the role.
    ///
    /// This will return [`Err`] if the data already has a role. Most protocols will send a protocol error if
    /// that is encountered.
    pub fn set_role(&mut self, role: &'static str) -> Result<(), AlreadyHasRole> {
        if self.0.is_some() {
            return Err(AlreadyHasRole);
        }

        self.0 = Some(role);
        Ok(())
    }

    /// Replaces the current role.
    ///
    /// For protocol implementations, [`Role::set_role`] should be preferred.
    ///
    /// This is intended for roles such as xdg-surface where an xdg_surface may be further extended to an
    /// xdg_popup or xdg_toplevel.
    pub fn replace_role(&mut self, role: &'static str) {
        assert!(
            self.0.is_some(),
            "Called Role::replace_role with no currently set role"
        );
        self.0 = Some(role);
    }
}

/// The buffer attached to a [`WlSurface`].
///
/// This can always be queried if the surface is alive.
#[derive(Debug)]
pub struct Buffer {
    assignment: Option<BufferAssignment>,
    delta: Option<Point<i32, Logical>>,
    scale: i32,
    transform: wl_output::Transform,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            assignment: None,
            delta: None,
            scale: 1,
            transform: wl_output::Transform::Normal,
        }
    }
}

impl Buffer {
    // TODO: Buffer
    pub fn buffer(&self) -> Option<BufferAssignment> {
        self.assignment.clone()
    }

    pub fn delta(&self) -> Option<Point<i32, Logical>> {
        self.delta
    }

    pub fn scale(&self) -> i32 {
        self.scale
    }

    pub fn transform(&self) -> wl_output::Transform {
        self.transform
    }
}

#[derive(Debug, Clone)]
pub enum BufferAssignment {
    NewBuffer(wl_buffer::WlBuffer),
    Removed,
}

#[derive(Debug)]
pub struct AlreadyHasRole;

/// The role object assoicated with a subsurface.
pub struct Subsurface {
    /// The entity of the surface which is the root of the subsurface.
    ///
    /// The root is the surface which is the ultimate parent of the sub surface.
    root: Entity,

    /// Whether this subsurface is marked as a sync subsurface.
    ///
    /// Note that a subsurface is sync if it's parent subsurface is sync, regardless of whether this subsurface
    /// is sync or not.
    sync: bool,
}

impl Subsurface {
    pub const ROLE: &str = "subsurface";
}

pub struct RegionData {
    inner: Mutex<RegionAttributes>,
}

/// Kind of a rectangle part of a region
#[derive(Copy, Clone, Debug)]
pub enum RectangleKind {
    /// This rectangle should be added to the region
    Add,
    /// The intersection of this rectangle with the region should
    /// be removed from the region
    Subtract,
}

/// Description of the contents of a region
///
/// A region is defined as an union and difference of rectangle.
///
/// This struct contains an ordered `Vec` containing the rectangles defining
/// a region. They should be added or subtracted in this order to compute the
/// actual contents of the region.
#[derive(Clone, Debug, Default)]
pub struct RegionAttributes {
    /// List of rectangle part of this region
    pub rects: Vec<(RectangleKind, Rectangle<i32, Logical>)>,
}

impl RegionAttributes {
    /// Checks whether given point is inside the region.
    pub fn contains<P: Into<Point<i32, Logical>>>(&self, point: P) -> bool {
        let point: Point<i32, Logical> = point.into();
        let mut contains = false;
        for (kind, rect) in &self.rects {
            if rect.contains(point) {
                match kind {
                    RectangleKind::Add => contains = true,
                    RectangleKind::Subtract => contains = false,
                }
            }
        }
        contains
    }
}
