use dive_deco::{
    BuehlmannConfig, BuehlmannModel, CeilingType, DecoModel, DecoRuntime, DecoStage, DecoStageType,
    Depth, Gas, Time,
};

pub mod fixtures;

#[test]
fn test_deco_ascent_no_deco() {
    let air = fixtures::gas_air();
    let mut model = fixtures::model_default();
    model.record(Depth::from_meters(20.), Time::from_minutes(5.), &air);

    let DecoRuntime {
        deco_stages, tts, ..
    } = model.deco(build_gasses(air)).unwrap();
    assert_eq!(deco_stages.len(), 1); // single continuous ascent
    assert_eq!(tts, Time::from_minutes(2.)); // tts in minutes
}

#[test]
fn test_deco_single_gas() {
    let air = fixtures::gas_air();
    let mut model = BuehlmannModel::new(BuehlmannConfig::default().with_deco_ascent_rate(9.));
    model.record(Depth::from_meters(40.), Time::from_minutes(20.), &air);

    let DecoRuntime {
        deco_stages, tts, ..
    } = model.deco(build_gasses(air)).unwrap();

    assert_eq!(tts, Time::from_seconds(754.));
    assert_eq!(deco_stages.len(), 5);

    let expected_deco_stages = vec![
        DecoStage {
            stage_type: DecoStageType::Ascent,
            start_depth: Depth::from_meters(40.0),
            end_depth: Depth::from_meters(6.0),
            duration: Time::from_seconds(226.),
            gas: air,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::DecoStop,
            start_depth: Depth::from_meters(6.0),
            end_depth: Depth::from_meters(6.0),
            duration: Time::from_seconds(88.),
            gas: air,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::Ascent,
            start_depth: Depth::from_meters(6.0),
            end_depth: Depth::from_meters(3.0),
            duration: Time::from_seconds(20.),
            gas: air,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::DecoStop,
            start_depth: Depth::from_meters(3.0),
            end_depth: Depth::from_meters(3.0),
            duration: Time::from_seconds(400.),
            gas: air,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::Ascent,
            start_depth: Depth::from_meters(3.0),
            end_depth: Depth::from_meters(0.0),
            duration: Time::from_seconds(20.),
            gas: air,
            valid: true,
        },
    ];

    assert_deco_stages_eq(deco_stages.to_vec(), expected_deco_stages);
}

const MAX_GASSES: usize = 16;

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
fn test_deco_multi_gas() {
    let mut model = BuehlmannModel::new(BuehlmannConfig::default().with_deco_ascent_rate(9.));

    let air = Gas::new(0.21, 0.);
    let ean_50 = Gas::new(0.50, 0.);

    model.record(Depth::from_meters(40.), Time::from_minutes(20.), &air);

    let DecoRuntime {
        deco_stages, tts, ..
    } = model.deco(build_2gasses(air, ean_50)).unwrap();

    let expected_deco_stages = vec![
        DecoStage {
            stage_type: DecoStageType::Ascent,
            start_depth: Depth::from_meters(40.),
            end_depth: Depth::from_meters(22.),
            duration: Time::from_seconds(120.),
            gas: air,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::GasSwitch,
            start_depth: Depth::from_meters(22.0),
            end_depth: Depth::from_meters(22.0),
            duration: Time::zero(),
            gas: ean_50,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::Ascent,
            start_depth: Depth::from_meters(22.),
            end_depth: Depth::from_meters(6.),
            duration: Time::from_seconds(106.),
            gas: ean_50,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::DecoStop,
            start_depth: Depth::from_meters(6.0),
            end_depth: Depth::from_meters(6.0),
            duration: Time::from_seconds(34.),
            gas: ean_50,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::Ascent,
            start_depth: Depth::from_meters(6.0),
            end_depth: Depth::from_meters(3.0),
            duration: Time::from_seconds(20.),
            gas: ean_50,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::DecoStop,
            start_depth: Depth::from_meters(3.0),
            end_depth: Depth::from_meters(3.0),
            duration: Time::from_seconds(291.),
            gas: ean_50,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::Ascent,
            start_depth: Depth::from_meters(3.0),
            end_depth: Depth::from_meters(0.0),
            duration: Time::from_seconds(20.),
            gas: ean_50,
            valid: true,
        },
    ];

    assert_deco_stages_eq(deco_stages.to_vec(), expected_deco_stages);
    assert_eq!(tts, Time::from_seconds(591.));
}

#[test]
fn test_deco_with_deco_mod_at_bottom() {
    let mut model = BuehlmannModel::new(BuehlmannConfig::default().with_deco_ascent_rate(9.));
    let air = Gas::air();
    let ean_36 = Gas::new(0.36, 0.);

    model.record(Depth::from_meters(30.), Time::from_minutes(30.), &air);

    let DecoRuntime {
        deco_stages, tts, ..
    } = model.deco(build_2gasses(air, ean_36)).unwrap();

    let expected_deco_stages = vec![
        DecoStage {
            stage_type: DecoStageType::GasSwitch,
            start_depth: Depth::from_meters(30.0),
            end_depth: Depth::from_meters(30.0),
            duration: Time::zero(),
            gas: ean_36,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::Ascent,
            start_depth: Depth::from_meters(30.0),
            end_depth: Depth::from_meters(3.0),
            duration: Time::from_seconds(180.),
            gas: ean_36,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::DecoStop,
            start_depth: Depth::from_meters(3.0),
            end_depth: Depth::from_meters(3.0),
            duration: Time::from_seconds(268.),
            gas: ean_36,
            valid: true,
        },
        DecoStage {
            stage_type: DecoStageType::Ascent,
            start_depth: Depth::from_meters(3.0),
            end_depth: Depth::from_meters(0.0),
            duration: Time::from_seconds(20.),
            gas: ean_36,
            valid: true,
        },
    ];
    assert_deco_stages_eq(deco_stages.to_vec(), expected_deco_stages);
    assert_eq!(tts, Time::from_seconds(468.));
}

