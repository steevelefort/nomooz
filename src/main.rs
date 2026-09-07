mod behavior;
mod bindings;
mod cache;
mod click;
mod color_rgba;
mod configuration;
mod dependencies;
mod direction;
mod draw_geometry;
mod draw_grid;
mod geometry;
mod labels;
mod layer_draw;
mod layer_keyboard_handler;
mod main_layer;
mod selection;
mod text_renderer;
mod theme;
mod virtual_pointer;
mod wayland_resources;
mod xkb_parser;
mod zone;

use smithay_client_toolkit::{
    compositor::{CompositorState, Region},
    reexports::{calloop::EventLoop, calloop_wayland_source::WaylandSource},
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell},
    },
    shm::{Shm, slot::SlotPool},
};
use wayland_client::{Connection, globals::registry_queue_init};

use crate::{
    behavior::Behavior, bindings::Bindings, cache::load_cache, configuration::Configuration,
    dependencies::Dependencies, main_layer::*, text_renderer::TextRenderer, theme::Theme,
    virtual_pointer::VirtualPointerManager, wayland_resources::WaylandResources,
    xkb_parser::XkbParser,
};

fn main() {
    env_logger::init();

    let config = Configuration::load_config().expect("Can’t load configuration");
    let theme = Theme::from_config(&config).expect("Can’t init theme");
    let bindings = Bindings::from_config(&config).expect("Can’t init bindings");
    let behavior = Behavior::from_config(&config).expect("Can’t init behavior");

    let text_renderer = TextRenderer::new(&theme.font);

    // Try to load the last keymap.
    let mut xkb_parser = XkbParser::new();
    if let Ok(keymap) = load_cache("keymap.xkb") {
        log::debug!("Cache found for keymap");
        xkb_parser
            .parse_from_string(keymap)
            .expect("Failed to compile keymap. Maybe you should delete the cache directory.");
    }
    if let Ok(layout) = load_cache("layout.dat") {
        log::debug!("Cache found for layout");
        let layout = layout.parse::<usize>().expect(
            "Failed to parse layout from cache. Maybe you should delete the cache directory.",
        );
        xkb_parser.set_layout(layout);
    }

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let mut event_loop: EventLoop<MainLayer> =
        EventLoop::try_new().expect("Failed to create event loop");

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");
    let virtual_pointer_manager = VirtualPointerManager::bind(&globals, &qh);

    let surface = compositor.create_surface(&qh);
    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("main_layer"), None);
    layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
    layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
    // Test: Try to let events pass through the overlay
    let input_region = Region::new(&compositor).expect("Failed to create region");
    layer.set_input_region(Some(input_region.wl_region()));
    layer.set_size(0, 0); // Must be set to 0 to let the compositor decide
    layer.set_exclusive_zone(-1);
    layer.commit();

    let pool = SlotPool::new(256 * 256 * 4, &shm).expect("Failed to create pool");

    let mut main_layer = MainLayer::new(
        &globals,
        &qh,
        WaylandResources {
            compositor,
            layer_shell,
            shm,
            virtual_pointer_manager,
            pool,
            layer,
        },
        Dependencies {
            text_renderer,
            xkb_parser,
            theme,
            bindings,
            behavior,
        },
        event_loop.handle(),
    );

    event_queue.roundtrip(&mut main_layer).unwrap();

    let outputs = main_layer.outputs();
    for output in outputs {
        log::debug!("{:?}", output);
    }

    WaylandSource::new(conn.clone(), event_queue)
        .insert(event_loop.handle())
        .expect("Failed to insert Wayland source into event loop");

    // We wait for the first configure before we draw.
    loop {
        event_loop.dispatch(None, &mut main_layer).unwrap();

        if main_layer.exit {
            break;
        }
    }

    main_layer.hide();
    conn.roundtrip().unwrap();

    click::execute_click(&main_layer, &qh, &conn);
}
