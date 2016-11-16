pub const DEFAULT_REPEAT: libc::c_long = 1_000i64;
pub const DEFAULT_INTERVAL: libc::c_long = 1_000i64;

use std::io::Write;
use std::ops::BitOr;
use std::ops::{Add, Sub, BitAnd, Not};

use ::libc;
use ::time;

use super::Display;
use super::device::control::Control;

pub use super::device::{Out, DeviceState};
pub use super::device::control::operate::key::Key;
pub use super::device::control::operate::mouse::Mouse;

//pub type Buf = [libc::c_uchar; 100];

#[derive(Copy, Clone)]
pub struct ShellState {
  /// The time limit required for a repetition.
  repeat: libc::c_long,
  /// The time limit required for a repetition.
  interval: libc::c_long,
  /// Update.
  idle: Option<()>,
  /// Signal.
  sig: Option<libc::c_int>,
  /// The pressed character.
  in_down: Option<Control>,
  /// The released character.
  in_up: Option<Control>,
  /// The number of the repetition.
  in_repeat: Option<libc::size_t>,
  /// The segment intervals.
  in_interval: Option<time::Tm>,
  /// The output of last text printed.
  out_last: Option<(Out, libc::size_t)>,
// The tmp buffer
//  buffer: Buf,
}

impl ShellState {

    /// The constructor method `new` returns a empty ShellState.
    pub fn new (
        repeat: Option<libc::c_long>,
        interval: Option<libc::c_long>,
    ) -> Self {
        ShellState {
            repeat: repeat.unwrap_or(DEFAULT_REPEAT),
            interval: interval.unwrap_or(DEFAULT_INTERVAL),
            idle: None,
            sig: None,
            in_down: None,
            in_up: None,
            in_repeat: None,
            in_interval: None,
            out_last: None,
//            buffer: [0; 100],
        }
    }

    /// The mutator method `set_repeat` change the time limit of repetition.
    pub fn set_repeat(&mut self, repeat: libc::c_long) {
        self.repeat = repeat;
    }

    /// The mutator method `set_interval` change the interval.
    pub fn set_interval(&mut self, interval: libc::c_long) {
        self.interval = interval;
    }

    /// The mutator method `set_idle` update the idle event status.
    pub fn set_idle(&mut self, entry: Option<()>) {
        self.idle = entry;
    }

    /// The mutator method `set_signal` update the signal
    /// and can resize the Display interface.
    pub fn set_signal(&mut self, out_screen: &mut Display, signal: Option<libc::c_int>) {
        self.sig = signal;
        if let Some(()) = self.is_signal_resized() {
            out_screen.resize().unwrap();
        }
    }

    /// The mutator method `set_input` update the `in_text`
    /// and save the old `in_text` to `in_text_past`.
    pub fn set_input(&mut self, out_screen: &mut Display, mut down: Option<Control>) {

          if out_screen.ss()
          { let ss: libc::c_uchar = match down
            { Some(after) =>
              { match after.as_slice()
                { &[b'\x1B', b'[', b'A', ref next..] => b'A',
                  &[b'\x1B', b'[', b'B', ref next..] => b'B',
                  &[b'\x1B', b'[', b'C', ref next..] => b'C',
                  &[b'\x1B', b'[', b'D', ref next..] => b'D',
                  &[b'\x0D', ref next..] => b'M',
                  &[b'\x0A', ref next..] => b'M',
                  _ => 0, }},
              _ => 0, };
            if ss > 0
            { down = Some(Control::new([b'\x1B', b'O', ss, 0, 0, 0, 0, 0, 0, 0, 0, 0], 3)); }}

        self.in_down = down;
        if let Some(after) = down {
            if let Some(before) = self.in_up {
                if before.eq(&after).bitand(
                    before.as_time().add(
                        time::Duration::milliseconds(self.repeat)
                    ) >= after.as_time()
                ) {
                    self.in_repeat = Some(self.in_repeat.unwrap_or_default().add(&1));
                } else {
                    self.in_repeat = Some(0);
                }
            } else {
                self.in_interval = Some(after.as_time());
            }
            self.in_up = Some(after);
        } else if let Some(before) = self.in_up {
            if before.as_time().add(
               time::Duration::milliseconds(self.repeat)
            ) <= time::now() {
                self.in_repeat = None;
                self.in_interval = None;
            }
        }
    }

