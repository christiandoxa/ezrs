use std::{fs, path::Path};

pub fn run_command(name: &str) -> Result<(), String> {
    run_command_in(Path::new("."), name)
}

fn run_command_in(root: &Path, name: &str) -> Result<(), String> {
    validate_module_name(name)?;

    let commands_dir = root.join("src/commands");
    if !commands_dir.exists() {
        return Err("src/commands does not exist; run this inside an ezrs app".into());
    }

    let module_path = commands_dir.join(format!("{name}.rs"));
    if module_path.exists() {
        return Err(format!("command '{name}' already exists"));
    }

    fs::write(&module_path, command_rs(name)).map_err(|err| err.to_string())?;
    update_mod_rs(root, name)?;

    println!("added command '{name}'");
    Ok(())
}

fn validate_module_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err("command name must be snake_case ASCII".into());
    }
    Ok(())
}

fn command_rs(name: &str) -> String {
    format!(
        r#"use ezrs::{{Context, Result}};

pub async fn run(ctx: Context) -> Result<()> {{
    ctx.println("{name} command");
    Ok(())
}}
"#
    )
}

fn update_mod_rs(root: &Path, name: &str) -> Result<(), String> {
    let path = root.join("src/commands/mod.rs");
    let mut text = if path.exists() {
        fs::read_to_string(&path).map_err(|err| err.to_string())?
    } else {
        String::new()
    };
    let line = format!("pub mod {name};");
    if !text.lines().any(|existing| existing.trim() == line) {
        if !text.ends_with('\n') && !text.is_empty() {
            text.push('\n');
        }
        text.push_str(&line);
        text.push('\n');
    }
    fs::write(path, text).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_command_writes_module_and_mod_rs() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::create_dir_all(dir.path().join("src/commands")).expect("mkdir");
        fs::write(dir.path().join("src/commands/mod.rs"), "").expect("mod");

        run_command_in(dir.path(), "scan").expect("add command");

        assert!(dir.path().join("src/commands/scan.rs").exists());
        let mod_rs = fs::read_to_string(dir.path().join("src/commands/mod.rs")).expect("read mod");
        assert!(mod_rs.contains("pub mod scan;"));
    }
}
