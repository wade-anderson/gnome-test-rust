use adw::prelude::*;
use gtk4::prelude::*;
use libadwaita as adw;
use libshumate::prelude::*;

use adw::{Application, ApplicationWindow};
use gnome_test_rust::GeoService;
use gtk4::{Button, Label, Orientation};

#[tokio::main]
async fn main() {
    let app = Application::builder()
        .application_id("org.example.GnomeTestRust")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn create_widgets() -> (gtk4::Image, Label, Button, Button, gtk4::Box) {
    let icon = gtk4::Image::builder()
        .file("icon.png")
        .pixel_size(128)
        .halign(gtk4::Align::Center)
        .margin_bottom(12)
        .build();

    let label = Label::builder()
        .label("Hello World")
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .vexpand(true)
        .hexpand(true)
        .build();

    let map_button = Button::builder()
        .label("Map")
        .halign(gtk4::Align::Center)
        .css_classes(["pill"])
        .build();

    let ok_button = Button::builder()
        .label("OK")
        .halign(gtk4::Align::Center)
        .css_classes(["pill", "suggested-action"])
        .build();

    let content = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    content.append(&icon);
    content.append(&label);
    content.append(&map_button);
    content.append(&ok_button);

    (icon, label, map_button, ok_button, content)
}

fn build_ui(app: &Application) {
    let (_, _, map_button, ok_button, content) = create_widgets();

    // When the Map button is clicked, show the map window
    map_button.connect_clicked(|_| {
        show_map_window();
    });

    // When the OK button is clicked, quit the application
    ok_button.connect_clicked(|_| {
        std::process::exit(0);
    });

    // Set the application icon
    let display = gtk4::gdk::Display::default().expect("Could not get default display");
    let icon_theme = gtk4::IconTheme::for_display(&display);
    if let Ok(current_dir) = std::env::current_dir() {
        icon_theme.add_search_path(current_dir);
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Gnome Test Rust")
        .default_width(400)
        .default_height(300)
        .content(&content)
        .icon_name("icon") // Refers to icon.png in the search path
        .build();

    window.present();
}

fn show_map_window() {
    let window = adw::Window::builder()
        .title("Map View")
        .default_width(800)
        .default_height(600)
        .icon_name("icon")
        .build();

    let content_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let header_bar = adw::HeaderBar::new();
    content_box.append(&header_bar);

    let map = libshumate::SimpleMap::new();

    // Add a map source (OpenStreetMap)
    let source =
        libshumate::RasterRenderer::from_url("https://tile.openstreetmap.org/{z}/{x}/{y}.png");
    map.set_map_source(Some(&source));

    let Some(viewport) = map.viewport() else {
        println!("Error: Map viewport could not be initialized.");
        return;
    };

    // Try to get current location from IP (fallback to London)
    let lat = 51.5074;
    let lon = -0.1278;

    let providers = vec![
        "https://ipapi.co/json/".to_string(),
        "https://freeipapi.com/api/json".to_string(),
    ];
    let geo_service = GeoService::new(providers);

    // Use GLib main context to spawn the async fetch
    let viewport_clone = viewport.clone();
    let map_clone = map.clone();
    glib::MainContext::default().spawn_local(async move {
        if let Some(location) = geo_service.fetch_location().await {
            println!(
                "Found location: {}, {}",
                location.latitude, location.longitude
            );
            viewport_clone.set_location(location.latitude, location.longitude);
            map_clone.queue_draw();
        } else {
            println!("All geolocation attempts failed. Using default location (London).");
        }
    });

    viewport.set_location(lat, lon);
    viewport.set_zoom_level(12.0);
    map.queue_draw();

    // Create a container for the map and a close button
    let overlay = gtk4::Overlay::builder().vexpand(true).hexpand(true).build();

    map.set_vexpand(true);
    map.set_hexpand(true);
    overlay.set_child(Some(&map));

    let close_button = Button::builder()
        .label("Close Map")
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::End)
        .margin_bottom(24)
        .margin_end(24)
        .css_classes(["pill", "suggested-action"])
        .build();

    let window_clone = window.clone();
    close_button.connect_clicked(move |_| {
        window_clone.close();
    });

    overlay.add_overlay(&close_button);

    content_box.append(&overlay);
    window.set_content(Some(&content_box));
    window.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_creation() {
        // Initialize GTK for testing
        gtk4::init().expect("Failed to initialize GTK");
        adw::init().expect("Failed to initialize Libadwaita");

        let (icon, label, map_button, ok_button, content) = create_widgets();

        // Basic property checks
        assert!(icon.file().is_some());
        assert_eq!(label.label(), "Hello World");
        assert_eq!(map_button.label().unwrap(), "Map");
        assert_eq!(ok_button.label().unwrap(), "OK");
        assert_eq!(content.orientation(), Orientation::Vertical);

        // CSS Class checks
        let map_classes: Vec<String> = map_button
            .css_classes()
            .iter()
            .map(|c| c.to_string())
            .collect();
        let ok_classes: Vec<String> = ok_button
            .css_classes()
            .iter()
            .map(|c| c.to_string())
            .collect();

        assert!(map_classes.contains(&"pill".to_string()));
        assert!(ok_classes.contains(&"pill".to_string()));
        assert!(ok_classes.contains(&"suggested-action".to_string()));

        // Children check
        // Count children in the box: icon, label, map_button, ok_button
        let mut child_count = 0;
        let mut next_child = content.first_child();
        while let Some(child) = next_child {
            child_count += 1;
            next_child = child.next_sibling();
        }
        assert_eq!(child_count, 4);
    }
}
