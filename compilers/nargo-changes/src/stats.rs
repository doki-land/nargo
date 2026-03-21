#![warn(missing_docs)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{change_set::ChangeSet, types::ChangeType};

/// Change statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeStats {
    /// Total number of change sets.
    pub total_change_sets: usize,
    /// Change sets grouped by type.
    pub change_sets_by_type: HashMap<ChangeType, usize>,
    /// Change sets grouped by package.
    pub change_sets_by_package: HashMap<String, usize>,
    /// Total number of breaking changes.
    pub breaking_changes: usize,
    /// Total number of features.
    pub features: usize,
    /// Total number of bug fixes.
    pub bug_fixes: usize,
    /// Total number of other changes.
    pub other_changes: usize,
    /// Average changes per package.
    pub avg_changes_per_package: f64,
    /// Most common change type.
    pub most_common_change_type: Option<ChangeType>,
    /// Most affected package.
    pub most_affected_package: Option<String>,
}

/// Change trend data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTrendPoint {
    /// The timestamp for this data point.
    pub timestamp: DateTime<Utc>,
    /// Number of change sets in this period.
    pub change_set_count: usize,
    /// Change sets by type in this period.
    pub change_sets_by_type: HashMap<ChangeType, usize>,
    /// Number of packages affected in this period.
    pub packages_affected: usize,
}

/// Change trend analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTrend {
    /// The time period for the trend analysis.
    pub time_period: String,
    /// The trend data points.
    pub data_points: Vec<ChangeTrendPoint>,
    /// Overall statistics for the trend period.
    pub overall_stats: ChangeStats,
    /// Change rate over time (changes per day).
    pub change_rate: f64,
    /// Trend direction (positive, negative, or stable).
    pub trend_direction: String,
    /// Most active period.
    pub most_active_period: Option<DateTime<Utc>>,
}

impl ChangeStats {
    /// Creates a new ChangeStats instance from a list of change sets.
    pub fn from_change_sets(change_sets: &[ChangeSet]) -> Self {
        let total_change_sets = change_sets.len();
        let mut change_sets_by_type = HashMap::new();
        let mut change_sets_by_package = HashMap::new();
        let mut breaking_changes = 0;
        let mut features = 0;
        let mut bug_fixes = 0;
        let mut other_changes = 0;

        // Count changes by type and package
        for change_set in change_sets {
            // Count by type
            *change_sets_by_type.entry(change_set.r#type.clone()).or_insert(0) += 1;

            // Count by package
            for package in &change_set.packages {
                *change_sets_by_package.entry(package.clone()).or_insert(0) += 1;
            }

            // Count specific types
            match change_set.r#type {
                ChangeType::Breaking => breaking_changes += 1,
                ChangeType::Feature => features += 1,
                ChangeType::Fix => bug_fixes += 1,
                _ => other_changes += 1,
            }
        }

        // Calculate average changes per package
        let avg_changes_per_package = if change_sets_by_package.is_empty() { 0.0 } else { change_sets.iter().map(|cs| cs.packages.len()).sum::<usize>() as f64 / change_sets_by_package.len() as f64 };

        // Find most common change type
        let most_common_change_type = change_sets_by_type.iter().max_by(|a, b| a.1.cmp(b.1)).map(|(ctype, _)| ctype.clone());

        // Find most affected package
        let most_affected_package = change_sets_by_package.iter().max_by(|a, b| a.1.cmp(b.1)).map(|(pkg, _)| pkg.clone());

        Self { total_change_sets, change_sets_by_type, change_sets_by_package, breaking_changes, features, bug_fixes, other_changes, avg_changes_per_package, most_common_change_type, most_affected_package }
    }

    /// Generates a summary of the change statistics.
    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&format!("Change Statistics Summary:\n"));
        summary.push_str(&format!("- Total Change Sets: {}\n", self.total_change_sets));
        summary.push_str(&format!("- Breaking Changes: {}\n", self.breaking_changes));
        summary.push_str(&format!("- Features: {}\n", self.features));
        summary.push_str(&format!("- Bug Fixes: {}\n", self.bug_fixes));
        summary.push_str(&format!("- Other Changes: {}\n", self.other_changes));
        summary.push_str(&format!("- Average Changes per Package: {:.2}\n", self.avg_changes_per_package));

        if let Some(ctype) = &self.most_common_change_type {
            summary.push_str(&format!("- Most Common Change Type: {}\n", ctype.as_str()));
        }

        if let Some(pkg) = &self.most_affected_package {
            summary.push_str(&format!("- Most Affected Package: {}\n", pkg));
        }

        summary.push_str("\nChanges by Type:\n");
        for (ctype, count) in &self.change_sets_by_type {
            summary.push_str(&format!("- {}: {}\n", ctype.as_str(), count));
        }

        summary.push_str("\nChanges by Package:\n");
        for (pkg, count) in &self.change_sets_by_package {
            summary.push_str(&format!("- {}: {}\n", pkg, count));
        }

        summary
    }
}

impl ChangeTrend {
    /// Creates a new ChangeTrend instance from a list of change sets and a time period.
    pub fn from_change_sets(change_sets: &[ChangeSet], time_period: &str) -> Self {
        // For simplicity, we'll create a basic trend analysis
        // In a real implementation, you would group changes by time periods
        let stats = ChangeStats::from_change_sets(change_sets);

        // Create a single data point for now
        let data_point = ChangeTrendPoint { timestamp: chrono::Utc::now(), change_set_count: change_sets.len(), change_sets_by_type: stats.change_sets_by_type.clone(), packages_affected: stats.change_sets_by_package.len() };

        // Calculate change rate (changes per day)
        // For simplicity, we'll assume a 30-day period
        let change_rate = change_sets.len() as f64 / 30.0;

        // Determine trend direction
        let trend_direction = if change_rate > 1.0 {
            "positive"
        }
        else if change_rate < 0.5 {
            "negative"
        }
        else {
            "stable"
        };

        Self { time_period: time_period.to_string(), data_points: vec![data_point], overall_stats: stats, change_rate, trend_direction: trend_direction.to_string(), most_active_period: Some(chrono::Utc::now()) }
    }

    /// Generates a summary of the change trend.
    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&format!("Change Trend Analysis ({}):\n", self.time_period));
        summary.push_str(&format!("- Change Rate: {:.2} changes per day\n", self.change_rate));
        summary.push_str(&format!("- Trend Direction: {}\n", self.trend_direction));

        if let Some(period) = &self.most_active_period {
            summary.push_str(&format!("- Most Active Period: {}\n", period.format("%Y-%m-%d %H:%M:%S")));
        }

        summary.push_str("\nOverall Statistics:\n");
        summary.push_str(&self.overall_stats.generate_summary());

        summary
    }
}
