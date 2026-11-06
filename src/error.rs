use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Configuration(String),
    Source(String),
    Post(String),
    Output(String),
    Component(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(message)
            | Self::Source(message)
            | Self::Post(message)
            | Self::Output(message)
            | Self::Component(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for AppError {}

pub type Result<Value> = std::result::Result<Value, AppError>;
