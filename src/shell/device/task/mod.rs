mod err;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
mod ffi;

#[cfg(feature = "task")]
use std::io::Write;
use std::ops::{Not, BitAnd};

pub use self::err::{ProcError, Result};

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::linux::*;
#[cfg(target_os = "macos")]
pub use self::macos::*;

pub type BufProc = (libc::pid_t, [libc::c_uchar; 32]);

use ::libc;

/// The proc directory.
#[cfg(any(target_os = "linux", target_os = "android"))]
const SPEC_PROC: &'static str = "/proc";

/// The status's sub-directory.
#[cfg(any(target_os = "linux", target_os = "android"))]
const SPEC_SUBD_STATUS: &'static str = "status";

/// The default capacity of proc dictionary.
const SPEC_CAPACITY_PROC: usize = 512;

impl Proc {
    /// The constructor method `new` returns the list of process.
    pub fn new(fpid: libc::pid_t) -> Result<Self> {
        let mut status: Proc = Proc::default();

        status.fpid = fpid;
        status.with_list_process().and_then(|_| {
            Ok(status)
        })
    }


    /// The accessor method `get_name` returns the name of
    /// the process according to the pid.
    pub fn get_name(&self, pid: libc::pid_t)-> Option<BufProc> {
        self.list.iter().find(
            |&&(ref cpid, _, _, _)| pid.eq(cpid)
        ).and_then(|&(_, _, _, ref name): &(_, _, _, String)| {
            let mut source: [libc::c_uchar; 32] = [b'\0'; 32];
            {
                let mut buffer: &mut [libc::c_uchar] = &mut source[..];

                buffer.write(name.as_bytes());
            }
            Some((pid, source))
        })
    }

    /// The method `from_pid` returns the last active child process
    /// from the fpid process argument.
    fn from_pid(&self, fpid: Option<libc::pid_t>) -> Option<libc::pid_t> {
        if let Some(&(cpid, _, _, _)) = self.list.iter().find(
            |&&(_, ref ppid, _, _)| fpid.unwrap_or(self.fpid).eq(ppid)
        ) {
            self.from_pid(Some(cpid))
        }
        else {
            fpid.or(Some(self.fpid))
        }
    }
}

impl Iterator for Proc {
    type Item = BufProc;

    fn next(&mut self) -> Option<BufProc> {
        self.list.clear();
        self.with_list_process().unwrap();

        self.from_pid(None).and_then(|cfpid| {
//            print!("(current pid: {}) != (first top pid: {}) && (last saved pid: {})<>(first top pid: {}) -> ", cfpid, self.fpid, self.lpid, self.fpid);
            if cfpid.eq(&self.fpid).not().bitand(
               self.lpid.eq(&self.fpid).not()
            ) {
                self.fpid = cfpid;
//                println!("{:?}", self.get_name(cfpid));
                self.get_name(cfpid)
            } else {
//                println!("None");
                None
            }
        })
    }
}

impl Default for Proc {

    /// The constructor method `default` returns a empty list of process.
    fn default() -> Proc {
        Proc {
            fpid: 0,
            lpid: 0,
            list: Vec::with_capacity(SPEC_CAPACITY_PROC),
        }
    }
}