    /// The mutator method `set_output` update the both `out_text`
    /// and `out_screen` variable.
    pub fn set_output(&mut self, out_screen: &mut Display, entry: Option<(Out, libc::size_t)>) {
        if let Some((buf, len)) = entry {
            self.out_last = Some((buf, len));
            // ------ ESCAPE SAVE -------
/*            let mut get = buf;
            get.reverse();
            match get.iter().find(|&&x| x == b'\x1B')
            { Some(&n) =>
                { let (checker, _) = get.split_at(n as usize);
                  if checker.iter().find(|&&i| i.eq(&b';').not().bitand(i.eq(&b'\x1B').not()).bitand(i.lt(&b'0').bitor(i.gt(&b'9')))).is_none()
                  { // Need concat with next buffer
                    }},
              None => {}, }
  */          // ------ ESCAPE SAVE -------
            out_screen.write(&buf[..len]);

        } else {
            self.out_last = None;
        }
    }

    /// The accessor method `is_idle` returns the Idle event.
    pub fn is_idle(&self) -> Option<()> {
        self.idle
    }

    /// The accessor method `is_signal` returns the Signal event.
    pub fn is_signal(&self) -> Option<libc::c_int> {
        self.sig
    }

    /// The accessor method `is_signal_resized` returns the Option for
    /// the WINCH Signal event.
    pub fn is_signal_resized(&self) -> Option<()> {
        if let Some(libc::SIGWINCH) = self.sig {
          //println!("RESIZE");
            Some(())
        } else {
            None
        }
    }

    /// The accessor method `is_input_keydown` returns the pressed Key event.
    pub fn is_input_keydown(&self) -> Option<Key> {
        if let Some(ref control) = self.in_down {
            control.is_key()
        } else {
            None
        }
    }

    /// The accessor method `is_input_keyrepeat` returns the number's repetition
    /// of the Key.
    pub fn is_input_keyrepeat(&self) -> Option<libc::size_t> {
        if let Some(_) = self.in_up {
            self.in_repeat
        } else {
            None
        }
    }

    /// The accessor method `is_input_keyinterval` returns the number's of repetition
    /// between a range of the interval.
    pub fn is_input_keyinterval(&self) -> Option<i64> {
        if let (Some(first), Some(last)) = (self.in_interval, self.in_down) {
            Some(
                first.sub(last.as_time()).num_milliseconds().abs()/self.interval
            )
        } else {
            None
        }
    }

    /// The accessor method `is_input_mouse` returns the pressed Mouse event.
    pub fn is_input_mouse(&self) -> Option<(Mouse, libc::c_ushort, libc::c_ushort)> {
        if let Some(ref control) = self.in_down {
            control.is_mouse()
        } else {
            None
        }
    }

    /// The accessor method `is_in_slice` returns the bytes for a Input event.
    pub fn is_input_slice(&self) -> Option<&[libc::c_uchar]> {
        if let Some(ref control) = self.in_down {
            Some(control.as_slice())
        } else {
            None
        }
    }

    /// The accessor method `is_output_last` returns the Output text event.
    pub fn is_output_last(&self) -> Option<&[libc::c_uchar]> {
        if let Some((ref out, len)) = self.out_last {
            Some(&out[..len])
        } else {
            None
        }
    }

    /// The accessor method `is_output_screen` returns the Output screen event.
    pub fn is_output_screen(&self) -> Option<()> {
        if self.is_output_last().is_some().bitor(
            self.is_signal_resized().is_some()
        ) {
            Some(())
        } else {
            None
        }
    }

    /// The method `with_device` updates the state from
    /// the event DeviceState interface.
    pub fn clone_from(&mut self, out_screen: &mut Display, event: DeviceState) {
        self.set_idle(event.is_idle());
        self.set_signal(out_screen, event.is_signal());
        self.set_output(out_screen, event.is_out_text());
        self.set_input(out_screen, event.is_input());
    }
}
