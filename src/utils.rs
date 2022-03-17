pub type AppError = Box<dyn std::error::Error>;
pub type AppResult<T> = Result<T, AppError>;
