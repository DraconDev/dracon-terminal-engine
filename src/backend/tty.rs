use std::io;
use std::os::fd::AsRawFd;
use std::os::fd::BorrowedFd;

pub use libc::termios as Termios;

/// Get the current terminal attributes.
pub fn get_terminal_attr(fd: BorrowedFd) -> io::Result<Termios> {
    // SAFETY: fd is a valid open file descriptor (borrowed from stdin/stdout).
    // tcgetattr writes into a locally-owned Termios struct. The raw fd is
    // obtained via AsRawFd which guarantees it matches the borrowed fd.
    unsafe {
        let mut termios = std::mem::zeroed();
        if libc::tcgetattr(fd.as_raw_fd(), &mut termios) < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(termios)
    }
}

/// Set the terminal attributes.
pub fn set_terminal_attr(fd: BorrowedFd, termios: &Termios) -> io::Result<()> {
    // SAFETY: fd is a valid open file descriptor. termios is a valid reference
    // to a Termios struct previously obtained via tcgetattr. TCSANOW ensures
    // the change takes effect immediately.
    if unsafe { libc::tcsetattr(fd.as_raw_fd(), libc::TCSANOW, termios) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Modifies the termios to enable Raw Mode.
/// This uses `cfmakeraw` which is standard on Unix.
pub fn make_raw(termios: &mut Termios) {
    // SAFETY: cfmakeraw modifies the termios struct in place. The pointer is
    // valid and uniquely borrowed (mut reference). cfmakeraw is a well-known
    // POSIX helper that sets raw mode flags.
    unsafe { libc::cfmakeraw(termios) };
}

/// Get terminal window size (cols, rows).
pub fn get_window_size(fd: BorrowedFd) -> io::Result<(u16, u16)> {
    // SAFETY: fd is a valid open file descriptor. winsize is a small
    // zero-initialized C struct that ioctl will populate. TIOCGWINSZ is
    // a read-only ioctl that writes into the provided winsize buffer.
    unsafe {
        let mut winsize: libc::winsize = std::mem::zeroed();
        if libc::ioctl(fd.as_raw_fd(), libc::TIOCGWINSZ, &mut winsize) < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((winsize.ws_col, winsize.ws_row))
    }
}

/// Check if input is available within a timeout (milliseconds).
pub fn poll_input(fd: BorrowedFd, timeout_ms: i32) -> io::Result<bool> {
    // SAFETY: fd is a valid open file descriptor. pollfd is a stack-local
    // struct that poll() reads (fd, events) and writes (revents) into.
    // We pass nfds=1, so poll only accesses our single pollfd.
    unsafe {
        let mut fds = libc::pollfd {
            fd: fd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let ret = libc::poll(&mut fds, 1, timeout_ms);
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(ret > 0 && (fds.revents & libc::POLLIN) != 0)
    }
}
