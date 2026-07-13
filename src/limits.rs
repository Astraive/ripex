/// Maximum input size in bytes (1 MB).
pub const MAX_INPUT_SIZE: usize = 1_048_576;

/// Maximum number of tokens a lexer may produce before being rejected.
pub const MAX_TOKENS: usize = 200_000;

/// Maximum parser recursion depth before we abort to avoid stack overflow.
pub const MAX_RECURSION: u32 = 512;
