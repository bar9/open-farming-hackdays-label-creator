/// Types of rules that can be applied in the label generation process
#[derive(Clone, Debug, PartialEq)]
pub enum RuleType {
    /// Rules that control input form elements and data transformation
    Input,
    /// Rules that validate form data and generate validation messages
    Validation,
    /// Rules that control how data is formatted on the output label
    Output,
    /// Rules that control conditional display of UI elements
    Conditional,
}

/// Trait for rules that can be applied during label generation
pub trait Rule {
    /// Returns the dependencies this rule requires to function correctly
    fn deps(&self) -> Vec<Self>
    where
        Self: Sized;
    /// Returns the category/type of this rule
    fn get_type(&self) -> RuleType;
    /// Returns a URL to the specification or documentation for this rule
    fn get_specs_url(&self) -> Option<&'static str>;
    /// Returns a human-readable description of what this rule does
    fn get_description(&self) -> &'static str;
}

/// Definition of all available rules in the system
///
/// Swiss food labeling rules follow the format AP{section}_{rule}_{description}
/// where section corresponds to Swiss food labeling law sections.
#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum RuleDef {
    /// Display percentage for all ingredients on labels (debugging/testing rule)
    AllPercentages,
    /// Display percentage only for ingredients starting with "M" (testing rule)
    PercentagesStartsWithM,
    /// Display weight in grams for all ingredients (testing rule)
    AllGram,

    // Swiss regulation compliance rules
    /// AP1.1: Validates that all ingredient amounts are greater than 0
    AP1_1_ZutatMengeValidierung,
    /// AP1.2: Shows percentage for name-giving ingredients on the label
    AP1_2_ProzentOutputNamensgebend,
    /// AP1.3: Enables input of name-giving ingredients in the UI
    AP1_3_EingabeNamensgebendeZutat,
    /// AP1.4: Enables manual input of total product weight
    AP1_4_ManuelleEingabeTotal,
    /// AP2.1: Shows composite ingredients with their sub-components
    AP2_1_ZusammegesetztOutput,
    /// AP7.1: Requires country of origin for ingredients >50% of total weight (Swiss requirement)
    AP7_1_HerkunftBenoetigtUeber50Prozent,
    /// AP7.3: Requires country of origin for meat ingredients >20% of total weight (Swiss requirement)
    AP7_3_HerkunftFleischUeber20Prozent,
    /// AP7.4: Requires birthplace and slaughter location for beef ingredients (Swiss requirement)
    AP7_4_RindfleischHerkunftDetails,
    /// AP7.5: Requires catch location for fish ingredients (Swiss requirement)
    AP7_5_FischFangort,
    /// Bio/Knospe: Requires country of origin for ALL ingredients (Bio/Knospe requirement)
    Bio_Knospe_AlleZutatenHerkunft,
    /// Knospe: When 100% of agricultural ingredients are from Switzerland, no origin display needed
    Knospe_100_Percent_CH_NoOrigin,
    /// Knospe: When 90-99.99% of agricultural ingredients are from Switzerland, show origin for Swiss ingredients
    Knospe_90_99_Percent_CH_ShowOrigin,
    /// Knospe: When <90% of agricultural ingredients are from Switzerland, show origin based on specific ingredient criteria
    Knospe_Under90_Percent_CH_IngredientRules,
    /// Bio/Knospe: Enables input of whether each ingredient is bio-certified
    Bio_Knospe_EingabeIstBio,
    /// Knospe: Shows Bio Suisse logo based on Swiss ingredient percentage
    Knospe_ShowBioSuisseLogo,
    /// Bio/Knospe: Requires certification body for Bio and Knospe products
    Bio_Knospe_ZertifizierungsstellePflicht,
}

impl Rule for RuleDef {
    fn deps(&self) -> Vec<Self> {
        match self {
            RuleDef::AP1_2_ProzentOutputNamensgebend => {
                vec![RuleDef::AP1_3_EingabeNamensgebendeZutat]
            }
            _ => vec![],
        }
    }

