#[cfg(not(feature = "js"))]
pub mod rs;
#[cfg(feature = "js")]
pub mod js;

#[cfg(not(feature = "js"))]
pub use rs::*;
#[cfg(feature = "js")]
pub use js::*;


