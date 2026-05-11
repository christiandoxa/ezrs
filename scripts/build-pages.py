#!/usr/bin/env python3
"""Build the ezrs GitHub Pages site.

The script intentionally uses only the Python standard library so GitHub Pages
generation stays boring and reproducible.
"""

from __future__ import annotations

import html
import os
import re
import shutil
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "target" / "pages"
API_DOCS = ROOT / "target" / "doc"


def slugify(value: str) -> str:
    value = value.lower()
    value = re.sub(r"`([^`]*)`", r"\1", value)
    value = re.sub(r"[^a-z0-9]+", "-", value)
    return value.strip("-") or "section"


def inline_markdown(value: str) -> str:
    value = html.escape(value)
    value = re.sub(r"`([^`]*)`", r"<code>\1</code>", value)
    value = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", value)
    value = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", r'<a href="\2">\1</a>', value)
    return value


def render_table(lines: list[str]) -> str:
    rows = []
    for line in lines:
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        rows.append(cells)

    if len(rows) < 2:
        return "".join(f"<p>{inline_markdown(line)}</p>" for line in lines)

    headers = rows[0]
    body = rows[2:]
    out = ["<div class=\"table-wrap\"><table>", "<thead><tr>"]
    out.extend(f"<th>{inline_markdown(cell)}</th>" for cell in headers)
    out.append("</tr></thead><tbody>")
    for row in body:
        out.append("<tr>")
        out.extend(f"<td>{inline_markdown(cell)}</td>" for cell in row)
        out.append("</tr>")
    out.append("</tbody></table></div>")
    return "\n".join(out)


def markdown_to_html(text: str) -> str:
    out: list[str] = []
    paragraph: list[str] = []
    table: list[str] = []
    in_code = False
    code_lang = ""
    code_lines: list[str] = []
    in_list = False

    def flush_paragraph() -> None:
        nonlocal paragraph
        if paragraph:
            out.append(f"<p>{inline_markdown(' '.join(paragraph))}</p>")
            paragraph = []

    def flush_table() -> None:
        nonlocal table
        if table:
            out.append(render_table(table))
            table = []

    def close_list() -> None:
        nonlocal in_list
        if in_list:
            out.append("</ul>")
            in_list = False

    for raw in text.splitlines():
        line = raw.rstrip()

        if line.startswith("```"):
            flush_paragraph()
            flush_table()
            close_list()
            if in_code:
                escaped = html.escape("\n".join(code_lines))
                lang_class = f" language-{html.escape(code_lang)}" if code_lang else ""
                out.append(f"<pre><code class=\"{lang_class}\">{escaped}</code></pre>")
                code_lines = []
                code_lang = ""
                in_code = False
            else:
                code_lang = line.strip("`").strip()
                in_code = True
            continue

        if in_code:
            code_lines.append(line)
            continue

        if not line:
            flush_paragraph()
            flush_table()
            close_list()
            continue

        if line.startswith("|") and line.endswith("|"):
            flush_paragraph()
            close_list()
            table.append(line)
            continue
        flush_table()

        heading = re.match(r"^(#{1,6})\s+(.+)$", line)
        if heading:
            flush_paragraph()
            close_list()
            level = len(heading.group(1))
            title = heading.group(2).strip()
            out.append(
                f'<h{level} id="{slugify(title)}">{inline_markdown(title)}</h{level}>'
            )
            continue

        bullet = re.match(r"^-\s+(.+)$", line)
        if bullet:
            flush_paragraph()
            if not in_list:
                out.append("<ul>")
                in_list = True
            out.append(f"<li>{inline_markdown(bullet.group(1))}</li>")
            continue

        close_list()
        paragraph.append(line.strip())

    flush_paragraph()
    flush_table()
    close_list()
    return "\n".join(out)


def page(title: str, body: str, active: str = "") -> str:
    nav = [
        ("Home", "index.html"),
        ("README", "readme.html"),
        ("Go Patterns", "golang-patterns.html"),
        ("Go Tour", "go-tour.html"),
        ("Process", "process-management.html"),
        ("Examples", "examples.html"),
        ("API", "api/ezrs/index.html"),
    ]
    nav_html = "\n".join(
        f'<a class="{"active" if label == active else ""}" href="{href}">{label}</a>'
        for label, href in nav
    )
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{html.escape(title)} | ezrs</title>
  <link rel="stylesheet" href="style.css">
