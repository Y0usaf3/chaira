use crate::kinds::*;
use crate::prelude::*;

// in progress ig lol

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MigrationStrategy {
    Safe,
    Risky,
    Destructive,
    NoOp, // nop, no operation,
}

impl FieldConfig {
    pub fn get_migration_strategy(&self, new_config: &FieldConfig) -> MigrationStrategy {
        match (self, new_config) {
            // 1. EXACT MATCH
            (old, new) if old == new => MigrationStrategy::NoOp,

            // 2. TEXT LOGIC
            (FieldConfig::Text(old), FieldConfig::Text(new)) => match (old, new) {
                // SINGLE LINE => LONG TEXT (safe)
                (TextConfig::SingleLine { .. }, TextConfig::LongText { .. }) => {
                    MigrationStrategy::Safe
                }
                // LONG TEXT => SINGLE LINE (risky: truncation)
                (TextConfig::LongText { .. }, TextConfig::SingleLine { .. }) => {
                    MigrationStrategy::Risky
                }
                // ANY => SPECIALIZED (destructive: format mismatch)
                (_, TextConfig::Email) | (_, TextConfig::URL) | (_, TextConfig::Phone) => {
                    MigrationStrategy::Destructive
                }
                // SPECIALIZED => ANY (safe: strings are strings)
                (TextConfig::Email, _) | (TextConfig::URL, _) | (TextConfig::Phone, _) => {
                    MigrationStrategy::Safe
                }
                _ => MigrationStrategy::Safe,
            },

            // 3. SELECT LOGIC
            (FieldConfig::Select(old), FieldConfig::Select(new)) => {
                let (old_opts, new_opts, transition_type) = match (old, new) {
                    // SINGLE => SINGLE (safe)
                    (SelectConfig::Single { options: o }, SelectConfig::Single { options: n }) => {
                        (o, n, MigrationStrategy::Safe)
                    }
                    // MULTI => MULTI (safe)
                    (SelectConfig::Multi { options: o }, SelectConfig::Multi { options: n }) => {
                        (o, n, MigrationStrategy::Safe)
                    }
                    // SINGLE => MULTI (safe: wrap in list)
                    (SelectConfig::Single { options: o }, SelectConfig::Multi { options: n }) => {
                        (o, n, MigrationStrategy::Safe)
                    }
                    // MULTI => SINGLE (risky: which one to keep?)
                    (SelectConfig::Multi { options: o }, SelectConfig::Single { options: n }) => {
                        (o, n, MigrationStrategy::Risky)
                    }
                };

                let old_ids: std::collections::HashSet<_> = old_opts.iter().map(|o| o.id).collect();
                let new_ids: std::collections::HashSet<_> = new_opts.iter().map(|o| o.id).collect();

                // OPTION REMOVAL (risky: existing cells lose references)
                if !old_ids.is_subset(&new_ids) {
                    MigrationStrategy::Risky
                } else {
                    transition_type
                }
            }

            // 4. NUMBER LOGIC
            (FieldConfig::Number(_old), FieldConfig::Number(_new)) =>
            // will work on this soon tm
            {
                MigrationStrategy::Safe
            }

            // 5. DATETIME LOGIC
            (FieldConfig::Datetime(old), FieldConfig::Datetime(new)) => match (old, new) {
                // DATE => DATE (safe: just a UI format change)
                (DatetimeConfig::Date { .. }, DatetimeConfig::Date { .. }) => {
                    MigrationStrategy::Safe
                }
                // DURATION => DURATION (safe: math conversion)
                (DatetimeConfig::Duration { .. }, DatetimeConfig::Duration { .. }) => {
                    MigrationStrategy::Safe
                }
                // DATE <=> DURATION (destructive: logically incompatible)
                _ => MigrationStrategy::Destructive,
            },

            // 6. RELATION LOGIC
            (FieldConfig::Relation(old), FieldConfig::Relation(new)) => match (old, new) {
                // LINK => LINK (risky: changing One-to-Many to One-to-One might drop records)
                (
                    RelationConfig::Link { r#type: t1, .. },
                    RelationConfig::Link { r#type: t2, .. },
                ) => {
                    if t1 == t2 {
                        MigrationStrategy::Safe
                    } else {
                        MigrationStrategy::Risky
                    }
                }
                // LOOKUP/ROLLUP => ANY (risky: switching from computed relation to static link)
                _ => MigrationStrategy::Risky,
            },

            // 7. GLOBAL FALLBACKS
            // ANY => TEXT (safe: stringification)
            (_, FieldConfig::Text(_)) => MigrationStrategy::Safe,
            // TEXT => NUMBER/DATE (risky: parsing might fail)
            (FieldConfig::Text(_), FieldConfig::Number(_) | FieldConfig::Datetime(_)) => {
                MigrationStrategy::Risky
            }
            // EVERYTHING ELSE (destructive: requires field clear)
            _ => MigrationStrategy::Destructive,
        }
    }
}
