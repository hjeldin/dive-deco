pub type Pressure = f32;
pub type DepthType = f32;
pub type GradientFactor = u8;
pub type GradientFactors = (u8, u8);
pub type MbarPressure = u16;
pub type AscentRatePerMinute = f32;
pub type Cns = f32;
pub type Otu = f32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NDLType {
    Actual,    // take into consideration off-gassing during ascent
    ByCeiling, // treat NDL as a point when ceiling > 0.
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CeilingType {
    Actual,
    Adaptive,
}
