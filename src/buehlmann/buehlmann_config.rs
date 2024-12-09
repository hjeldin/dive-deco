use crate::{
    common::{
        AscentRatePerMinute, ConfigValidationErr, ConfigValidationErrorReason, ConfigValidationErrorField, DecoModelConfig, GradientFactors, MbarPressure,
    },
    CeilingType,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BuehlmannConfig {
    pub gf: GradientFactors,
    pub surface_pressure: MbarPressure,
    pub deco_ascent_rate: AscentRatePerMinute,
    pub ceiling_type: CeilingType,
    pub round_ceiling: bool,
    pub recalc_all_tissues_m_values: bool,
}

impl BuehlmannConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_gradient_factors(mut self, gf_low: u8, gf_high: u8) -> Self {
        self.gf = (gf_low, gf_high);
        self
    }

    pub fn with_surface_pressure(mut self, surface_pressure: MbarPressure) -> Self {
        self.surface_pressure = surface_pressure;
        self
    }

    pub fn with_deco_ascent_rate(mut self, deco_ascent_rate: AscentRatePerMinute) -> Self {
        self.deco_ascent_rate = deco_ascent_rate;
        self
    }

    pub fn with_ceiling_type(mut self, ceiling_type: CeilingType) -> Self {
        self.ceiling_type = ceiling_type;
        self
    }

    pub fn with_round_ceiling(mut self, round_ceiling: bool) -> Self {
        self.round_ceiling = round_ceiling;
        self
    }

    pub fn with_all_m_values_recalculated(mut self, recalc_all_tissues_m_values: bool) -> Self {
        self.recalc_all_tissues_m_values = recalc_all_tissues_m_values;
        self
    }
}

impl Default for BuehlmannConfig {
    fn default() -> Self {
        Self {
            gf: (100, 100),
            surface_pressure: 1013,
            deco_ascent_rate: 10.,
            ceiling_type: CeilingType::Actual,
            round_ceiling: false,
            recalc_all_tissues_m_values: true,
        }
    }
}

impl DecoModelConfig for BuehlmannConfig {
    fn validate(&self) -> Result<(), ConfigValidationErr> {
        let Self {
            gf,
            surface_pressure,
            deco_ascent_rate,
            ..
        } = self;

        self.validate_gradient_factors(gf)?;
        self.validate_surface_pressure(surface_pressure)?;
        self.validate_deco_ascent_rate(deco_ascent_rate)?;

        Ok(())
    }

    fn surface_pressure(&self) -> MbarPressure {
        self.surface_pressure
    }

    fn deco_ascent_rate(&self) -> AscentRatePerMinute {
        self.deco_ascent_rate
    }

    fn ceiling_type(&self) -> CeilingType {
        self.ceiling_type
    }

    fn round_ceiling(&self) -> bool {
        self.round_ceiling
    }
}

impl BuehlmannConfig {
    fn validate_gradient_factors(&self, gf: &GradientFactors) -> Result<(), ConfigValidationErr> {
        let (gf_low, gf_high) = gf;
        let gf_range = 1..=100;

        if !gf_range.contains(gf_low) || !gf_range.contains(gf_high) {
            return Err(ConfigValidationErr::new(ConfigValidationErrorField::GradientFactors, ConfigValidationErrorReason::GF_RANGE_ERR_MSG));
        }

        if gf_low > gf_high {
            return Err(ConfigValidationErr::new(ConfigValidationErrorField::GradientFactors, ConfigValidationErrorReason::GF_ORDER_ERR_MSG));
        }

        Ok(())
    }

    fn validate_surface_pressure(
        &self,
        surface_pressure: &MbarPressure,
    ) -> Result<(), ConfigValidationErr> {
        let mbar_pressure_range = 500..=1500;
        if !mbar_pressure_range.contains(surface_pressure) {
            return Err(ConfigValidationErr::new(
                ConfigValidationErrorField::SurfacePressure,
                ConfigValidationErrorReason::SURFACE_PRESSURE_ERR_MSG,
            ));
        }

        Ok(())
    }

    fn validate_deco_ascent_rate(
        &self,
        deco_ascent_rate: &AscentRatePerMinute,
    ) -> Result<(), ConfigValidationErr> {
        let ascent_rate_range = 1.0..=30.0;
        if !ascent_rate_range.contains(deco_ascent_rate) {
            return Err(ConfigValidationErr::new(
                ConfigValidationErrorField::DecoAscentRate,
                ConfigValidationErrorReason::DECO_ASCENT_RATE_ERR_MSG,
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BuehlmannConfig::default();
        assert_eq!(config.validate(), Ok(()));
        assert_eq!(config.gf, (100, 100));
        assert_eq!(config.deco_ascent_rate, 10.);
        assert_eq!(config.ceiling_type, CeilingType::Actual);
        assert!(!config.round_ceiling);
    }

    #[test]
    fn test_variable_gradient_factors() {
        let config = BuehlmannConfig::new().with_gradient_factors(30, 70);
        assert_eq!(config.validate(), Ok(()));
        assert_eq!(config.gf, (30, 70));
    }

    #[test]
    fn test_gf_range() {
        let invalid_gf_range_cases = [(1, 101), (0, 99), (120, 240)];
        for case in invalid_gf_range_cases {
            let (gf_low, gf_high) = case;
            let config = BuehlmannConfig::new().with_gradient_factors(gf_low, gf_high);
            assert_eq!(
                config.validate(),
                Err(ConfigValidationErr::new(ConfigValidationErrorField::GradientFactors, ConfigValidationErrorReason::GF_RANGE_ERR_MSG))
            );
        }
    }

    #[test]
    fn test_gf_order() {
        let config = BuehlmannConfig::new().with_gradient_factors(90, 80);
        assert_eq!(
            config.validate(),
            Err(ConfigValidationErr::new(ConfigValidationErrorField::GradientFactors, ConfigValidationErrorReason::GF_ORDER_ERR_MSG))
        );
    }

    #[test]
    fn test_surface_pressure_config() {
        let config = BuehlmannConfig::new().with_surface_pressure(1032);
        assert_eq!(config.validate(), Ok(()));
        assert_eq!(config.surface_pressure, 1032);
    }

    #[test]
    fn test_invalid_surface_pressure_values() {
        let invalid_surface_pressure_cases = [0, 100, 2000];
        for invalid_case in invalid_surface_pressure_cases {
            let config = BuehlmannConfig::new().with_surface_pressure(invalid_case);
            assert_eq!(
                config.validate(),
                Err(ConfigValidationErr::new(
                    ConfigValidationErrorField::SurfacePressure,
                    ConfigValidationErrorReason::SURFACE_PRESSURE_ERR_MSG
                ))
            );
        }
    }

    #[test]
    fn test_deco_ascent_rate_config() {
        let config = BuehlmannConfig::new().with_deco_ascent_rate(15.5);
        assert_eq!(config.validate(), Ok(()));
        assert_eq!(config.deco_ascent_rate, 15.5);
    }

    #[test]
    fn test_invalid_deco_ascent_rate_values() {
        let invalid_deco_ascent_rate_cases = [-3., 0.5, 31.0, 50.5];
        for invalid_case in invalid_deco_ascent_rate_cases {
            let config = BuehlmannConfig::new().with_deco_ascent_rate(invalid_case);
            assert_eq!(
                config.validate(),
                Err(ConfigValidationErr::new(
                    ConfigValidationErrorField::DecoAscentRate,
                    ConfigValidationErrorReason::DECO_ASCENT_RATE_ERR_MSG
                ))
            );
        }
    }
}
