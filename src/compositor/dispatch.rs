use std::sync::Mutex;

use smithay::utils::Rectangle;
use wayland_backend::server::{ClientId, ObjectId};
use wayland_server::{
    protocol::{
        wl_callback::WlCallback,
        wl_compositor::{self, WlCompositor},
        wl_region::{self, WlRegion},
        wl_surface::{self, WlSurface},
    },
    Client, DataInit, Dispatch, DisplayHandle, Resource, WEnum,
};

use crate::EntityData;

use super::{
    Buffer, BufferAssignment, Compositor, CompositorHandler, Damage, Internal, RectangleKind,
    RegionAttributes, RegionData, Role,
};

impl<State> Dispatch<WlCompositor, (), State> for Compositor
where
    State: Dispatch<WlCompositor, ()>
        + Dispatch<WlRegion, RegionData>
        + Dispatch<WlSurface, EntityData>
        + CompositorHandler,
{
    fn request(
        state: &mut State,
        _client: &Client,
        _resource: &WlCompositor,
        request: wl_compositor::Request,
        _data: &(),
        _display: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            wl_compositor::Request::CreateSurface { id } => {
                let entity = state.ecs().world().reserve_entity();
                let surface = data_init.init(id, EntityData(entity));

                state
                    .ecs()
                    .world()
                    .insert(
                        entity,
                        (
                            Internal::<State>::default(),
                            Role::default(),
                            Buffer::default(),
                        ),
                    )
                    .expect("Entity was reserved");

                state
                    .compositor()
                    .surfaces
                    .insert(surface.id(), surface.clone());
                state.new_surface(surface);
            }

            wl_compositor::Request::CreateRegion { id } => {
                data_init.init(
                    id,
                    RegionData {
                        inner: Mutex::new(RegionAttributes::default()),
                    },
                );
            }

            _ => unreachable!(),
        }
    }
}

