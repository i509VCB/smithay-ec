use std::{sync::Arc, time::Duration};

use calloop::EventLoop;
use smithay_ecs::{
    compositor::{Compositor, CompositorHandler, RegionData, Role},
    shm::Shm,
    wayland_protocols::xdg::shell::server::{
        xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel, xdg_wm_base::XdgWmBase,
    },
    wayland_server::{
        delegate_dispatch, delegate_global_dispatch,
        protocol::{
            wl_callback::WlCallback, wl_compositor::WlCompositor, wl_region::WlRegion,
            wl_shm::WlShm, wl_shm_pool::WlShmPool, wl_subcompositor::WlSubcompositor,
            wl_subsurface::WlSubsurface, wl_surface::WlSurface,
        },
        Display, ListeningSocket,
    },
    xdg_shell::{XdgShell, XdgShellHandler},
    Ecs, EcsAccess, EntityData,
};

pub struct CalloopData {
    state: SmallvilEcs,
    display: Display<SmallvilEcs>,
}

fn main() {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    let mut event_loop: EventLoop<CalloopData> = EventLoop::try_new().unwrap();
    let display = Display::new().unwrap();
    let mut display_handle = display.handle();

    let state = SmallvilEcs {
        ecs: Ecs::new(),
        compositor: Compositor::new::<SmallvilEcs>(&mut display_handle),
        shm: Shm::new::<SmallvilEcs>(&mut display_handle),
        xdg_shell: XdgShell::new::<SmallvilEcs>(&mut display_handle),
    };
    let mut data = CalloopData { state, display };

    let socket = ListeningSocket::bind("wayland-3").unwrap();

    event_loop
        .run(Duration::from_millis(16), &mut data, |state| {
            if let Some(socket) = socket.accept().unwrap() {
                state
                    .display
                    .handle()
                    .insert_client(socket, Arc::new(()))
                    .unwrap();
            }

            state.display.dispatch_clients(&mut state.state).unwrap();
            state.display.flush_clients().unwrap();
        })
        .unwrap();
}

pub struct SmallvilEcs {
    ecs: Ecs,
    compositor: Compositor,
    shm: Shm,
    xdg_shell: XdgShell,
}

impl EcsAccess for SmallvilEcs {
    fn ecs(&mut self) -> &mut Ecs {
        &mut self.ecs
    }
}

impl CompositorHandler for SmallvilEcs {
    fn compositor(&mut self) -> &mut Compositor {
        &mut self.compositor
    }

    fn new_surface(&mut self, _surface: WlSurface) {}

    fn commit(&mut self, surface: &WlSurface) {
        let role = self.ecs().query_one_mut::<&Role, _>(surface).unwrap();
        dbg!("Commited with role:", role.role());
    }
}

impl XdgShellHandler for SmallvilEcs {
    fn new_toplevel(&mut self, toplevel: XdgToplevel) {
        let role = self.ecs().query_one_mut::<&Role, _>(&toplevel).unwrap();
        assert_eq!(role.role(), Some(XdgShell::TOPLEVEL_ROLE));
    }
}

delegate_global_dispatch!(SmallvilEcs: [WlCompositor: ()] => Compositor);
delegate_dispatch!(SmallvilEcs: [WlCompositor: ()] => Compositor);
delegate_dispatch!(SmallvilEcs: [WlRegion: RegionData] => Compositor);
delegate_dispatch!(SmallvilEcs: [WlSurface: EntityData] => Compositor);
delegate_dispatch!(SmallvilEcs: [WlCallback: ()] => Compositor);

delegate_global_dispatch!(SmallvilEcs: [WlSubcompositor: ()] => Compositor);
delegate_dispatch!(SmallvilEcs: [WlSubcompositor: ()] => Compositor);
delegate_dispatch!(SmallvilEcs: [WlSubsurface: EntityData] => Compositor);

delegate_global_dispatch!(SmallvilEcs: [WlShm: ()] => Shm);
delegate_dispatch!(SmallvilEcs: [WlShm: ()] => Shm);
delegate_dispatch!(SmallvilEcs: [WlShmPool: ()] => Shm);

delegate_global_dispatch!(SmallvilEcs: [XdgWmBase: ()] => XdgShell);
delegate_dispatch!(SmallvilEcs: [XdgWmBase: ()] => XdgShell);
delegate_dispatch!(SmallvilEcs: [XdgSurface: EntityData] => XdgShell);
delegate_dispatch!(SmallvilEcs: [XdgToplevel: EntityData] => XdgShell);
