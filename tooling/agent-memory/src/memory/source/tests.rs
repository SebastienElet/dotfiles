use super::{read_bounded_regular_file, validated_https_url};
use rustix::fs::{Mode, OFlags, open};
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn official_url_policy_preserves_international_domains() -> Result<(), Box<dyn std::error::Error>> {
    let url = validated_https_url("https://bücher.example/proof")
        .map_err(|()| "international domain must be accepted")?;
    assert_eq!(url.as_str(), "https://xn--bcher-kva.example/proof");
    Ok(())
}

#[test]
fn official_url_policy_rejects_normalized_ip_hosts_and_invalid_punycode() {
    for locator in [
        "https://１２７。０。０。１/proof",
        "https://0x7f000001/proof",
        "https://xn--/proof",
        "https://reader@bücher.example/proof",
        "https://bücher.example/proof#fragment",
    ] {
        assert!(validated_https_url(locator).is_err(), "{locator}");
    }
}

#[test]
fn rejects_a_fifo_without_waiting_for_a_writer() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = tempfile::tempdir()?;
    let path = fixture.path().join("source");
    assert!(Command::new("mkfifo").arg(&path).status()?.success());
    let reader_path = path.clone();
    let (sender, receiver) = mpsc::channel();
    let reader = std::thread::spawn(move || sender.send(read_bounded_regular_file(&reader_path)));
    let result = receiver.recv_timeout(Duration::from_secs(1));
    let release = open(&path, OFlags::RDWR | OFlags::NONBLOCK, Mode::empty())?;
    reader
        .join()
        .map_err(|_| "source reader thread panicked")??;
    drop(release);

    let error = result?.err().ok_or("FIFO source must be rejected")?;
    assert_eq!(error.code(), "source_invalid");
    Ok(())
}
