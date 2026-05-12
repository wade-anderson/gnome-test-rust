use crate::GeoService;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libshumate::prelude::*;
use std::cell::Cell;

glib::wrapper! {
    pub struct MapView(ObjectSubclass<imp::MapView>)
        @extends adw::Bin, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl MapView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn center_on_current_location(&self, toast_overlay: &adw::ToastOverlay) {
        let imp = self.imp();

        // Update loading state
        self.set_is_loading(true);

        let providers = vec![
            "https://ipapi.co/json/".to_string(),
            "https://freeipapi.com/api/json".to_string(),
        ];
        let geo_service = GeoService::new(providers);

        let viewport = imp.map.viewport().expect("Map viewport not initialized");
        let viewport_clone = viewport.clone();
        let map_clone = imp.map.clone();
        let toast_overlay_clone = toast_overlay.clone();
        let obj_clone = self.clone();

        glib::MainContext::default().spawn_local(async move {
            match geo_service.fetch_location().await {
                Ok(location) => {
                    viewport_clone.set_location(location.latitude, location.longitude);
                    viewport_clone.set_zoom_level(12.0);
                    map_clone.queue_draw();

                    // Emit signal
                    obj_clone.emit_by_name::<()>(
                        "location-changed",
                        &[&location.latitude, &location.longitude],
                    );
                }
                Err(e) => {
                    let error_msg = format!("Geolocation failed: {}", e);
                    let toast = adw::Toast::new(&error_msg);
                    toast_overlay_clone.add_toast(toast);
                }
            }
            // Reset loading state
            obj_clone.set_is_loading(false);
        });
    }
}

impl Default for MapView {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::MapView)]
    pub struct MapView {
        #[property(get, set = Self::set_is_loading, explicit_notify)]
        pub is_loading: Cell<bool>,

        pub map: libshumate::SimpleMap,
        pub spinner: gtk4::Spinner,
    }

    impl MapView {
        fn set_is_loading(&self, value: bool) {
            if self.is_loading.get() != value {
                self.is_loading.set(value);
                if value {
                    self.spinner.start();
                    self.spinner.set_visible(true);
                } else {
                    self.spinner.stop();
                    self.spinner.set_visible(false);
                }
                self.obj().notify_is_loading();
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MapView {
        const NAME: &'static str = "GnomeTestRustMapView";
        type Type = super::MapView;
        type ParentType = adw::Bin;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MapView {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: std::sync::LazyLock<Vec<glib::subclass::Signal>> =
                std::sync::LazyLock::new(|| {
                    vec![
                        glib::subclass::Signal::builder("location-changed")
                            .param_types([f64::static_type(), f64::static_type()])
                            .build(),
                    ]
                });
            SIGNALS.as_ref()
        }

        fn constructed(&self) {
            self.parent_constructed();

            // Set up the map source
            let source = libshumate::RasterRenderer::from_url(
                "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
            );
            self.map.set_map_source(Some(&source));
            self.map.set_vexpand(true);
            self.map.set_hexpand(true);

            // Set up UI with overlay for spinner
            let overlay = gtk4::Overlay::new();
            overlay.set_child(Some(&self.map));

            self.spinner.set_halign(gtk4::Align::Center);
            self.spinner.set_valign(gtk4::Align::Center);
            self.spinner.set_width_request(40);
            self.spinner.set_height_request(40);
            self.spinner.set_visible(false);
            overlay.add_overlay(&self.spinner);

            self.obj().set_child(Some(&overlay));
        }
    }

    impl WidgetImpl for MapView {}
    impl BinImpl for MapView {}
}
