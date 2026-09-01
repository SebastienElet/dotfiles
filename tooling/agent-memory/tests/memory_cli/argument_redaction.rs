use super::support::{CliFixture, assert_error};

#[test]
fn invalid_argument_values_are_redacted() {
    let fixture = CliFixture::new();
    let secret = "ghp_abcdefghijklmnopqrstuvwxyz1234567890";
    let invalid_format = fixture.run(["admit", "--format", secret], b"");
    let invalid_status = fixture.run(
        [
            "confirm",
            "--id",
            "mem_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "--status",
            secret,
            "--reason-stdin",
        ],
        b"reason",
    );
    let invalid_agent = fixture.run(["hook", "--agent", secret], b"{}");
    let unknown = format!("--{secret}");
    let invalid_option = fixture.run(["admit", unknown.as_str()], b"");

    for output in [
        invalid_format,
        invalid_status,
        invalid_agent,
        invalid_option,
    ] {
        assert_error(&output, 2, "invalid_arguments");
        assert!(!String::from_utf8_lossy(&output.stderr).contains(secret));
    }
}
