use calloop::EventLoop;
use smithay_ecs::wayland_server::Display;

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

    let state = SmallvilEcs {};
    let mut data = CalloopData { state, display };

    event_loop.run(None, &mut data, |_| ()).unwrap();
}

pub struct SmallvilEcs {}
