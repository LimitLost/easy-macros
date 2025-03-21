pub use context_internal::context_internal;

#[macro_export]
/// Same syntax as format! macro (from std)
///
/// Makes .with_context() from anyhow more convenient
///
/// Returns a closure
///
/// Add current file and line number to context
macro_rules! context {
    ()=>{
        ||{
            $crate::context_internal!()
        }
    };
    ($($arg:tt)*) => {
        ||{
            //Adds syntax checking from format! macro
            let _= ||{
                let _ = format!($($arg)*);
            };
            $crate::context_internal!($($arg)*)
        }
    };
}
