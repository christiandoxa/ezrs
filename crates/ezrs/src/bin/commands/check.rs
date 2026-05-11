use std::process::Command;

pub fn run() -> Result<(), String> {
    for command in commands() {
        let status = Command::new("cargo")
            .args(&command)
            .status()
            .map_err(|err| err.to_string())?;
        if !status.success() {
            return Err(format!(
                "cargo {} failed with status {status}",
                command.join(" ")
            ));
        }
    }
    Ok(())
}

pub fn commands() -> Vec<Vec<&'static str>> {
    vec![vec!["fmt"], vec!["check"], vec!["test"]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_command_construction() {
        assert_eq!(commands(), vec![vec!["fmt"], vec!["check"], vec!["test"]]);
    }
}
