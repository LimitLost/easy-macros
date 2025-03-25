#[cfg(feature = "for-macro")]
pub mod macros {
    pub use all_syntax_cases::*;
    pub use always_context::*;
    pub use attributes::*;
    pub use macro_result::*;
}

#[cfg(feature = "for-macro")]
pub use {helpers, proc_macro2, quote, syn};

#[cfg(all(feature = "general", not(feature = "for-macro")))]
pub mod helpers {
    pub use helpers_context::*;
}

#[cfg(all(feature = "general", not(feature = "for-macro")))]
pub mod macros {
    pub use always_context::*;
}

#[cfg(test)]
mod macro_tests;
