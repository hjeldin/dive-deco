mod cns_table;
mod deco;
mod deco_model;
mod depth;
mod gas;
mod global_types;
mod ox_tox;
mod record;
mod sim;
mod time;

pub const MAX_GASSES : usize = 16;
pub const MAX_DECO_STAGE: usize = 16;

pub use cns_table::{CNSCoeffRow, CNS_COEFFICIENTS};
pub use deco::{Deco, DecoCalculationError, DecoRuntime, DecoStage, DecoStageType};
pub use deco_model::{ConfigValidationErr, DecoModel, DecoModelConfig, DiveState, ConfigValidationErrorField, ConfigValidationErrorReason};
pub use depth::{Depth, Unit, Units};
pub use time::Time;

pub use gas::{Gas, InertGas, PartialPressures};
pub use global_types::{
    AscentRatePerMinute, CeilingType, Cns, DepthType, GradientFactor, GradientFactors,
    MbarPressure, NDLType, Otu, Pressure,
};
pub use ox_tox::OxTox;
pub use record::RecordData;
pub use sim::Sim;
