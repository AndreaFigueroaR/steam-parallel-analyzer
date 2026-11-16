//! Procesador paralelo de reviews de Steam.
//!
//! La biblioteca expone el pipeline completo (lectura por lotes, procesamiento fork-join con
//! Rayon, serialización y gráficos) para que lo consuman los binarios del proyecto
//! (`tp1-AndreaFigueroaR`, `charts` y `scaling`) y los tests de integración.

pub mod charts;
pub mod collectors;
pub mod dtos;
pub mod processing;
pub mod utils;
