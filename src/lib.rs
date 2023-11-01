#![feature(exclusive_range_pattern)]
#![feature(slice_pattern)]
#![feature(slice_as_chunks)]

pub mod cpu;
pub mod instruction;
pub mod memory;
pub mod quirks;

pub mod chip8;

pub use chip8::Chip8;
