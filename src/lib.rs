#[cfg(feature = "for-macro")]
pub mod macros {
    pub use all_syntax_cases::*;
    pub use always_context::*;
    #[cfg(feature = "build")]
    pub use always_context_build;
    pub use anyhow_result::*;
    pub use attributes::*;
}

#[cfg(all(feature = "build", not(feature = "for-macro")))]
pub mod macros {

    #[cfg(feature = "build")]
    pub use always_context_build;
}

#[cfg(feature = "for-macro")]
pub use {anyhow, helpers, proc_macro2, quote, syn};

#[cfg(all(feature = "general", not(feature = "for-macro")))]
pub mod helpers {
    pub use helpers::*;
}

#[cfg(all(feature = "general", not(feature = "for-macro")))]
pub mod macros {
    pub use always_context::*;
}

#[cfg(test)]
mod macro_tests;
