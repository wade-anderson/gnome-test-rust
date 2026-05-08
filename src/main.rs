use gtk4::prelude::*;
use libadwaita as adw;

use adw::{Application, ApplicationWindow};
use gtk4::{Button, Label, Orientation};

fn main() {
    let app = Application::builder()
        .application_id("org.example.GnomeTestRust")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn create_widgets() -> (gtk4::Image, Label, Button, gtk4::Box) {
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

    let button = Button::builder()
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
    content.append(&button);

    (icon, label, button, content)
}

fn build_ui(app: &Application) {
    let (_, _, button, content) = create_widgets();

    // When the button is clicked, quit the application
    button.connect_clicked(|_| {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_creation() {
        // Initialize GTK for testing
        gtk4::init().expect("Failed to initialize GTK");
        adw::init().expect("Failed to initialize Libadwaita");

        let (icon, label, button, content) = create_widgets();

        assert!(icon.file().is_some());
        assert_eq!(label.label(), "Hello World");
        assert_eq!(button.label().unwrap(), "OK");
        assert_eq!(content.orientation(), Orientation::Vertical);
    }
}
