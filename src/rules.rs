// pub enum RuleType {
//     Input,
//     Validation,
//     Output,
//     // Core
// }
// pub trait Rule {
// fn deps(&self) -> Vec<Self> where Self: Sized;
// fn get_type(&self) -> RuleType;
// fn get_specs_url(&self) -> &'static str;
// }

#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum RuleDef {
    AllPercentages,
    PercentagesStartsWithM,
    AllGram,
    // Composite,
    // MaxDetails,
    // I_001_Zusammengesetzte_Zutaten,
    AP1_1_ZutatMengeValidierung,
    AP1_2_ProzentOutputNamensgebend,
    AP1_3_EingabeNamensgebendeZutat,
    AP1_4_ManuelleEingabeTotal,
    AP2_1_ZusammegesetztOutput,
}

// impl Rule for RuleDef {
// fn deps(&self) -> Vec<Self> {
//     match self {
//         RuleDef::AP1_2_ProzentOutputNamensgebend => vec![
//             AP1_3_EingabeNamensgebendeZutat
//         ],
//         _ => vec![]
//     }
// }

// fn get_type(&self) -> RuleType {
//     match self {
//         RuleDef::AllPercentages => Validation,
//         RuleDef::PercentagesStartsWithM => Output,
//         RuleDef::AllGram => Output,
//         RuleDef::Composite => Output,
//         RuleDef::MaxDetails => Output,
//         RuleDef::I_001_Zusammengesetzte_Zutaten => Input,
//         RuleDef::AP1_1_ZutatMengeValidierung => Validation,
//         RuleDef::AP1_2_ProzentOutputNamensgebend => Output,
//         RuleDef::AP1_3_EingabeNamensgebendeZutat => Input,
//         RuleDef::AP1_4_ManuelleEingabeTotal => Input,
//         RuleDef::AP2_1_ZusammegesetztOutput => Output,
//     }
// }

// fn get_specs_url(&self) -> &'static str {
//     match self {
//         RuleDef::I_001_Zusammengesetzte_Zutaten => "https://github.com/bar9/open-farming-hackdays-label-creator/issues/11",
//         _ => ""
//     }
// }
// }
