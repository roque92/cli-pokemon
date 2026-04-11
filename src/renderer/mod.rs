pub mod detect;
pub mod iterm2;
pub mod kitty;
pub mod unicode;

pub use detect::detect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererKind {
    Kitty,
    Iterm2,
    Unicode,
}