impl<State> Dispatch<WlSurface, EntityData, State> for Compositor
where
    State: Dispatch<WlSurface, EntityData> + Dispatch<WlCallback, ()> + CompositorHandler + 'static,
{
    fn request(
        state: &mut State,
        _client: &Client,
        surface: &WlSurface,
        request: wl_surface::Request,
        data: &EntityData,
        _display: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            wl_surface::Request::Destroy => {
                // this is handled by Dispatch::destroyed
            }

            wl_surface::Request::Attach { buffer, x, y } => {
                let offset = (x, y).into();
                let offset = (x != 0 || y != 0).then_some(offset);

                if offset.is_some() && surface.version() >= 5 {
                    surface.post_error(
                        wl_surface::Error::InvalidOffset,
                        "Passing non-zero x, y offset in attach is protocol violation since version 5",
                    );
                    return;
                }

                let internal = state
                    .ecs()
                    .world()
                    .query_one_mut::<&mut Internal<State>>(data.0)
                    .expect("Surface must be a valid entity if dispatched");

                if offset.is_some() {
                    internal.pending.delta = offset;
                }

                internal.pending.buffer = Some(match buffer {
                    Some(buffer) => BufferAssignment::NewBuffer(buffer),
                    None => BufferAssignment::Removed,
                });
            }

            wl_surface::Request::Damage {
                x,
                y,
                width,
                height,
            } => {
                let internal = state
                    .ecs()
                    .world()
                    .query_one_mut::<&mut Internal<State>>(data.0)
                    .expect("Surface must be a valid entity if dispatched");
                internal
                    .pending
                    .damage
                    .push(Damage::Surface(Rectangle::from_loc_and_size(
                        (x, y),
                        (width, height),
                    )));
            }

            wl_surface::Request::Frame { callback } => {
                let callback = data_init.init(callback, ());
                let internal = state
                    .ecs()
                    .world()
                    .query_one_mut::<&mut Internal<State>>(data.0)
                    .expect("Surface must be a valid entity if dispatched");
                internal.pending.frame_callbacks.push(callback);
            }

            wl_surface::Request::SetOpaqueRegion { region } => {
                let attributes = region.map(|r| {
                    let attributes_mutex = &r.data::<RegionData>().unwrap().inner;
                    attributes_mutex.lock().unwrap().clone()
                });

                let internal = state
                    .ecs()
                    .world()
                    .query_one_mut::<&mut Internal<State>>(data.0)
                    .expect("Surface must be a valid entity if dispatched");
                internal.pending.opaque_region = attributes;
            }

            wl_surface::Request::SetInputRegion { region } => {
                let attributes = region.map(|r| {
                    let attributes_mutex = &r.data::<RegionData>().unwrap().inner;
                    attributes_mutex.lock().unwrap().clone()
                });

                let internal = state
                    .ecs()
                    .world()
                    .query_one_mut::<&mut Internal<State>>(data.0)
                    .expect("Surface must be a valid entity if dispatched");
                internal.pending.input_region = attributes;
            }

            wl_surface::Request::Commit => {
                let internal = state
                    .ecs()
                    .world()
                    .query_one_mut::<&mut Internal<State>>(data.0)
                    .expect("Surface must be a valid entity if dispatched");

                // The list of system functions need to be cloned ahead of time to deal with the systems that
                // could query the world.
                let pre_commit_systems = internal.pre_commit_systems.clone();

                for system in pre_commit_systems {
                    system(state, surface)
                }

                let internal = state
                    .ecs()
                    .world()
                    .query_one_mut::<&mut Internal<State>>(data.0)
                    .expect("Surface must be a valid entity if dispatched");
                let post_commit_systems = internal.post_commit_systems.clone();

                // TODO: Apply current state

                for system in post_commit_systems {
                    system(state, surface)
                }

                state.commit(surface);
            }

            wl_surface::Request::SetBufferTransform { transform } => {
                if let WEnum::Value(transform) = transform {
                    let internal = state
                        .ecs()
                        .world()
                        .query_one_mut::<&mut Internal<State>>(data.0)
                        .expect("Surface must be a valid entity if dispatched");
                    internal.pending.transform = transform;
                }
            }

            wl_surface::Request::SetBufferScale { scale } => {
                let internal = state
                    .ecs()
                    .world()
                    .query_one_mut::<&mut Internal<State>>(data.0)
                    .expect("Surface must be a valid entity if dispatched");
                internal.pending.scale = scale;
            }

            wl_surface::Request::DamageBuffer {
                x,
                y,
                width,
                height,
            } => {
                let internal = state
                    .ecs()
                    .world()
                    .query_one_mut::<&mut Internal<State>>(data.0)
                    .expect("Surface must be a valid entity if dispatched");
                internal
                    .pending
                    .damage
                    .push(Damage::Buffer(Rectangle::from_loc_and_size(
                        (x, y),
                        (width, height),
                    )));
            }

            wl_surface::Request::Offset { x, y } => {
                let internal = state
                    .ecs()
                    .world()
                    .query_one_mut::<&mut Internal<State>>(data.0)
                    .expect("Surface must be a valid entity if dispatched");
                internal.pending.delta = Some((x, y).into());
            }

            _ => unreachable!(),
        }
    }

    fn destroyed(state: &mut State, _client: ClientId, resource: ObjectId, data: &EntityData) {
        if let Some(surface) = state.compositor().surfaces.remove(&resource) {
            state.destroy(&surface);

            let internal = state
                .ecs()
                .world()
                .query_one_mut::<&mut Internal<State>>(data.0)
                .expect("Surface must be a valid entity if dispatched");
            let destroy_systems = internal.destroy_systems.clone();

            for system in destroy_systems {
                system(state, &surface);
            }
        }
    }
}

impl<State> Dispatch<WlRegion, RegionData, State> for Compositor
where
    State: Dispatch<WlRegion, RegionData>,
{
    fn request(
        _state: &mut State,
        _client: &Client,
        _resource: &WlRegion,
        request: wl_region::Request,
        data: &RegionData,
        _display: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
        let mut guard = data.inner.lock().unwrap();
        match request {
            wl_region::Request::Add {
                x,
                y,
                width,
                height,
            } => guard.rects.push((
                RectangleKind::Add,
                Rectangle::from_loc_and_size((x, y), (width, height)),
            )),

            wl_region::Request::Subtract {
                x,
                y,
                width,
                height,
            } => guard.rects.push((
                RectangleKind::Subtract,
                Rectangle::from_loc_and_size((x, y), (width, height)),
            )),

            wl_region::Request::Destroy => {
                // all is handled by our destructor
            }

            _ => unreachable!(),
        }
    }
}
