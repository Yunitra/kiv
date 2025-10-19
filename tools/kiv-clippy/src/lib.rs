//! Kiv linting library.

mod rules;

pub use rules::LintRule;

/// Returns all available lint rules
pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(rules::UnusedVariable),
        Box::new(rules::UnreachableCode),
        Box::new(rules::RedundantElse),
        Box::new(rules::ConstMutable),
    ]
}
