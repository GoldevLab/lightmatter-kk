//! Núcleo numérico compartido por el servidor Resuma y el módulo WASM de la PWA.

pub mod chart;
pub mod fit;
pub mod measure;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
mod wasm_api;
