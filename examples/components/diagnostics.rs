//! Component example: doctor-style diagnostics with sync checks.

use ezrs::{Check, DiagnosticRunner};

fn main() {
    let report = DiagnosticRunner::new("doctor")
        .check(|| Check::pass("config", "loaded"))
        .check(|| Check::warn("cache", "directory will be created"))
        .run();

    println!("{}", report.render_text());

    if report.has_failures() {
        std::process::exit(1);
    }
}
