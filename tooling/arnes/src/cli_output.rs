use std::io;
use std::os::fd::BorrowedFd;

pub fn write_output(output: &str) -> io::Result<()> {
    if output.is_empty() {
        return Ok(());
    }

    let output = format!("{output}\n");
    write_bytes(rustix::stdio::stdout(), output.as_bytes())
}

pub fn write_error(message: std::fmt::Arguments<'_>) -> io::Result<()> {
    write_bytes(rustix::stdio::stderr(), format!("{message}\n").as_bytes())
}

fn write_bytes(descriptor: BorrowedFd<'_>, mut remaining: &[u8]) -> io::Result<()> {
    while !remaining.is_empty() {
        let written = rustix::io::write(descriptor, remaining).map_err(io::Error::from)?;
        if written == 0 {
            return Err(io::Error::from(io::ErrorKind::WriteZero));
        }
        remaining = remaining.get(written..).ok_or(io::ErrorKind::InvalidData)?;
    }
    Ok(())
}
