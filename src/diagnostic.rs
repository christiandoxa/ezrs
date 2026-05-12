//! Small diagnostic checks for doctor-style commands and config validation.

/// Result state for one diagnostic check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckStatus {
    /// The check passed.
    Pass,
    /// The check found a non-fatal issue.
    Warn,
    /// The check failed.
    Fail,
    /// The check was intentionally skipped.
    Skip,
}

impl CheckStatus {
    /// Returns the stable uppercase label used in plain CLI reports.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
            Self::Skip => "SKIP",
        }
    }

    /// Returns true when this status should make a doctor command fail.
    pub const fn is_failure(self) -> bool {
        matches!(self, Self::Fail)
    }
}

/// One completed diagnostic check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Check {
    name: String,
    status: CheckStatus,
    message: String,
}

impl Check {
    /// Creates a check result.
    pub fn new(name: impl Into<String>, status: CheckStatus, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status,
            message: message.into(),
        }
    }

    /// Creates a passing check.
    pub fn pass(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(name, CheckStatus::Pass, message)
    }

    /// Creates a warning check.
    pub fn warn(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(name, CheckStatus::Warn, message)
    }

    /// Creates a failing check.
    pub fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(name, CheckStatus::Fail, message)
    }

    /// Creates a skipped check.
    pub fn skip(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(name, CheckStatus::Skip, message)
    }

    /// Returns the check name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the check status.
    pub const fn status(&self) -> CheckStatus {
        self.status
    }

    /// Returns the user-facing check message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Completed diagnostic output.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiagnosticReport {
    title: String,
    checks: Vec<Check>,
}

impl DiagnosticReport {
    /// Creates an empty diagnostic report.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            checks: Vec::new(),
        }
    }

    /// Adds a completed check.
    pub fn check(mut self, check: Check) -> Self {
        self.checks.push(check);
        self
    }

    /// Returns the report title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns all checks.
    pub fn checks(&self) -> &[Check] {
        &self.checks
    }

    /// Returns true when any check failed.
    pub fn has_failures(&self) -> bool {
        self.checks.iter().any(|check| check.status.is_failure())
    }

    /// Renders a shell-friendly plain text report.
    pub fn render_text(&self) -> String {
        let mut out = String::new();

        if !self.title.is_empty() {
            out.push_str(&self.title);
            out.push('\n');
        }

        for check in &self.checks {
            out.push_str(check.status.as_str());
            out.push_str("  ");
            out.push_str(&check.name);

            if !check.message.is_empty() {
                out.push_str(" - ");
                out.push_str(&check.message);
            }

            out.push('\n');
        }

        out.trim_end().to_string()
    }
}

/// Sync diagnostic runner for doctor commands and table-driven tests.
#[derive(Default)]
pub struct DiagnosticRunner {
    title: String,
    checks: Vec<Box<dyn Fn() -> Check + Send + Sync>>,
}

impl DiagnosticRunner {
    /// Creates an empty runner.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            checks: Vec::new(),
        }
    }

    /// Adds a sync check closure.
    pub fn check(mut self, check: impl Fn() -> Check + Send + Sync + 'static) -> Self {
        self.checks.push(Box::new(check));
        self
    }

    /// Runs all checks and returns a report.
    pub fn run(&self) -> DiagnosticReport {
        let mut report = DiagnosticReport::new(self.title.clone());

        for check in &self.checks {
            report = report.check(check());
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_tracks_failures_and_renders_checks() {
        let report = DiagnosticReport::new("doctor")
            .check(Check::pass("config", "loaded"))
            .check(Check::warn("cache", "missing"))
            .check(Check::fail("token", "not configured"));

        assert!(report.has_failures());
        assert_eq!(
            report.render_text(),
            "doctor\nPASS  config - loaded\nWARN  cache - missing\nFAIL  token - not configured"
        );
    }

    #[test]
    fn runner_executes_sync_checks() {
        let report = DiagnosticRunner::new("config")
            .check(|| Check::pass("file", "found"))
            .check(|| Check::skip("remote", "disabled"))
            .run();

        assert_eq!(report.checks().len(), 2);
        assert_eq!(report.checks()[1].status(), CheckStatus::Skip);
        assert!(!report.has_failures());
    }
}
