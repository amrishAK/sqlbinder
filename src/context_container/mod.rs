/// Error types for context container initialization and access.
pub mod error;
/// Context container data model shared across application layers.
pub mod model;
/// APIs for loading and reading values from the context container.
pub mod context_container_handler;


/// Public error type for context container operations.
pub use error::ContextContainerError;
/// Public context container type.
pub use model::ContextContainer;