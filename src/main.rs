use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use libshumate::prelude::*;

use adw::{Application, ApplicationWindow};
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

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Gnome Test Rust")
        .default_width(400)
        .default_height(300)
        .content(&content)
        .build();

    window.present();
}

fn show_map_window() {
    let window = adw::Window::builder()
        .title("Map View")
        .default_width(800)
        .default_height(600)
        .build();

    let content_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let header_bar = adw::HeaderBar::new();
    content_box.append(&header_bar);

    let map = libshumate::SimpleMap::new();
    
    // Add a map source (OpenStreetMap)
    let source = libshumate::RasterRenderer::from_url("https://tile.openstreetmap.org/{z}/{x}/{y}.png");
    map.set_map_source(Some(&source));

    let Some(viewport) = map.viewport() else {
        println!("Error: Map viewport could not be initialized.");
        return;
    };
    
    // Try to get current location from IP (fallback to London)
    let lat = 51.5074;
    let lon = -0.1278;

    // Create an HTTP client with a custom User-Agent
    let client = reqwest::Client::builder()
        .user_agent("GnomeTestRust/0.1.0")
        .build()
        .unwrap_or_default();

    // Use GLib main context to spawn the async fetch
    let viewport_clone = viewport.clone();
    glib::MainContext::default().spawn_local(async move {
        println!("Fetching current location (Attempt 1)...");
        let mut success = false;
        
        // Attempt 1: ipapi.co
        if let Ok(response) = client.get("https://ipapi.co/json/").send().await {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                if let (Some(l_lat), Some(l_lon)) = (json["latitude"].as_f64(), json["longitude"].as_f64()) {
                    println!("Found location (ipapi.co): {}, {}", l_lat, l_lon);
                    viewport_clone.set_location(l_lat, l_lon);
                    success = true;
                }
            }
        }

        // Attempt 2: freeipapi.com (Secure HTTPS Fallback)
        if !success {
            println!("Attempt 1 failed. Fetching current location (Attempt 2)...");
            if let Ok(response) = client.get("https://freeipapi.com/api/json").send().await {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    if let (Some(l_lat), Some(l_lon)) = (json["latitude"].as_f64(), json["longitude"].as_f64()) {
                        println!("Found location (freeipapi.com): {}, {}", l_lat, l_lon);
                        viewport_clone.set_location(l_lat, l_lon);
                        success = true;
                    }
                }
            }
        }

        if !success {
            println!("All geolocation attempts failed. Using default location (London).");
        }
    });
    
    viewport.set_location(lat, lon);
    viewport.set_zoom_level(12.0);

    // Create a container for the map and a close button
    let overlay = gtk4::Overlay::builder()
        .vexpand(true)
        .hexpand(true)
        .build();
    
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
        let map_classes: Vec<String> = map_button.css_classes().iter().map(|c| c.to_string()).collect();
        let ok_classes: Vec<String> = ok_button.css_classes().iter().map(|c| c.to_string()).collect();

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
