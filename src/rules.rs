use crate::rules::RuleType::{Input, Output, Validation};

pub enum RuleType {
    Input,
    Validation,
    Output,
    Core
}
pub trait Rule {
    fn deps(&self) -> Vec<Self> where Self: Sized;
    fn get_type(&self) -> RuleType;
    fn get_specs_url(&self) -> &'static str;
}

#[derive(Clone)]
#[allow(non_c)]
pub enum RuleDef {
    AllPercentages,
    PercentagesStartsWithM,
    AllGram,
    Composite,
    MaxDetails,
    I_001_Zusammengesetzte_Zutaten,
    V_001_Menge_Immer_Benoetigt

}

impl Rule for RuleDef {
    fn deps(&self) -> Vec<Self> {
        match self {
            RuleDef::AllPercentages =>  vec![],
            RuleDef::PercentagesStartsWithM => vec![],
            RuleDef::AllGram => vec![],
            RuleDef::Composite => vec![],
            RuleDef::MaxDetails => vec![],
            RuleDef::I_001_Zusammengesetzte_Zutaten => vec![],
            RuleDef::V_001_Menge_Immer_Benoetigt => vec![]
        }
    }

    fn get_type(&self) -> RuleType {
        match self {
            RuleDef::AllPercentages => Validation,
            RuleDef::PercentagesStartsWithM => Output,
            RuleDef::AllGram => Output,
            RuleDef::Composite => Output,
            RuleDef::MaxDetails => Output,
            RuleDef::I_001_Zusammengesetzte_Zutaten => Input,
            RuleDef::V_001_Menge_Immer_Benoetigt => Validation
        }
    }

    fn get_specs_url(&self) -> &'static str {
        match self {
            RuleDef::I_001_Zusammengesetzte_Zutaten => "https://github.com/bar9/open-farming-hackdays-label-creator/issues/11",
            _ => ""
        }
    }
}

// struct RuleList(Vec<dyn Rule>);
// impl RuleList {
//
// }