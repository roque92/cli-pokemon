pub mod detect;
pub mod kitty;
pub mod unicode;

pub use detect::detect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererKind {
    Kitty,
    Unicode,
}
