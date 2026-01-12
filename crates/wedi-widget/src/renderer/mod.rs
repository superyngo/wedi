#[cfg(feature = "crossterm")]
pub mod crossterm_renderer;

#[cfg(feature = "crossterm")]
pub use crossterm_renderer::CrosstermRenderer;
