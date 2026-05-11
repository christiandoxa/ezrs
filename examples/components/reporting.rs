//! Component example: plain CLI reports.

use ezrs::{Report, Table};

fn main() {
    let jobs = Table::new(["job", "status"])
        .row(["import", "ok"])
        .row(["cleanup", "skipped"]);

    let output = Report::new("daily run")
        .field("environment", "local")
        .table(jobs)
        .render_text();

    println!("{output}");
}
