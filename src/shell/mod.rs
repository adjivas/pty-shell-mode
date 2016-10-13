pub mod display;
pub mod mode;
pub mod device;
mod err;
mod state;

use std::os::unix::io::AsRawFd;
use std::io::{self, Write};
use std::mem;

use ::libc;
use ::fork::Child;
use ::pty::prelude as pty;

use self::mode::Mode;
use self::device::Device;
pub use self::state::ShellState;
pub use self::err::{ShellError, Result};
pub use self::display::Display;

use super::terminal::Termios;

/// The struct `Shell` is the speudo terminal interface.

pub struct Shell {
  pid: libc::pid_t,
  mode: Mode,
  #[allow(dead_code)]
  config: Termios,
  speudo: pty::Master,
  device: Device,
  state: ShellState,
}

impl Shell {

  /// The constructor method `new` returns a shell interface according to 
  /// the command's option and a configured mode Line by Line.
  pub fn new (
    command: Option<&'static str>,
  ) -> Result<Self> {
    Shell::from_mode(command, Mode::None)
  }

  /// The constructor method `from_mode` returns a shell interface according to
  /// the command's option and the mode.
  pub fn from_mode (
    command: Option<&'static str>,
    mode: Mode,
  ) -> Result<Self> {
    match pty::Fork::from_ptmx() {
      Err(cause) => Err(ShellError::BadFork(cause)),
      Ok(fork) => match fork {
        pty::Fork::Child(ref slave) => slave.exec(command.unwrap_or("bash")),
        pty::Fork::Parent(pid, master) => {
        mem::forget(fork);
          Ok(Shell {
            pid: pid,
            config: Termios::default(),
            mode: mode,
            speudo: master,
            device: Device::from_speudo(master),
            state: ShellState::new(master.as_raw_fd()),
          })
        },
      },
    }
  }

  /// The accessor method `get_pid` returns the pid from the master.
  pub fn get_pid(&self) -> &libc::pid_t {
    &self.pid
  }

  /// The method `set_mode` changes the terminal mode.
  pub fn set_mode(&mut self, mode: Mode) {
    self.mode = mode;
  }

  /// The method `mode_pass` sends the input to the speudo terminal
  /// if the mode was defined with a procedure.
  fn mode_pass (
    &mut self,
    state: &ShellState
  ) {
    match self.mode {
      Mode::Character => {
        if let Some(text) = state.is_in_text() {
          self.write(text).unwrap();
          self.flush().unwrap();
        }
      },
      Mode::Line => {
        if let Some(line) = state.is_line() {
          self.write(&line[..]).unwrap();
          self.flush().unwrap();
        }
      },
      _ => {},
    }
  }
}

impl io::Write for Shell {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.speudo.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.speudo.flush()
  }
}

impl Drop for Shell {
  fn drop(&mut self) {
    unsafe {
      if libc::close(self.speudo.as_raw_fd()).eq(&-1) {
        unimplemented!()
      }
    }
  }
}

impl Iterator for Shell {
  type Item = ShellState;

  fn next(&mut self) -> Option<ShellState> {
    match self.device.next() {
      None => None,
      Some(event) => {
        if let Some(state) = self.state.with_device(event).ok() {
          self.mode_pass(&state);
          Some(state)
        }
        else {
          None
        }
      },
    }
  }
}