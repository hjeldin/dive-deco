use core::{
    cmp::Ordering,
    ops::{Add, AddAssign, Div, Mul, Sub},
};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Time {
    s: f32,
}

impl Add for Time {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self { s: self.s + rhs.s }
    }
}
impl Sub for Time {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self { s: self.s - rhs.s }
    }
}
impl AddAssign for Time {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self { s: self.s + rhs.s }
    }
}
impl Mul<Self> for Time {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self { s: self.s * rhs.s }
    }
}
impl Mul<u8> for Time {
    type Output = Self;
    fn mul(self, rhs: u8) -> Self::Output {
        Self {
            s: self.s * rhs as f32,
        }
    }
}
impl Div<Self> for Time {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self { s: self.s / rhs.s }
    }
}
impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.s.partial_cmp(&other.s)
    }
}

impl Time {
    pub fn from_seconds(val: f32) -> Self {
        Self { s: val }
    }
    pub fn from_minutes(val: f32) -> Self {
        Self { s: val * 60. }
    }
    pub fn zero() -> Self {
        Self { s: 0. }
    }
    pub fn as_seconds(&self) -> f32 {
        self.s
    }
    pub fn as_minutes(&self) -> f32 {
        self.s / 60.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_seconds() {
        let time = Time::from_seconds(120.0);
        assert_eq!(time.as_seconds(), 120.0);
    }

    #[test]
    fn test_from_minutes() {
        let time = Time::from_minutes(2.0);
        assert_eq!(time.as_seconds(), 120.0);
    }

    #[test]
    fn test_as_seconds() {
        let time = Time::from_minutes(2.);
        assert_eq!(time.as_seconds(), 120.);
    }

    #[test]
    fn test_as_minutes() {
        let time = Time::from_seconds(30.0);
        assert_eq!(time.as_minutes(), 0.5);
    }
}
