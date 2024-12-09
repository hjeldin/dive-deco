use core::{cmp::Ordering, fmt};

use crate::{common::MAX_DECO_STAGE, DecoModel, Depth, DepthType, Gas, Time};

use super::{DecoModelConfig, DiveState, MbarPressure, Sim, MAX_GASSES};

// @todo move to model config
const DEFAULT_CEILING_WINDOW: DepthType = 3.;
const DEFAULT_MAX_END_DEPTH: DepthType = 30.;

#[derive(Copy, Clone, Debug, PartialEq)]
enum DecoAction {
    AscentToCeil,
    AscentToGasSwitchDepth,
    SwitchGas,
    Stop,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DecoStageType {
    Ascent,
    DecoStop,
    GasSwitch,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DecoStage {
    pub stage_type: DecoStageType,
    pub start_depth: Depth,
    pub end_depth: Depth,
    pub duration: Time,
    pub gas: Gas,
    pub valid: bool
}

impl Default for DecoStage {
    fn default() -> Self {
        Self {
            stage_type: DecoStageType::Ascent,
            start_depth: Depth::zero(),
            end_depth: Depth::zero(),
            duration: Time::zero(),
            gas: Gas::air(),
            valid: false
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Deco {
    deco_stages: [DecoStage; super::MAX_DECO_STAGE],
    tts: Time,
    sim: bool,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct DecoRuntime {
    // runtime
    pub deco_stages: [DecoStage; super::MAX_DECO_STAGE],
    // current TTS in minutes
    pub tts: Time,
    // TTS @+5 (TTS in 5 min given current depth and gas mix)
    pub tts_at_5: Time,
    // TTS Δ+5 (absolute change in TTS after 5 mins given current depth and gas mix)
    pub tts_delta_at_5: Time,
}

#[derive(Debug)]
struct MissedDecoStopViolation;

#[derive(Debug, PartialEq, Clone)]
pub enum DecoCalculationError {
    EmptyGasList,
    CurrentGasNotInList,
}

impl fmt::Display for DecoCalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            DecoCalculationError::EmptyGasList => {
                write!(f, "At least one available gas mix required")
            }
            DecoCalculationError::CurrentGasNotInList => write!(
                f,
                "Avaibalbe gas mixes must include current gas mix used by deco model"
            ),
        }
    }
}

impl Sim for Deco {
    fn fork(&self) -> Self {
        Self {
            sim: true,
            ..self.clone()
        }
    }
    fn is_sim(&self) -> bool {
        self.sim
    }
}

impl Deco {
    pub fn new_sim() -> Self {
        let deco = Self::default();
        deco.fork()
    }

    pub fn calc<T: DecoModel + Clone + Sim>(
        &mut self,
        deco_model: T,
        gas_mixes: [Gas; MAX_GASSES],
    ) -> Result<DecoRuntime, DecoCalculationError> {
        // validate gas mixes
        Self::validate_gas_mixes(&deco_model, &gas_mixes)?;

        // run model simulation until no deco stages
        let mut sim_model: T = deco_model.clone();
        let ascent_rate = sim_model.config().deco_ascent_rate();
        loop {
            let DiveState {
                depth: pre_stage_depth,
                time: pre_stage_time,
                gas: pre_stage_gas,
                ..
            } = sim_model.dive_state();
            let ceiling = sim_model.ceiling();

            // handle missed deco stop
            // if missed deco stop, override sim model to depth at the expected stop and rerun the calculation
            let next_deco_action = self.next_deco_action(&sim_model, gas_mixes.clone());
            if let Err(e) = next_deco_action {
                match e {
                    MissedDecoStopViolation => {
                        sim_model.record(
                            self.deco_stop_depth(ceiling),
                            Time::zero(),
                            &pre_stage_gas,
                        );
                        return self.calc(sim_model, gas_mixes);
                    }
                }
            }

            // handle deco actions
            let mut deco_stages: [DecoStage; MAX_DECO_STAGE] = [ DecoStage::default(); MAX_DECO_STAGE ];
            let (deco_action, next_switch_gas) = next_deco_action.unwrap();
            match deco_action {
                // deco obligation cleared
                None => {
                    break;
                }

                // handle mandatory deco stage
                Some(deco_action) => {
                    match deco_action {
                        // ascent to min depth (deco stop or surface)
                        DecoAction::AscentToCeil => {
                            sim_model.record_travel_with_rate(
                                self.deco_stop_depth(ceiling),
                                ascent_rate,
                                &pre_stage_gas,
                            );
                            let current_sim_state = sim_model.dive_state();
                            let current_sim_time = current_sim_state.time;
                            deco_stages.iter_mut().for_each(|stage| {
                                if !stage.valid {
                                    stage.valid = true;
                                    stage.stage_type = DecoStageType::Ascent;
                                    stage.start_depth = pre_stage_depth;
                                    stage.end_depth = current_sim_state.depth;
                                    stage.duration = current_sim_time - pre_stage_time;
                                    stage.gas = pre_stage_gas;
                                    return;
                                }
                            });
                        }

                        // ascent to min depth with gas switch on next deco gas maximum operating depth
                        DecoAction::AscentToGasSwitchDepth => {
                            // @todo unwrap and handler err
                            if let Some(next_switch_gas) = next_switch_gas {
                                // travel to MOD
                                let switch_gas_mod = next_switch_gas.max_operating_depth(1.6);
                                sim_model.record_travel_with_rate(
                                    switch_gas_mod,
                                    ascent_rate,
                                    &pre_stage_gas,
                                );
                                let DiveState {
                                    depth: post_ascent_depth,
                                    time: post_ascent_time,
                                    ..
                                } = sim_model.dive_state();
                                deco_stages.iter_mut().for_each(|stage| {
                                    if !stage.valid {
                                        stage.valid = true;
                                        stage.stage_type = DecoStageType::Ascent;
                                        stage.start_depth = pre_stage_depth;
                                        stage.end_depth = post_ascent_depth;
                                        stage.duration = post_ascent_time - pre_stage_time;
                                        stage.gas = pre_stage_gas;
                                        return;
                                    }
                                });

                                // switch gas @todo configurable gas change duration
                                sim_model.record(
                                    sim_model.dive_state().depth,
                                    Time::zero(),
                                    &next_switch_gas,
                                );
                                // @todo configurable oxygen window stop
                                let post_switch_state = sim_model.dive_state();

                                deco_stages.iter_mut().for_each(|stage| {
                                    if !stage.valid {
                                        stage.valid = true;
                                        stage.stage_type = DecoStageType::GasSwitch;
                                        stage.start_depth = post_ascent_depth;
                                        stage.end_depth = post_switch_state.depth;
                                        stage.duration = post_switch_state.time - post_ascent_time;
                                        stage.gas = next_switch_gas;
                                        return;
                                    }
                                });
                            }
                        }

                        // switch gas without ascent
                        DecoAction::SwitchGas => {
                            let switch_gas = next_switch_gas.unwrap();
                            // @todo configurable gas switch duration
                            sim_model.record(pre_stage_depth, Time::zero(), &switch_gas);
                            deco_stages.iter_mut().for_each(|stage| {
                                if !stage.valid {
                                    stage.valid = true;
                                    stage.stage_type = DecoStageType::GasSwitch;
                                    stage.start_depth = pre_stage_depth;
                                    stage.end_depth = pre_stage_depth;
                                    stage.duration = Time::zero();
                                    stage.gas = switch_gas;
                                    return;
                                }
                            });
                        }

                        // decompression stop (a series of 1s segments, merged into one on cleared stop)
                        DecoAction::Stop => {
                            sim_model.record(
                                pre_stage_depth,
                                Time::from_seconds(1.),
                                &pre_stage_gas,
                            );
                            let sim_state = sim_model.dive_state();
                            // @todo dedupe here on deco instead of of add deco
                            deco_stages.iter_mut().for_each(|stage| {
                                if !stage.valid {
                                    stage.valid = true;
                                    stage.stage_type = DecoStageType::DecoStop;
                                    stage.start_depth = pre_stage_depth;
                                    stage.end_depth = sim_state.depth;
                                    stage.duration = sim_state.time - pre_stage_time;
                                    stage.gas = sim_state.gas;
                                    return;
                                }
                            })
                        }
                    }
                }
            }
            // register deco stages
            deco_stages
                .into_iter()
                .for_each(|deco_stage| self.register_deco_stage(deco_stage));
        }

        let tts = self.tts;
        let mut tts_at_5 = Time::zero();
        let mut tts_delta_at_5 = Time::zero();
        if !self.is_sim() {
            let mut nested_sim_deco = Deco::new_sim();
            let mut nested_sim_model = deco_model.clone();
            let DiveState {
                depth: sim_depth,
                gas: sim_gas,
                ..
            } = nested_sim_model.dive_state();
            nested_sim_model.record(sim_depth, Time::from_minutes(5.), &sim_gas);
            let nested_deco = nested_sim_deco
                .calc(nested_sim_model, gas_mixes.clone())
                .unwrap();
            tts_at_5 = nested_deco.tts;
            tts_delta_at_5 = tts_at_5 as Time - tts as Time;
        }

        Ok(DecoRuntime {
            deco_stages: self.deco_stages.clone(),
            tts,
            tts_at_5,
            tts_delta_at_5,
        })
    }

    fn next_deco_action(
        &self,
        sim_model: &impl DecoModel,
        gas_mixes: [Gas; MAX_GASSES],
    ) -> Result<(Option<DecoAction>, Option<Gas>), MissedDecoStopViolation> {
        let DiveState {
            depth: current_depth,
            gas: current_gas,
            ..
        } = sim_model.dive_state();
        let surface_pressure = sim_model.config().surface_pressure();

        // end deco simulation - surface
        if current_depth <= Depth::zero() {
            return Ok((None, None));
        }

        let ceiling = sim_model.ceiling();

        match ceiling.partial_cmp(&Depth::zero()) {
            Some(Ordering::Equal | Ordering::Less) => Ok((Some(DecoAction::AscentToCeil), None)),
            Some(Ordering::Greater) => {
                // check if deco violation
                if current_depth < self.deco_stop_depth(ceiling) {
                    return Err(MissedDecoStopViolation);
                }

                let next_switch_gas =
                    self.next_switch_gas(current_depth, &current_gas, gas_mixes, surface_pressure);
                // check if within mod @todo min operational depth
                if let Some(switch_gas) = next_switch_gas {
                    //switch gas without ascent if within mod of next deco gas
                    let gas_mod = switch_gas.max_operating_depth(1.6);
                    let gas_end = switch_gas.equivalent_narcotic_depth(current_depth);
                    if (switch_gas != current_gas)
                        && (current_depth <= gas_mod)
                        && (gas_end <= Depth::from_meters(DEFAULT_MAX_END_DEPTH))
                    {
                        return Ok((Some(DecoAction::SwitchGas), Some(switch_gas)));
                    }
                }

                // check if within or below deco stop window
                let ceiling_padding = current_depth - ceiling;
                if ceiling_padding <= Depth::from_meters(DEFAULT_CEILING_WINDOW) {
                    Ok((Some(DecoAction::Stop), None))
                } else {
                    // ascent to next gas switch depth if next gas' MOD below ceiling
                    if let Some(next_switch_gas) = next_switch_gas {
                        if next_switch_gas.max_operating_depth(1.6) >= ceiling {
                            return Ok((
                                Some(DecoAction::AscentToGasSwitchDepth),
                                Some(next_switch_gas),
                            ));
                        }
                    }
                    Ok((Some(DecoAction::AscentToCeil), None))
                }
            }
            None => panic!("Ceiling and depth uncomparable"),
        }
    }

    /// check next deco gas in deco (the one with lowest MOD while more oxygen-rich than current)
    fn next_switch_gas(
        &self,
        current_depth: Depth,
        current_gas: &Gas,
        gas_mixes: [Gas; MAX_GASSES],
        surface_pressure: MbarPressure,
    ) -> Option<Gas> {
        let current_gas_partial_pressures =
            current_gas.partial_pressures(current_depth, surface_pressure);
        // all potential deco gases that are more oxygen-rich than current (inc. trimix / heliox)
        // let mut switch_gasses: [Gas; MAX_GASSES] = gas_mixes
        //     .into_iter()
        //     .filter(|gas| {
        //         let partial_pressures = gas.partial_pressures(current_depth, surface_pressure);
        //         partial_pressures.o2 > current_gas_partial_pressures.o2
        //     })
        //     .collect();
        let mut switch_gasses: [Gas; MAX_GASSES] = gas_mixes.clone();
        let mut i = 0;
        for gas in gas_mixes.iter() {
            let partial_pressures = gas.partial_pressures(current_depth, surface_pressure);
            if partial_pressures.o2 > current_gas_partial_pressures.o2 {
                switch_gasses[i] = *gas;
                i += 1;
            }
        }
        // sort deco gasses by o2 content
        // switch_gasses[..i].sort_by(|a, b| {
        //     let x = a.gas_pressures_compound(1.);
        //     let y = b.gas_pressures_compound(1.);
        //     x.o2.partial_cmp(&y.o2).unwrap()
        // });
        let mut j = 0;
        panic!("switch_gasses: {:?}", switch_gasses);
        for gas in switch_gasses.iter_mut() {
            let x = gas.gas_pressures_compound(1.);
            let y = gas.gas_pressures_compound(1.);
            if x.o2 > y.o2 {
                let temp = switch_gasses[j];
                switch_gasses[j] = *gas;
                *gas = temp;
                j += 1;
            }
        }

        // mix with lowest MOD (by absolute o2 content)
        switch_gasses.first().copied()
    }

    fn register_deco_stage(&mut self, stage: DecoStage) {
        // dedupe iterative deco stops and merge into one
        let mut push_new = true;
        let last_stage = self.deco_stages.last_mut();
        if let Some(last_stage) = last_stage {
            if last_stage.stage_type == stage.stage_type {
                last_stage.duration += stage.duration;
                last_stage.end_depth = stage.end_depth;
                push_new = false;
            }
        }
        if push_new {
            self.deco_stages.iter_mut().for_each(|stage| {
                if !stage.valid {
                    stage.valid = true;
                    stage.stage_type = stage.stage_type;
                    stage.start_depth = stage.start_depth;
                    stage.end_depth = stage.end_depth;
                    stage.duration = stage.duration;
                    stage.gas = stage.gas;
                    return;
                }
            });
        }

        // increment TTS by deco stage duration
        self.tts += stage.duration;
    }

    // round ceiling up to the bottom of deco window
    fn deco_stop_depth(&self, ceiling: Depth) -> Depth {
        Depth::from_meters(
            DEFAULT_CEILING_WINDOW * libm::ceil(ceiling.as_meters() as f64 / DEFAULT_CEILING_WINDOW as f64) as f32,
        )
    }

    fn validate_gas_mixes<T: DecoModel>(
        deco_model: &T,
        gas_mixes: &[Gas],
    ) -> Result<(), DecoCalculationError> {
        if gas_mixes.is_empty() {
            return Err(DecoCalculationError::EmptyGasList);
        }
        if gas_mixes.iter().all(|gas| !gas.is_valid()) {
            return Err(DecoCalculationError::EmptyGasList);
        }
        let current_gas = deco_model.dive_state().gas;
        let current_gas_in_available = gas_mixes.iter().find(|gas_mix| **gas_mix == current_gas);
        if current_gas_in_available.is_none() {
            return Err(DecoCalculationError::CurrentGasNotInList);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BuehlmannModel;

    #[test]
    fn test_ceiling_rounding() {
        let test_cases: [(DepthType, DepthType); 6] = [
            (0., 0.),
            (2., 3.),
            (2.999, 3.),
            (3., 3.),
            (3.00001, 6.),
            (12., 12.),
        ];
        let deco = Deco::default();
        for case in test_cases.into_iter() {
            let (input_depth, expected_depth) = case;
            let res = deco.deco_stop_depth(Depth::from_meters(input_depth));
            assert_eq!(res, Depth::from_meters(expected_depth));
        }
    }

    fn build_gasses(gas: Gas) -> [Gas; MAX_GASSES] {
        let mut gasses = [Gas::default(); MAX_GASSES];
        gasses[0] = gas;
        gasses
    }

    fn build_2gasses(gas: Gas, gas2: Gas) -> [Gas; MAX_GASSES] {
        let mut gasses = [Gas::default(); MAX_GASSES];
        gasses[0] = gas;
        gasses[1] = gas2;
        gasses
    }

    fn build_3gasses(gas: Gas, gas2: Gas, gas3: Gas) -> [Gas; MAX_GASSES] {
        let mut gasses = [Gas::default(); MAX_GASSES];
        gasses[0] = gas;
        gasses[1] = gas2;
        gasses[2] = gas3;
        gasses
    }

    #[test]
    fn test_next_switch_gas() {
        let air = Gas::air();
        let ean_50 = Gas::new(0.5, 0.);
        let oxygen = Gas::new(1., 0.);
        let trimix = Gas::new(0.5, 0.2);

        // potential switch if in deco!
        // [ (current_depth, current_gas, gas_mixes, expected_result) ]
        // @todo depth constructor in test cases
        let test_cases: [(DepthType, Gas, [Gas; MAX_GASSES], Option<Gas>); 6] = [
            // single gas air
            (10., air, build_gasses(air), None),
            // air + ean50 within MOD
            (10., air, build_2gasses(air, ean_50), Some(ean_50)),
            // air + ean50 over MOD
            (30., air, build_2gasses(air, ean_50), Some(ean_50)),
            // air + ean50 + oxygen, ean50 withing MOD, oxygen out
            (20., air, build_3gasses(air, ean_50, oxygen), Some(ean_50)),
            // air + ean50 + oxy, deco on ean50, oxygen within MOD
            (5.5, ean_50, build_3gasses(air, ean_50, oxygen), Some(oxygen)),
            // air + heliox within o2 MOD, not considered deco gas
            (30., air, build_2gasses(air, trimix), Some(trimix)),
        ];

        let deco = Deco::default();
        for case in test_cases.into_iter() {
            let (current_depth, current_gas, available_gas_mixes, expected_switch_gas) = case;
            let res = deco.next_switch_gas(
                Depth::from_meters(current_depth),
                &current_gas,
                available_gas_mixes,
                1000,
            );
            assert_eq!(res, expected_switch_gas);
        }
    }

    #[test]
    fn should_err_on_empty_gas_mixes() {
        let mut deco = Deco::default();
        let deco_model = BuehlmannModel::default();
        let deco_res = deco.calc(deco_model, [Gas::default(); 16]);
        assert_eq!(deco_res, Err(DecoCalculationError::EmptyGasList));
    }

    #[test]
    fn should_err_on_gas_mixes_without_current_mix() {
        let mut deco = Deco::default();
        let mut deco_model = BuehlmannModel::default();
        let air = Gas::air();
        let ean50 = Gas::new(0.50, 0.);
        let tmx2135 = Gas::new(0.21, 0.35);
        deco_model.record_travel_with_rate(Depth::from_meters(40.), 10., &air);
        let deco_res = deco.calc(deco_model, build_2gasses(ean50, tmx2135));
        assert_eq!(deco_res, Err(DecoCalculationError::CurrentGasNotInList));
    }
}
