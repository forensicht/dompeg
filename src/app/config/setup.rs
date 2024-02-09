use super::{resources, actions, settings};
use anyhow::Result;
use relm4::gtk;

pub fn init() -> Result<()> {
    gtk::init()?;

    // Enable logging
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_max_level(tracing::Level::ERROR)
        .init();
    resources::init()?;
    relm4_icons::initialize_icons();
    actions::init();
    settings::init()?;

    Ok(())
}
