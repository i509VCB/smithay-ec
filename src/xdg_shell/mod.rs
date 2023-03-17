use wayland_protocols::xdg::shell::server::{xdg_toplevel::XdgToplevel, xdg_wm_base::XdgWmBase};
use wayland_server::{DisplayHandle, GlobalDispatch};

use crate::EcsAccess;

mod dispatch;

pub trait XdgShellHandler: EcsAccess {
    fn new_toplevel(&mut self, toplevel: XdgToplevel);
}

pub struct XdgShell {}

impl XdgShell {
    pub fn new<State>(display: &mut DisplayHandle) -> Self
    where
        State: GlobalDispatch<XdgWmBase, ()> + XdgShellHandler,
    {
        display.create_global::<State, XdgWmBase, ()>(4, ());
        Self {}
    }

    pub const SURFACE_ROLE: &str = "xdg_surface";
    pub const TOPLEVEL_ROLE: &str = "xdg_toplevel";
}