    fn get_type(&self) -> RuleType {
        match self {
            RuleDef::AllPercentages => RuleType::Output,
            RuleDef::PercentagesStartsWithM => RuleType::Output,
            RuleDef::AllGram => RuleType::Output,
            RuleDef::AP1_1_ZutatMengeValidierung => RuleType::Validation,
            RuleDef::AP1_2_ProzentOutputNamensgebend => RuleType::Output,
            RuleDef::AP1_3_EingabeNamensgebendeZutat => RuleType::Conditional,
            RuleDef::AP1_4_ManuelleEingabeTotal => RuleType::Conditional,
            RuleDef::AP2_1_ZusammegesetztOutput => RuleType::Output,
            RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent => RuleType::Conditional,
            RuleDef::AP7_3_HerkunftFleischUeber20Prozent => RuleType::Conditional,
            RuleDef::AP7_4_RindfleischHerkunftDetails => RuleType::Conditional,
            RuleDef::AP7_5_FischFangort => RuleType::Conditional,
            RuleDef::Bio_Knospe_AlleZutatenHerkunft => RuleType::Validation,
            RuleDef::Knospe_100_Percent_CH_NoOrigin => RuleType::Output,
            RuleDef::Knospe_90_99_Percent_CH_ShowOrigin => RuleType::Output,
            RuleDef::Knospe_Under90_Percent_CH_IngredientRules => RuleType::Output,
            RuleDef::Bio_Knospe_EingabeIstBio => RuleType::Conditional,
            RuleDef::Knospe_ShowBioSuisseLogo => RuleType::Conditional,
            RuleDef::Bio_Knospe_ZertifizierungsstellePflicht => RuleType::Validation,
        }
    }

    fn get_specs_url(&self) -> Option<&'static str> {
        match self {
            RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent => Some("https://www.blv.admin.ch/blv/de/home/lebensmittel-und-ernaehrung/rechts-und-vollzugsgrundlagen/hilfsmittel-vollzug.html"),
            RuleDef::AP7_3_HerkunftFleischUeber20Prozent => Some("https://www.blv.admin.ch/blv/de/home/lebensmittel-und-ernaehrung/rechts-und-vollzugsgrundlagen/hilfsmittel-vollzug.html"),
            RuleDef::AP7_4_RindfleischHerkunftDetails => Some("https://www.blv.admin.ch/blv/de/home/lebensmittel-und-ernaehrung/rechts-und-vollzugsgrundlagen/hilfsmittel-vollzug.html"),
            RuleDef::AP7_5_FischFangort => Some("https://www.blv.admin.ch/blv/de/home/lebensmittel-und-ernaehrung/rechts-und-vollzugsgrundlagen/hilfsmittel-vollzug.html"),
            _ => None
        }
    }

    fn get_description(&self) -> &'static str {
        match self {
            RuleDef::AllPercentages => "Zeige Prozentangaben für alle Zutaten auf dem Etikett",
            RuleDef::PercentagesStartsWithM => "Zeige Prozentangaben nur für Zutaten, die mit 'M' beginnen",
            RuleDef::AllGram => "Zeige Gewichtsangaben in Gramm für alle Zutaten",
            RuleDef::AP1_1_ZutatMengeValidierung => "Validiert, dass alle Zutatenmengen grösser als 0 sind",
            RuleDef::AP1_2_ProzentOutputNamensgebend => "Zeigt Prozentangabe für namensgebende Zutaten auf dem Etikett",
            RuleDef::AP1_3_EingabeNamensgebendeZutat => "Ermöglicht die Eingabe von namensgebenden Zutaten in der Benutzeroberfläche",
            RuleDef::AP1_4_ManuelleEingabeTotal => "Ermöglicht die manuelle Eingabe der Gesamtmenge",
            RuleDef::AP2_1_ZusammegesetztOutput => "Zeigt zusammengesetzte Zutaten mit ihren Bestandteilen auf dem Etikett",
            RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent => "Erfordert Herkunftsangabe für Zutaten, die mehr als 50% des Gesamtgewichts ausmachen (Schweizer Vorschrift)",
            RuleDef::AP7_3_HerkunftFleischUeber20Prozent => "Erfordert Herkunftsangabe für Fleisch-Zutaten, die mehr als 20% des Gesamtgewichts ausmachen (Schweizer Vorschrift)",
            RuleDef::AP7_4_RindfleischHerkunftDetails => "Erfordert detaillierte Herkunftsangabe für Rindfleisch: Aufzucht- und Schlachtungsort (Schweizer Vorschrift)",
            RuleDef::AP7_5_FischFangort => "Erfordert Fangort für Fisch-Zutaten (Schweizer Vorschrift)",
            RuleDef::Bio_Knospe_AlleZutatenHerkunft => "Erfordert Herkunftsangabe für alle Zutaten (Bio/Knospe Anforderung)",
            RuleDef::Knospe_100_Percent_CH_NoOrigin => "Knospe: Bei 100% landwirtschaftlichen Zutaten aus CH keine Herkunftsangabe nötig",
            RuleDef::Knospe_90_99_Percent_CH_ShowOrigin => "Knospe: Bei 90-99.99% landwirtschaftlichen Zutaten aus CH Herkunftsangabe für CH-Zutaten",
            RuleDef::Knospe_Under90_Percent_CH_IngredientRules => "Knospe: Bei <90% landwirtschaftlichen Zutaten aus CH Herkunftsangabe nach spezifischen Zutatkriterien",
            RuleDef::Bio_Knospe_EingabeIstBio => "Ermöglicht die Eingabe ob eine Zutat bio-zertifiziert ist",
            RuleDef::Knospe_ShowBioSuisseLogo => "Zeigt Bio Suisse Logo basierend auf Schweizer Zutaten-Prozentsatz",
            RuleDef::Bio_Knospe_ZertifizierungsstellePflicht => "Erfordert die Angabe der Bio-Zertifizierungsstelle für Bio und Knospe Produkte",
        }
    }
}

