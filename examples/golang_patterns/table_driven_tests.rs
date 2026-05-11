//! Go pattern: table-driven tests.

use ezrs::Result;

fn greet(name: &str) -> Result<String> {
    Ok(format!("hello {name}"))
}

#[ezrs::test]
async fn table_driven_greetings() {
    struct Case {
        name: &'static str,
        want: &'static str,
    }

    let cases = vec![
        Case {
            name: "Ayu",
            want: "hello Ayu",
        },
        Case {
            name: "Bima",
            want: "hello Bima",
        },
    ];

    for case in cases {
        assert_eq!(greet(case.name).expect("greet"), case.want);
    }
}

fn main() {
    let _ = greet("demo");
}
