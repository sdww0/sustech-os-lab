mod file;
pub mod file_table;
pub mod pipe;

pub use file::{FileLike, Stderr, Stdin, Stdout};

pub trait FileSystem {
    fn name(&self) -> &str;
}
