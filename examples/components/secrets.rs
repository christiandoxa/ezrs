//! Component example: redacted secrets with explicit exposure.

use ezrs::SecretString;

fn main() {
    let token = SecretString::new("local-dev-token");

    println!("token for logs: {token}");
    println!("token length for auth code: {}", token.expose().len());
}