use crate::shared::Configuration;
use std::collections::HashMap;

/// Registry for organizing and managing rules by configuration
///
/// The RuleRegistry provides a centralized way to manage which rules
/// apply to different label configurations (Swiss, Bio, etc.) and
/// helps ensure rule dependencies are satisfied.
pub struct RuleRegistry {
    rules_by_config: HashMap<Configuration, Vec<RuleDef>>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        let mut registry = RuleRegistry {
            rules_by_config: HashMap::new(),
        };
        registry.init_configurations();
        registry
    }

    fn init_configurations(&mut self) {
        // Define rule sets for each configuration
        self.rules_by_config.insert(
            Configuration::Conventional,
            vec![
                RuleDef::AP1_1_ZutatMengeValidierung,
                RuleDef::AP1_2_ProzentOutputNamensgebend,
                RuleDef::AP1_3_EingabeNamensgebendeZutat,
                RuleDef::AP1_4_ManuelleEingabeTotal,
                RuleDef::AP2_1_ZusammegesetztOutput,
                RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
                RuleDef::AP7_3_HerkunftFleischUeber20Prozent,
                RuleDef::AP7_4_RindfleischHerkunftDetails,
                RuleDef::AP7_5_FischFangort,
            ],
        );

        self.rules_by_config.insert(
            Configuration::Bio,
            vec![
                RuleDef::AP1_1_ZutatMengeValidierung,
                RuleDef::AP1_2_ProzentOutputNamensgebend,
                RuleDef::AP1_3_EingabeNamensgebendeZutat,
                RuleDef::AP1_4_ManuelleEingabeTotal,
                RuleDef::AP2_1_ZusammegesetztOutput,
                RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
                RuleDef::AP7_3_HerkunftFleischUeber20Prozent,
                RuleDef::AP7_4_RindfleischHerkunftDetails,
                RuleDef::AP7_5_FischFangort,
                RuleDef::Bio_Knospe_AlleZutatenHerkunft,
                RuleDef::Bio_Knospe_EingabeIstBio,
                RuleDef::Bio_Knospe_ZertifizierungsstellePflicht,
            ],
        );

        self.rules_by_config.insert(
            Configuration::Knospe,
            vec![
                RuleDef::AP1_1_ZutatMengeValidierung,
                RuleDef::AP1_2_ProzentOutputNamensgebend,
                RuleDef::AP1_3_EingabeNamensgebendeZutat,
                RuleDef::AP1_4_ManuelleEingabeTotal,
                RuleDef::AP2_1_ZusammegesetztOutput,
                RuleDef::AP7_1_HerkunftBenoetigtUeber50Prozent,
                RuleDef::AP7_3_HerkunftFleischUeber20Prozent,
                RuleDef::AP7_4_RindfleischHerkunftDetails,
                RuleDef::AP7_5_FischFangort,
                RuleDef::Bio_Knospe_AlleZutatenHerkunft,
                RuleDef::Knospe_100_Percent_CH_NoOrigin,
                RuleDef::Knospe_90_99_Percent_CH_ShowOrigin,
                RuleDef::Knospe_Under90_Percent_CH_IngredientRules,
                RuleDef::Bio_Knospe_EingabeIstBio,
                RuleDef::Knospe_ShowBioSuisseLogo,
                RuleDef::Bio_Knospe_ZertifizierungsstellePflicht,
            ],
        );
    }

    pub fn get_rules_for_config(&self, config: &Configuration) -> Option<&Vec<RuleDef>> {
        self.rules_by_config.get(config)
    }

    pub fn get_rules_by_type(&self, config: &Configuration, rule_type: RuleType) -> Vec<RuleDef> {
        self.get_rules_for_config(config)
            .map(|rules| {
                rules
                    .iter()
                    .filter(|rule| rule.get_type() == rule_type)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_rule_description(&self, rule: &RuleDef) -> &'static str {
        rule.get_description()
    }

    pub fn get_rule_specs_url(&self, rule: &RuleDef) -> Option<&'static str> {
        rule.get_specs_url()
    }

    pub fn validate_dependencies(&self, rules: &[RuleDef]) -> Result<(), String> {
        for rule in rules {
            let deps = rule.deps();
            for dep in deps {
                if !rules.contains(&dep) {
                    return Err(format!(
                        "Rule {:?} depends on {:?}, but dependency is not included",
                        rule, dep
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