#[test]
fn test_tts_delta() {
    let mut model = fixtures::model_gf((30, 70));
    let air = Gas::air();
    let ean_50 = Gas::new(0.5, 0.);
    let gas_mixes = build_2gasses(air, ean_50);
    model.record(Depth::from_meters(40.), Time::from_minutes(20.), &air);
    let deco_1 = model.deco(gas_mixes.clone()).unwrap();
    model.record(Depth::from_meters(40.), Time::from_minutes(5.), &air);
    let deco_2 = model.deco(gas_mixes).unwrap();
    assert_eq!(deco_1.tts_at_5, deco_2.tts);
    assert_eq!(deco_1.tts_delta_at_5, (deco_2.tts - deco_1.tts) as Time);
}

#[test]
fn test_runtime_on_missed_stop() {
    let air = Gas::air();
    let ean_50 = Gas::new(0.50, 0.);
    let available_gas_mixes = build_2gasses(air, ean_50);

    let configs = vec![
        BuehlmannConfig::default()
            .with_ceiling_type(dive_deco::CeilingType::Actual)
            .with_gradient_factors(30, 70),
        BuehlmannConfig::default()
            .with_ceiling_type(dive_deco::CeilingType::Adaptive)
            .with_gradient_factors(30, 70),
    ];

    for config in configs.into_iter() {
        let mut model = BuehlmannModel::new(config);
        model.record(Depth::from_meters(40.), Time::from_minutes(30.), &air);
        model.record(Depth::from_meters(22.), Time::zero(), &air);
        let initial_deco = model.deco(available_gas_mixes.clone()).unwrap();
        // 21
        let initial_deco_stop_depth = get_first_deco_stop_depth(initial_deco);

        // between stop and ceiling (18 - 21)
        model.record(Depth::from_meters(20.), Time::zero(), &air);
        let between_deco = model.deco(available_gas_mixes.clone()).unwrap();
        let between_deco_stop_depth = get_first_deco_stop_depth(between_deco);

        // below
        model.record(Depth::from_meters(15.), Time::zero(), &air);
        let below_deco = model.deco(available_gas_mixes.clone()).unwrap();
        let below_deco_stop_depth = get_first_deco_stop_depth(below_deco);

        assert_eq!(
            initial_deco_stop_depth, between_deco_stop_depth,
            "below deco stop, above ceiling"
        );
        assert_eq!(
            initial_deco_stop_depth, below_deco_stop_depth,
            "below ceiling"
        );
    }
}

#[test]
fn test_deco_runtime_integrity() {
    let config = BuehlmannConfig::new()
        .with_gradient_factors(30, 70)
        .with_ceiling_type(CeilingType::Adaptive);
    let mut model = BuehlmannModel::new(config);
    let air = Gas::air();
    let ean_50 = Gas::new(0.50, 0.);
    let oxygen = Gas::new(1., 0.);
    model.record(Depth::from_meters(40.), Time::from_minutes(20.), &air);

    let deco_runtime = model.deco(build_3gasses(air, ean_50, oxygen)).unwrap();
    let deco_stages = deco_runtime.deco_stages;

    deco_stages.iter().reduce(|a, b| {
        // validate depth order
        assert!(
            b.start_depth == a.end_depth,
            "next stage start depth ({:?}@{}m) should equal previous stage end depth ({:?}@{}m)",
            b.stage_type,
            b.start_depth,
            a.stage_type,
            a.end_depth
        );
        // validate gas switch MOD
        if a.stage_type == DecoStageType::GasSwitch {
            let gas_switch_target_mod = a.gas.max_operating_depth(1.6);
            assert!(a.start_depth <= gas_switch_target_mod);
        }
        b
    });
}

fn get_first_deco_stop_depth(deco: DecoRuntime) -> Option<Depth> {
    let first_stop = deco
        .deco_stages
        .into_iter()
        .find(|stage| stage.stage_type == DecoStageType::DecoStop);
    if let Some(stop) = first_stop {
        return Some(stop.start_depth);
    }
    None
}

fn assert_deco_stages_eq(deco_stages: Vec<DecoStage>, expected_deco_stages: Vec<DecoStage>) {
    assert_eq!(deco_stages.len(), expected_deco_stages.len());
    for (i, expected_stage) in expected_deco_stages.iter().enumerate() {
        assert_eq!(deco_stages[i].stage_type, expected_stage.stage_type);
        assert_eq!(deco_stages[i].start_depth, expected_stage.start_depth);
        assert_eq!(deco_stages[i].end_depth, expected_stage.end_depth);
        assert_eq!(deco_stages[i].duration, expected_stage.duration);
        assert_eq!(deco_stages[i].gas, expected_stage.gas);
    }
}
