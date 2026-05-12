use crate::GeoService;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libshumate::prelude::*;

glib::wrapper! {
    pub struct MapView(ObjectSubclass<imp::MapView>)
        @extends adw::Bin, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl MapView {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for MapView {
    fn default() -> Self {
        Self::new()
    }
}

impl MapView {
    pub fn center_on_current_location(&self, toast_overlay: &adw::ToastOverlay) {
        let imp = self.imp();
        let viewport = imp.map.viewport().expect("Map viewport not initialized");

        let providers = vec![
            "https://ipapi.co/json/".to_string(),
            "https://freeipapi.com/api/json".to_string(),
        ];
        let geo_service = GeoService::new(providers);

        let viewport_clone = viewport.clone();
        let map_clone = imp.map.clone();
        let toast_overlay_clone = toast_overlay.clone();

        glib::MainContext::default().spawn_local(async move {
            match geo_service.fetch_location().await {
                Ok(location) => {
                    viewport_clone.set_location(location.latitude, location.longitude);
                    viewport_clone.set_zoom_level(12.0);
                    map_clone.queue_draw();
                }
                Err(e) => {
                    let error_msg = format!("Geolocation failed: {}", e);
                    let toast = adw::Toast::new(&error_msg);
                    toast_overlay_clone.add_toast(toast);
                }
            }
        });
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MapView {
        pub map: libshumate::SimpleMap,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MapView {
        const NAME: &'static str = "GnomeTestRustMapView";
        type Type = super::MapView;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for MapView {
        fn constructed(&self) {
            self.parent_constructed();

            // Set up the map source
            let source = libshumate::RasterRenderer::from_url(
                "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
            );
            self.map.set_map_source(Some(&source));
            self.map.set_vexpand(true);
            self.map.set_hexpand(true);

            // Default location (London)
            if let Some(viewport) = self.map.viewport() {
                viewport.set_location(51.5074, -0.1278);
                viewport.set_zoom_level(12.0);
            }

            self.obj().set_child(Some(&self.map));
        }
    }

    impl WidgetImpl for MapView {}
    impl BinImpl for MapView {}
}
