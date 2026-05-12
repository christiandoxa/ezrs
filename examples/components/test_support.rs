//! Test support helpers for Go-style table tests and fake dependencies.
//!
#[cfg(test)]
use ezrs::Result;
#[cfg(test)]
use ezrs::test_support::{
    EnvMap, FakeClock, FakeCommandOutput, FakeCommandRequest, FakeProcessRunner, TempWorkspace,
};
#[cfg(test)]
use std::time::{Duration, SystemTime};

#[cfg(test)]
#[derive(Clone)]
struct BuildService {
    runner: FakeProcessRunner,
    env: EnvMap,
}

#[cfg(test)]
impl BuildService {
    fn check(&self, package: &str) -> Result<String> {
        let output = self.runner.run(
            FakeCommandRequest::new("cargo")
                .args(["check", "-p", package])
                .env(
                    "RUST_LOG",
                    self.env
                        .get_string("RUST_LOG")
                        .unwrap_or_else(|| String::from("info")),
                ),
        )?;

        Ok(output.stdout_lossy())
    }
}

#[cfg(test)]
#[test]
fn table_driven_service_tests_with_fake_processes() {
    struct Case {
        name: &'static str,
        package: &'static str,
        output: &'static str,
        want: &'static str,
    }

    let cases = [
        Case {
            name: "root package",
            package: "ezrs",
            output: "checked ezrs\n",
            want: "checked ezrs\n",
        },
        Case {
            name: "tool package",
            package: "ezrs-cli",
            output: "checked ezrs-cli\n",
            want: "checked ezrs-cli\n",
        },
    ];

    for case in cases {
        let runner =
            FakeProcessRunner::new().with_outputs([FakeCommandOutput::success(case.output)]);
        let service = BuildService {
            runner: runner.clone(),
            env: EnvMap::new().set("RUST_LOG", "debug"),
        };

        let got = service.check(case.package).expect(case.name);
        assert_eq!(got, case.want, "{}", case.name);

        let request = runner.last_request().expect("recorded fake request");
        assert_eq!(request.program, "cargo");
        assert_eq!(request.args, ["check", "-p", case.package]);
        assert_eq!(request.env.get_string("RUST_LOG").as_deref(), Some("debug"));
    }
}

#[cfg(test)]
#[test]
fn temp_workspace_is_removed_on_drop() {
    let path;
    {
        let workspace = TempWorkspace::new("component-test").expect("workspace");
        path = workspace.root().to_path_buf();
        workspace.write("input.txt", "hello").expect("write input");
        assert_eq!(
            workspace
                .read_to_string("input.txt")
                .expect("read input fixture"),
            "hello"
        );
    }

    assert!(!path.exists());
}

#[cfg(test)]
#[test]
fn fake_clock_advances_without_sleeping() {
    let clock = FakeClock::epoch();
    clock.advance(Duration::from_secs(10));
    assert_eq!(
        clock
            .now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("duration"),
        Duration::from_secs(10)
    );
}

#[cfg(not(test))]
fn main() {
    println!("Run `cargo test --example test_support` to execute the table-driven example tests.");
}
