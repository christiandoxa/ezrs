use std::process::Command;

pub fn run(args: &[String]) -> Result<(), String> {
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .args(args)
        .status()
        .map_err(|err| err.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo run failed with status {status}"))
    }
}
