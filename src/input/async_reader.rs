//! Async input reader for use with Tokio.
//!
//! Enable with `features = ["async"]` in Cargo.toml.

#[cfg(feature = "async")]
use tokio::sync::mpsc;

#[cfg(feature = "async")]
use tokio::time::{sleep, Duration};

/// Spawns a task that reads stdin asynchronously and invokes a callback for each input event.
#[cfg(feature = "async")]
pub struct AsyncInputReader;

#[cfg(feature = "async")]
impl AsyncInputReader {
    /// Spawns the async reader task and returns a handle to it.
    pub fn spawn<F>(mut callback: F) -> tokio::task::JoinHandle<()>
    where
        F: FnMut(crate::input::event::Event) + Send + 'static,
    {
        tokio::spawn(async move {
            let mut parser = crate::input::parser::Parser::new();

            loop {
                let bytes = tokio::task::block_in_place(|| {
                    use std::io::Read;
                    let mut stdin = std::io::stdin();
                    let mut buf = [0u8; 1024];
                    stdin.read(&mut buf).map(|n| buf[..n].to_vec())
                });

                let bytes = match bytes {
                    Ok(b) => b,
                    Err(_) => break,
                };

                if bytes.is_empty() {
                    break;
                }

                for &byte in &bytes {
                    if let Some(event) = parser.advance(byte) {
                        callback(event);
                    }
                }

                sleep(Duration::from_millis(20)).await;

                if let Some(evt) = parser.check_timeout() {
                    callback(evt);
                }
            }
        })
    }

    /// Spawns the async reader and returns a handle plus a guard for graceful shutdown.
    pub fn spawn_with_shutdown<F>(mut callback: F) -> (tokio::task::JoinHandle<()>, ShutdownGuard)
    where
        F: FnMut(crate::input::event::Event) + Send + 'static,
    {
        let (tx, rx) = mpsc::channel(1);
        let handle = tokio::spawn(async move {
            let mut parser = crate::input::parser::Parser::new();
            let mut rx = rx;

            loop {
                let bytes = tokio::task::block_in_place(|| {
                    use std::io::Read;
                    let mut stdin = std::io::stdin();
                    let mut buf = [0u8; 1024];
                    stdin.read(&mut buf).map(|n| buf[..n].to_vec())
                });

                let bytes = match bytes {
                    Ok(b) => b,
                    Err(_) => break,
                };

                if bytes.is_empty() {
                    break;
                }

                for &byte in &bytes {
                    if let Some(event) = parser.advance(byte) {
                        callback(event);
                    }
                }

                tokio::select! {
                    _ = rx.recv() => {
                        break;
                    }
                    _ = sleep(Duration::from_millis(20)) => {
                        if let Some(evt) = parser.check_timeout() {
                            callback(evt);
                        }
                    }
                }
            }
        });
        (handle, ShutdownGuard { tx })
    }
}

/// A guard that keeps the async reader task alive until dropped.
#[cfg(feature = "async")]
pub struct ShutdownGuard {
    tx: mpsc::Sender<()>,
}

#[cfg(feature = "async")]
impl ShutdownGuard {
    /// Signals the async reader to shut down.
    pub fn shutdown(self) {
        drop(self.tx);
    }
}
