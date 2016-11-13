#![feature(slice_patterns)]
#![feature(advanced_slice_patterns)]
#![feature(integer_atomics)]

#![crate_type= "lib"]
#![cfg_attr(feature = "nightly", feature(plugin))]
#![cfg_attr(feature = "lints", plugin(clippy))]
#![cfg_attr(feature = "lints", deny(warnings))]
#![cfg_attr(not(any(feature = "lints", feature = "nightly")), deny())]
#![deny(
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]

#[macro_use(chan_select)] extern crate chan;
extern crate pty;
extern crate libc;
extern crate time;
extern crate vt100;
extern crate errno;

#[macro_use]
mod macros;
mod fork;
pub mod shell;
pub mod prelude;
