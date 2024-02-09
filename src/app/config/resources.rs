use anyhow::{Result, Ok};
use relm4::{
    gtk,
    gtk::glib,
    adw::{gdk, gio, prelude::ApplicationExt},
    main_adw_application,
};

use super::info::{APP_ID, APP_NAME};

pub(crate) fn init() -> Result<()> {
    glib::set_application_name(APP_NAME);
    gio::resources_register_include!("resources.gresource")?;
    // let provider = gtk::CssProvider::new();
    // provider.load_from_resource("/com/github/forensicht/Dompeg/app/style.css");
    // provider.load_from_resource("/com/github/forensicht/Dompeg/app/style-dark.css");

    if let Some(display) = gdk::Display::default() {
        // gtk::style_context_add_provider_for_display(
        //     &display, 
        //     &provider, 
        //     gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        // );

        gtk::IconTheme::for_display(&display)
            .add_resource_path("/com/github/forensicht/Dompeg/icons");
    }
    gtk::Window::set_default_icon_name(APP_ID);

    let app = main_adw_application();
    app.set_resource_base_path(Some("/com/github/forensicht/Dompeg/app"));
    
    Ok(())
}
