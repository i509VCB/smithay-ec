use wayland_protocols::xdg::shell::server::{
    xdg_surface::{self, XdgSurface},
    xdg_toplevel::{self, XdgToplevel},
    xdg_wm_base::{self, XdgWmBase},
};
use wayland_server::{Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, Resource};

use crate::{
    compositor::{AlreadyHasRole, Role},
    EntityData,
};

use super::{XdgShell, XdgShellHandler};

impl<State> GlobalDispatch<XdgWmBase, (), State> for XdgShell
where
    State: Dispatch<XdgWmBase, ()> + XdgShellHandler,
{
    fn bind(
        state: &mut State,
        handle: &DisplayHandle,
        client: &Client,
        resource: wayland_server::New<XdgWmBase>,
        global_data: &(),
        data_init: &mut DataInit<'_, State>,
    ) {
        data_init.init(resource, ());
    }
}

impl<State> Dispatch<XdgWmBase, (), State> for XdgShell
where
    State: Dispatch<XdgSurface, EntityData> + XdgShellHandler,
{
    fn request(
        state: &mut State,
        client: &Client,
        resource: &XdgWmBase,
        request: xdg_wm_base::Request,
        data: &(),
        dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            xdg_wm_base::Request::Destroy => todo!(),

            xdg_wm_base::Request::CreatePositioner { id } => todo!(),

            xdg_wm_base::Request::GetXdgSurface { id, surface } => {
                let role = state.ecs().query_one_mut::<&mut Role, _>(&surface).unwrap();
                if let Err(AlreadyHasRole) = role.set_role(XdgShell::SURFACE_ROLE) {
                    resource.post_error(xdg_wm_base::Error::Role, "surface already has a role");
                    return;
                }

                let entity = surface.data::<EntityData>().unwrap().0;
                data_init.init(id, EntityData(entity));

                // TODO: Add xdg_surface data to the entity
            }

            xdg_wm_base::Request::Pong { serial } => todo!(),

            _ => unreachable!(),
        }
    }
}

impl<State> Dispatch<XdgSurface, EntityData, State> for XdgShell
where
    State: Dispatch<XdgToplevel, EntityData> + XdgShellHandler,
{
    fn request(
        state: &mut State,
        client: &Client,
        resource: &XdgSurface,
        request: xdg_surface::Request,
        data: &EntityData,
        dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            xdg_surface::Request::Destroy => todo!(),

            xdg_surface::Request::GetToplevel { id } => {
                let role = state.ecs().query_one_mut::<&mut Role, _>(resource).unwrap();

                // xdg_surface's role is special as creating a toplevel actually replaces the role.
                if role.role() != Some(XdgShell::SURFACE_ROLE) {
                    panic!("TODO: There is supposed to be an error for this?")
                }

                role.replace_role(XdgShell::TOPLEVEL_ROLE);

                let toplevel = data_init.init(id, EntityData(data.0));

                // TODO: Add xdg_surface data to the entity
                state.new_toplevel(toplevel);
            }

            xdg_surface::Request::GetPopup {
                id,
                parent,
                positioner,
            } => todo!(),

            xdg_surface::Request::SetWindowGeometry {
                x,
                y,
                width,
                height,
            } => todo!(),

            xdg_surface::Request::AckConfigure { serial } => todo!(),
            _ => todo!(),
        }
    }
}

impl<State> Dispatch<XdgToplevel, EntityData, State> for XdgShell {
    fn request(
        state: &mut State,
        client: &Client,
        resource: &XdgToplevel,
        request: xdg_toplevel::Request,
        data: &EntityData,
        dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        todo!()
    }
}
