//! Embedded PTY pane — runs a shell inside a ratatui widget.
use anyhow::Result;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
    thread,
};

pub struct PtyPane {
    pub output: Arc<Mutex<String>>,
    writer: Box<dyn Write + Send>,
}

impl PtyPane {
    pub fn spawn(cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
        let mut cmd = CommandBuilder::new(&shell);
        cmd.env("TERM", "xterm-256color");
        pair.slave.spawn_command(cmd)?;

        let output = Arc::new(Mutex::new(String::new()));
        let output_clone = Arc::clone(&output);

        let mut reader = pair.master.try_clone_reader()?;
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                        if let Ok(mut out) = output_clone.lock() {
                            out.push_str(&chunk);
                            // Keep last 200 lines max
                            let lines: Vec<&str> = out.lines().collect();
                            if lines.len() > 200 {
                                *out = lines[lines.len() - 200..].join("\n");
                            }
                        }
                    }
                }
            }
        });

        let writer = pair.master.take_writer()?;
        Ok(Self { output, writer })
    }

    /// Send raw bytes to the shell (keypresses).
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data)?;
        Ok(())
    }

    /// Get current visible output as a string.
    pub fn read_output(&self) -> String {
        self.output.lock().map(|o| o.clone()).unwrap_or_default()
    }
}