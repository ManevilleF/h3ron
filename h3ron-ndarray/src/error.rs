use thiserror::Error as DeriveError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Transform is not invertible")]
    TransformNotInvertible,
    #[error("Empty array")]
    EmptyArray,
    #[error("Unsupported array shape")]
    UnsupportedArrayShape,
    #[error("h3ron error: {0}")]
    H3ron(#[from] h3ron::Error),
}