</head>
<body>
  <header class="site-header">
    <a class="brand" href="index.html">ezrs</a>
    <nav>{nav_html}</nav>
  </header>
  <main>
{body}
  </main>
</body>
</html>
"""


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def render_markdown_file(source: Path, output: Path, title: str, active: str) -> None:
    body = f'<article class="doc">\n{markdown_to_html(source.read_text(encoding="utf-8"))}\n</article>'
    write(output, page(title, body, active))


def example_group(path: Path) -> str:
    parts = path.parts
    if "components" in parts:
        return "Component Examples"
    if "golang_patterns" in parts:
        return "Go Application Pattern Examples"
    if "go_tour" in parts:
        return "Go Tour Mapping Examples"
    return "Examples"


def build_examples() -> None:
    examples = sorted((ROOT / "examples").glob("**/*"))
    rust_examples = [path for path in examples if path.suffix == ".rs"]
    markdown_examples = [path for path in examples if path.suffix == ".md"]
    grouped: dict[str, list[Path]] = {}
    for path in rust_examples + markdown_examples:
        grouped.setdefault(example_group(path), []).append(path)

    sections = ["<section class=\"hero compact\"><h1>Examples</h1><p>Compiling examples for ezrs components, Go application patterns, and Go Tour mappings.</p></section>"]
    for group, paths in grouped.items():
        sections.append(f"<section class=\"cards\"><h2>{html.escape(group)}</h2><ul class=\"example-list\">")
        for path in paths:
            rel = path.relative_to(ROOT)
            target = Path("examples") / rel.with_suffix(".html").relative_to("examples")
            sections.append(
                f'<li><a href="{target.as_posix()}">{html.escape(rel.as_posix())}</a></li>'
            )
            code = path.read_text(encoding="utf-8")
            escaped = html.escape(code)
            body = (
                f'<article class="doc"><p><a href="../../examples.html">Back to examples</a></p>'
                f"<h1>{html.escape(rel.as_posix())}</h1>"
                f"<pre><code>{escaped}</code></pre></article>"
            )
            write(OUT / target, page(rel.as_posix(), body, "Examples"))
        sections.append("</ul></section>")

    write(OUT / "examples.html", page("Examples", "\n".join(sections), "Examples"))


def copy_api_docs() -> None:
    if API_DOCS.exists():
        target = OUT / "api"
        if target.exists():
            shutil.rmtree(target)
        shutil.copytree(API_DOCS, target, ignore=shutil.ignore_patterns(".lock"))


def build_site() -> None:
    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir(parents=True)

    index = """
<section class="hero">
  <p class="eyebrow">Go-style application patterns, Rust-grade safety.</p>
  <h1>Build Go-like app architecture in idiomatic Rust.</h1>
  <p>ezrs teaches and implements familiar Go application patterns for Rust CLI tools, workers, file processors, automation tools, and small daemons.</p>
  <div class="actions">
    <a class="button primary" href="readme.html">Read Guide</a>
    <a class="button" href="api/ezrs/index.html">API Docs</a>
    <a class="button" href="go-tour.html">Go Tour Mapping</a>
  </div>
</section>
<section class="grid">
  <article><h2>Pattern First</h2><p>Map <code>run() error</code>, <code>context.Context</code>, explicit dependencies, goroutines, channels, cancellation, and table-driven tests to Rust.</p></article>
  <article><h2>Single Crate</h2><p>Depend on <code>ezrs</code> for both the app framework and the <code>ezrs</code> CLI binary.</p></article>
  <article><h2>Generated Docs</h2><p>This site is deployed by GitHub Actions and includes Rust API docs built with <code>cargo doc</code>.</p></article>
</section>
<section class="doc">
<h2>Quickstart</h2>
<pre><code>use ezrs::prelude::*;

#[ezrs::main]
async fn main() -> Result&lt;()&gt; {
    App::new()
        .name("demo")
        .command(hello)
        .run()
        .await
}

