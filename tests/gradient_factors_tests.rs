use dive_deco::{ DecoModel, Depth, GradientFactors, Minutes };
pub mod fixtures;

#[test]
fn test_ndl() {
    // (gradient_factors, depth, expected_ndl)
    let test_cases: Vec<(GradientFactors, Depth, Minutes)> = vec![
        // 100/100
        ((100, 100), 21., 39),
        ((100, 100), 15., 86),

        // 70/70
        ((70, 70), 21., 18),
        ((70, 70), 15., 46),
    ];

    let air = fixtures::gas_air();
    for test_case in test_cases {
        let (gradient_factors, test_depth, expected_ndl) = test_case;
        let mut model = fixtures::model_gf(gradient_factors);
        model.step(&test_depth, &0, &air);
        assert_eq!(model.ndl(), expected_ndl);
    }
}
