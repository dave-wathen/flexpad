use rust_i18n::i18n;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod model;
mod ui;
mod version;

i18n!("locales", fallback = "en");

fn main() -> Result<(), FlexpadError> {
    tracing_subscriber::registry()
        .with(fmt::layer().with_thread_ids(true).pretty())
        .with(EnvFilter::from_default_env())
        .init();
    info!(target: "flexpad", "Flexpad started");

    ui::run()?;

    info!(target: "flexpad", "Flexpad finished");
    Ok(())
}

#[derive(Debug)]
enum FlexpadError {
    IcedError(iced::Error),
    TracingError(tracing::subscriber::SetGlobalDefaultError),
}

impl From<iced::Error> for FlexpadError {
    fn from(value: iced::Error) -> Self {
        FlexpadError::IcedError(value)
    }
}

impl From<tracing::subscriber::SetGlobalDefaultError> for FlexpadError {
    fn from(value: tracing::subscriber::SetGlobalDefaultError) -> Self {
        FlexpadError::TracingError(value)
    }
}

fn display_iter<T: std::fmt::Display>(
    iter: impl Iterator<Item = T>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    f.write_str("[")?;
    for (idx, id) in iter.enumerate() {
        if idx > 0 {
            f.write_str(", ")?;
        }
        id.fmt(f)?;
    }
    f.write_str("]")
}
