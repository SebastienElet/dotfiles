use std::io;

pub(super) fn write_output(output: &str) -> io::Result<()> {
    if output.is_empty() {
        return Ok(());
    }

    let output = format!("{output}\n");
    let mut remaining = output.as_bytes();
    while !remaining.is_empty() {
        let written =
            rustix::io::write(rustix::stdio::stdout(), remaining).map_err(io::Error::from)?;
        if written == 0 {
            return Err(io::Error::from(io::ErrorKind::WriteZero));
        }
        remaining = &remaining[written..];
    }
    Ok(())
}
