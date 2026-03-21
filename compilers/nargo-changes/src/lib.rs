//! Change set management for HXO framework.
//!
//! This crate provides tools for managing version changes, detecting file changes,
//! generating changelogs, and integrating with version control systems.

#![warn(missing_docs)]

pub mod change_set;
pub mod file_change;
pub mod preview;
pub mod stats;
pub mod types;
pub mod vcs;

use crate::{
    change_set::ChangeSetManager,
    file_change::FileChangeDetector,
    preview::ChangePreview,
    stats::{ChangeStats, ChangeTrend},
    vcs::VcsIntegration,
};

/// Gets change statistics for all change sets.
pub fn get_change_stats(manager: &ChangeSetManager) -> nargo_types::Result<ChangeStats> {
    let change_sets = manager.read_change_sets()?;
    Ok(ChangeStats::from_change_sets(&change_sets))
}

/// Analyzes change trends over a specified time period.
pub fn analyze_change_trend(manager: &ChangeSetManager, time_period: &str) -> nargo_types::Result<ChangeTrend> {
    let change_sets = manager.read_change_sets()?;
    Ok(ChangeTrend::from_change_sets(&change_sets, time_period))
}

/// Generates a comprehensive change report including statistics and trends.
pub fn generate_change_report(manager: &ChangeSetManager, time_period: &str) -> nargo_types::Result<String> {
    let stats = get_change_stats(manager)?;
    let trend = analyze_change_trend(manager, time_period)?;

    let mut report = String::new();
    report.push_str("# Change Report\n\n");
    report.push_str("## Statistics\n\n");
    report.push_str(&stats.generate_summary());
    report.push_str("\n## Trend Analysis\n\n");
    report.push_str(&trend.generate_summary());

    Ok(report)
}
