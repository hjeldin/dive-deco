use core::fmt;
use core::ops::{Add, AddAssign, Div, Mul, Sub};

use super::DepthType;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Units {
    Metric,
    Imperial,
}

pub trait Unit<T = f32>: Sized {
    fn from_units(val: T, units: Units) -> Self;
    fn to_units(&self, units: Units) -> T;
    fn base_unit(&self) -> T;
}

#[derive(Clone, Copy, Debug)]
pub struct Depth {
    m: DepthType,
}

impl Default for Depth {
    fn default() -> Self {
        Self { m: 0. }
    }
}

impl fmt::Display for Depth {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, r"{}m \ {}ft", self.as_meters(), self.as_feet())
    }
}

impl PartialEq<Self> for Depth {
    fn eq(&self, other: &Self) -> bool {
        self.m == other.m
    }
}

impl PartialOrd<Self> for Depth {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.m.partial_cmp(&other.m)
    }
}

impl Add<Self> for Depth {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self { m: self.m + rhs.m }
    }
}

impl Sub<Self> for Depth {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self { m: self.m - rhs.m }
    }
}

impl Mul<Self> for Depth {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self { m: self.m * rhs.m }
    }
}

impl Mul<f32> for Depth {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self { m: self.m * rhs }
    }
}

impl Div<Self> for Depth {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self { m: self.m / rhs.m }
    }
}

impl Div<f32> for Depth {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self { m: self.m / rhs }
    }
}

impl AddAssign for Depth {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self { m: self.m + rhs.m }
    }
}

impl Unit for Depth {
    fn from_units(val: DepthType, units: Units) -> Self {
        match units {
            Units::Metric => Self::from_meters(val),
            Units::Imperial => Self::from_feet(val),
        }
    }
    fn to_units(&self, units: Units) -> DepthType {
        match units {
            Units::Metric => self.as_meters(),
            Units::Imperial => self.as_feet(),
        }
    }
    fn base_unit(&self) -> f32 {
        self.m
    }
}

impl Depth {
    pub fn zero() -> Self {
        Self { m: 0. }
    }
    pub fn from_meters(val: DepthType) -> Self {
        Self { m: val }
    }
    pub fn from_feet(val: DepthType) -> Self {
        Self {
            m: Self::ft_to_m(val),
        }
    }
    pub fn as_meters(&self) -> DepthType {
        self.m
    }
    pub fn as_feet(&self) -> DepthType {
        Self::m_to_ft(self.m)
    }
    fn m_to_ft(m: DepthType) -> DepthType {
        m * 3.28084
    }
    fn ft_to_m(ft: DepthType) -> DepthType {
        ft * 0.3048
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn m_to_ft() {
        let depth = Depth::from_meters(1.);
        let ft = depth.as_feet();
        assert_eq!(ft, 3.28084);
    }

    #[test]
    fn ft_to_m() {
        let depth = Depth::from_feet(100.);
        let m = depth.as_meters();
        assert_eq!(m, 30.48);
    }

    #[test]
    fn depth_conversion_factors() {
        let depth = Depth::from_meters(1.);
        let ft = depth.as_feet();
        let new_depth = Depth::from_feet(ft);
        let m = new_depth.as_meters();
        assert_eq!(with_precision(m, 5), 1.);
    }

    #[test]
    fn from_units_constructor() {
        let depth_m = Depth::from_units(1., Units::Metric);
        assert_eq!(depth_m.as_meters(), 1.);
        assert_eq!(depth_m.as_feet(), 3.28084);

        let depth_ft = Depth::from_units(1., Units::Imperial);
        assert_eq!(with_precision(depth_ft.as_feet(), 5), 1.);
        assert_eq!(depth_ft.as_meters(), 0.3048);
    }

    fn with_precision(x: f32, precision: u32) -> f32 {
        let d = 10_u32.pow(precision) as f32;
        (x * d).round() / d
    }
}
