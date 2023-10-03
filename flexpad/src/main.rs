use tracing::info;

mod model;
mod ui;
mod version;

fn main() -> Result<(), FlexpadError> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;
    info!("Flexpad started");
    ui::run()?;
    info!("Flexpad finished");
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