async fn hello(ctx: Context) -> Result&lt;()&gt; {
    ctx.println("hello from ezrs");
    Ok(())
}</code></pre>
</section>
"""
    write(OUT / "index.html", page("Documentation", index, "Home"))
    render_markdown_file(ROOT / "README.md", OUT / "readme.html", "README", "README")
    render_markdown_file(
        ROOT / "docs" / "golang-patterns.md",
        OUT / "golang-patterns.html",
        "Go Application Patterns",
        "Go Patterns",
    )
    render_markdown_file(
        ROOT / "docs" / "go-tour-mapping.md",
        OUT / "go-tour.html",
        "Go Tour Mapping",
        "Go Tour",
    )
    render_markdown_file(
        ROOT / "docs" / "process-management.md",
        OUT / "process-management.html",
        "Process Management",
        "Process",
    )
    build_examples()
    copy_api_docs()
    write(OUT / ".nojekyll", "")
    write(OUT / "style.css", STYLE)


STYLE = """
:root {
  --bg: #f6f1e8;
  --ink: #1d2524;
  --muted: #5e6b66;
  --card: #fffaf0;
  --line: #d8cdbb;
  --accent: #0e6b57;
  --accent-2: #d36b32;
}

* { box-sizing: border-box; }
body {
  margin: 0;
  font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  color: var(--ink);
  background:
    radial-gradient(circle at top left, #ffe2ba 0, transparent 30rem),
    linear-gradient(135deg, var(--bg), #eef3ec);
  line-height: 1.6;
}
a { color: var(--accent); }
.site-header {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem min(5vw, 4rem);
  border-bottom: 1px solid var(--line);
  background: rgba(246, 241, 232, 0.9);
  backdrop-filter: blur(10px);
}
.brand {
  color: var(--ink);
  font-size: 1.5rem;
  font-weight: 800;
  text-decoration: none;
}
nav {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}
nav a {
  color: var(--ink);
  padding: 0.45rem 0.7rem;
  border-radius: 999px;
  text-decoration: none;
}
nav a.active,
nav a:hover {
  background: var(--ink);
  color: var(--card);
}
main {
  width: min(1120px, 92vw);
  margin: 0 auto;
  padding: 3rem 0 5rem;
}
.hero {
  padding: clamp(2rem, 8vw, 6rem);
  border: 1px solid var(--line);
  border-radius: 2rem;
  background: linear-gradient(135deg, #fffaf0, #e4efe8);
  box-shadow: 0 24px 80px rgba(29, 37, 36, 0.12);
}
.hero.compact { padding: 2rem; }
.eyebrow {
  color: var(--accent-2);
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
h1 {
  margin: 0.4rem 0 1rem;
  font-size: clamp(2.2rem, 7vw, 5.5rem);
  line-height: 0.95;
}
h2 { margin-top: 2rem; }
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  margin-top: 1.5rem;
}
.button {
  display: inline-flex;
  padding: 0.85rem 1rem;
  border: 1px solid var(--ink);
  border-radius: 999px;
  color: var(--ink);
  text-decoration: none;
  font-weight: 700;
}
.button.primary {
  background: var(--ink);
  color: var(--card);
}
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 1rem;
  margin: 1rem 0;
}
.grid article,
.cards,
.doc {
  padding: 1.25rem;
  border: 1px solid var(--line);
  border-radius: 1.2rem;
  background: rgba(255, 250, 240, 0.86);
}
.doc {
  margin-top: 1rem;
}
code {
  padding: 0.1rem 0.25rem;
  border-radius: 0.3rem;
  background: #efe5d4;
}
pre {
  overflow-x: auto;
  padding: 1rem;
  border-radius: 1rem;
  background: #17211f;
  color: #f8f2e8;
}
pre code {
  padding: 0;
  background: transparent;
  color: inherit;
}
.table-wrap {
  overflow-x: auto;
}
table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.95rem;
}
th, td {
  padding: 0.65rem;
  border: 1px solid var(--line);
  vertical-align: top;
}
th {
  background: #efe5d4;
  text-align: left;
}
.example-list {
  columns: 2 20rem;
}
@media (max-width: 720px) {
  .site-header { align-items: flex-start; flex-direction: column; }
  h1 { font-size: 2.4rem; }
}
"""


if __name__ == "__main__":
    os.chdir(ROOT)
    build_site()
    print(f"built {OUT}")
