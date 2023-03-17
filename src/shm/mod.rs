use wayland_server::{
    protocol::{
        wl_shm::{self, WlShm},
        wl_shm_pool::{self, WlShmPool},
    },
    Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New,
};

pub struct Shm {}

impl Shm {
    pub fn new<State>(display: &mut DisplayHandle) -> Self
    where
        State: GlobalDispatch<WlShm, ()> + Dispatch<WlShm, ()> + 'static,
    {
        let _global = display.create_global::<State, WlShm, ()>(1, ());

        Self {}
    }
}

impl<State> GlobalDispatch<WlShm, (), State> for Shm
where
    State: GlobalDispatch<WlShm, ()> + Dispatch<WlShm, ()>,
{
    fn bind(
        _state: &mut State,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<WlShm>,
        _global_data: &(),
        data_init: &mut DataInit<'_, State>,
    ) {
        let shm = data_init.init(resource, ());
        shm.format(wl_shm::Format::Argb8888);
        shm.format(wl_shm::Format::Xrgb8888);
    }
}

impl<State> Dispatch<WlShm, (), State> for Shm
where
    State: Dispatch<WlShm, ()> + Dispatch<WlShmPool, ()>,
{
    fn request(
        state: &mut State,
        client: &Client,
        resource: &WlShm,
        request: wl_shm::Request,
        data: &(),
        dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            wl_shm::Request::CreatePool { id, fd, size } => {
                // TODO: This could be an entity actually.
                let _pool = data_init.init(id, ());
            }

            _ => unreachable!(),
        }
    }
}

impl<State> Dispatch<WlShmPool, (), State> for Shm {
    fn request(
        state: &mut State,
        client: &Client,
        resource: &WlShmPool,
        request: wl_shm_pool::Request,
        data: &(),
        dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        todo!()
    }
}
