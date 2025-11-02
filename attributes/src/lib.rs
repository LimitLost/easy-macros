/// Internal implementation details. Not intended for direct use.
pub mod internal;

// Re-export for use by proc-macro crate (but hidden in internal module)
#[doc(hidden)]
pub use internal::AttrWithUnknown;

pub use attributes_macros::*;
