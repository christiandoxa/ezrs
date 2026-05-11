//! Plain text report helpers for CLI output.

use serde_json::{Value, json};

/// Plain text table builder.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    /// Creates a table with headers.
    pub fn new(headers: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            headers: headers.into_iter().map(Into::into).collect(),
            rows: Vec::new(),
        }
    }

    /// Adds one row.
    pub fn row(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.rows.push(values.into_iter().map(Into::into).collect());
        self
    }

    /// Returns table headers.
    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    /// Returns table rows.
    pub fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }

    /// Renders a padded plain text table.
    pub fn render_text(&self) -> String {
        let column_count = self
            .headers
            .len()
            .max(self.rows.iter().map(Vec::len).max().unwrap_or(0));

        if column_count == 0 {
            return String::new();
        }

        let mut widths = vec![0; column_count];
        for (index, header) in self.headers.iter().enumerate() {
            widths[index] = widths[index].max(header.len());
        }
        for row in &self.rows {
            for (index, value) in row.iter().enumerate() {
                widths[index] = widths[index].max(value.len());
            }
        }

        let mut lines = Vec::new();
        if !self.headers.is_empty() {
            lines.push(render_row(&self.headers, &widths));
            lines.push(
                widths
                    .iter()
                    .map(|width| "-".repeat(*width))
                    .collect::<Vec<_>>()
                    .join("  "),
            );
        }

        for row in &self.rows {
            lines.push(render_row(row, &widths));
        }

        lines.join("\n")
    }

    /// Renders table data as JSON.
    pub fn render_json(&self) -> Value {
        let rows = self
            .rows
            .iter()
            .map(|row| {
                let mut object = serde_json::Map::new();
                for (index, value) in row.iter().enumerate() {
                    let key = self
                        .headers
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| format!("column_{index}"));
                    object.insert(key, json!(value));
                }
                Value::Object(object)
            })
            .collect::<Vec<_>>();

        json!(rows)
    }
}

fn render_row(values: &[String], widths: &[usize]) -> String {
    (0..widths.len())
        .map(|index| {
            let value = values.get(index).map(String::as_str).unwrap_or("");
            format!("{value:<width$}", width = widths[index])
        })
        .collect::<Vec<_>>()
        .join("  ")
        .trim_end()
        .to_string()
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ReportPart {
    Line(String),
    Field(String, String),
    Table(Table),
}

/// Plain text report builder for command output.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Report {
    title: String,
    parts: Vec<ReportPart>,
}

impl Report {
    /// Creates an empty report.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            parts: Vec::new(),
        }
    }

    /// Adds a free-form line.
    pub fn line(mut self, value: impl Into<String>) -> Self {
        self.parts.push(ReportPart::Line(value.into()));
        self
    }

    /// Adds a key/value field.
    pub fn field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parts.push(ReportPart::Field(key.into(), value.into()));
        self
    }

    /// Adds a table.
    pub fn table(mut self, table: Table) -> Self {
        self.parts.push(ReportPart::Table(table));
        self
    }

    /// Renders a plain text report.
    pub fn render_text(&self) -> String {
        let mut blocks = Vec::new();

        if !self.title.is_empty() {
            blocks.push(self.title.clone());
        }

        for part in &self.parts {
            match part {
                ReportPart::Line(line) => blocks.push(line.clone()),
                ReportPart::Field(key, value) => blocks.push(format!("{key}: {value}")),
                ReportPart::Table(table) => blocks.push(table.render_text()),
            }
        }

        blocks
            .into_iter()
            .filter(|block| !block.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Renders report data as simple JSON blocks.
    pub fn render_json(&self) -> Value {
        let parts = self
            .parts
            .iter()
            .map(|part| match part {
                ReportPart::Line(line) => json!({ "type": "line", "value": line }),
                ReportPart::Field(key, value) => {
                    json!({ "type": "field", "key": key, "value": value })
                }
                ReportPart::Table(table) => {
                    json!({ "type": "table", "headers": table.headers(), "rows": table.rows() })
                }
            })
            .collect::<Vec<_>>();

        json!({
            "title": self.title,
            "parts": parts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_renders_padded_columns() {
        let table = Table::new(["name", "status"])
            .row(["config", "ok"])
            .row(["database", "missing"]);

        assert_eq!(
            table.render_text(),
            "name      status\n--------  -------\nconfig    ok\ndatabase  missing"
        );
    }

    #[test]
    fn report_renders_plain_text_blocks() {
        let report = Report::new("run")
            .field("env", "dev")
            .table(Table::new(["job", "result"]).row(["sync", "pass"]));

        assert_eq!(
            report.render_text(),
            "run\n\nenv: dev\n\njob   result\n----  ------\nsync  pass"
        );
    }

    #[test]
    fn report_renders_json() {
        let json = Report::new("summary").line("ok").render_json();

        assert_eq!(json["title"], "summary");
        assert_eq!(json["parts"][0]["type"], "line");
        assert_eq!(json["parts"][0]["value"], "ok");
    }
}
