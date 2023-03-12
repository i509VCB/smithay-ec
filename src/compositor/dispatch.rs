use wayland_backend::server::{ClientId, ObjectId};
use wayland_server::{
    protocol::wl_surface::{self, WlSurface},
    Client, DataInit, Dispatch, DisplayHandle,
};

use crate::{EcsHandler, EntityData};

use super::Compositor;

/// Internal component for data assoicated with a [`WlSurface`].
///
/// This is not public API. 
struct Internal {
    // TODO: Double buffered pending state
}

impl<State> Dispatch<WlSurface, EntityData, State> for Compositor
where
    State: Dispatch<WlSurface, EntityData> + EcsHandler,
{
    fn request(
        state: &mut State,
        client: &Client,
        resource: &WlSurface,
        request: wl_surface::Request,
        data: &EntityData,
        dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            wl_surface::Request::Destroy => {}

            wl_surface::Request::Attach { buffer, x, y } => todo!(),

            wl_surface::Request::Damage {
                x,
                y,
                width,
                height,
            } => todo!(),

            wl_surface::Request::Frame { callback } => todo!(),

            wl_surface::Request::SetOpaqueRegion { region } => todo!(),

            wl_surface::Request::SetInputRegion { region } => todo!(),

            wl_surface::Request::Commit => todo!(),

            wl_surface::Request::SetBufferTransform { transform } => todo!(),

            wl_surface::Request::SetBufferScale { scale } => todo!(),

            wl_surface::Request::DamageBuffer {
                x,
                y,
                width,
                height,
            } => todo!(),

            wl_surface::Request::Offset { x, y } => todo!(),

            _ => unreachable!(),
        }
    }

    fn destroyed(_state: &mut State, _client: ClientId, _resource: ObjectId, _data: &EntityData) {}
}
