//! https://docs.rs/not-io/0.1.0-alpha/not_io/
//! Provides `Read` and `Write` alternatives on `no_std` while being compatible with the full
//! traits from `std` when allowed.
//!
//! ## Motivation
//!
//! The file parser ecosystem of Rust is more or less split across crates that use `no_std` and
//! crates that do not, as well as between crates using `alloc` and no-alloc (and the largely
//! overlapping zero-copy) crates. This has several reasons:
//!
//! * The `std::io::Read` and `std::io::Write` traits require an allocator due to their internal
//!   implementation and were not written to be OS independent.
//! * Before `1.36` it was not possible to depend on `alloc` without `std`.
//! * The lack of specialization makes it hard to be both generic over implementors of the standard
//!   traits while still allowing use when those traits are not available. This is in particular
//!   also since several types (e.g. `&[u8]`) implement those traits but would obviously be useful
//!   as byte sources and sinks even when they are unavailable.
//!
//! ## Usage guide
//!
//! This crate assumes you have a structure declared roughly as follows:
//!
//! ```rust
//! # struct SomeItem;
//! # use std::io::Read;
//!
//! struct Decoder<T> {
//!     reader: T,
//! }
//!
//! impl<T: std::io::Read> Decoder<T> {
//!     fn next(&mut self) -> Result<SomeItem, std::io::Error> {
//!         let mut buffer = vec![];
//!         self.reader.read_to_end(&mut buffer)?;
//! # unimplemented!()
//!     }
//! }
//! ```
//!
//! There is only one necessary change, be sure to keep the `std` feature enabled for now. This
//! should not break any code except if you relied on the precise type `T` in which case you will
//! need to use a few derefs and/or `into_inner`.
//!
//! ```
//! use not_io::AllowStd;
//! # use std::io::Read;
//!
//! struct Decoder<T> {
//!     reader: AllowStd<T>,
//! }
//!
//! # struct SomeItem;
//! # impl<T: std::io::Read> Decoder<T> {
//! #    fn next(&mut self) -> Result<SomeItem, std::io::Error> {
//! #        let mut buffer = vec![];
//! #        self.reader.0.read_to_end(&mut buffer)?;
//! # unimplemented!()
//! #    }
//! # }
//! ```
//!
//! And finally you can add to your crate a new default feature which enables the `std`/`alloc`
//! feature of this crate, and conditionally active your existing interfaces only when that feature
//! is active. Then add a few new impls that can be used even when the feature is inactive.
//!
//! ```
//! use not_io::AllowStd;
//! # struct SomeItem;
//!
//! struct Decoder<T> {
//!     reader: AllowStd<T>,
//! }
//!
//! /// The interface which lets the caller select which feature to turn on.
//! impl<T> Decoder<T>
//! where
//!     AllowStd<T>: not_io::Read
//! {
//!     fn no_std_next(&mut self) -> Result<SomeItem, not_io::Error> {
//! # unimplemented!()
//!     }
//! }
//!
//! /// An interface for pure no_std use with caller provide no_std reader.
//! impl<T> Decoder<T>
//! where
//!     T: not_io::Read
//! {
//!     fn not_io_next(&mut self) -> Result<SomeItem, not_io::Error> {
//!         let reader = &mut self.reader.0;
//! # unimplemented!()
//!     }
//! }
//! ```
//!
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[derive(Debug)]
pub struct Error {
    _private: (),
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
}

/// A simple new type wrapper holding a potential reader or writer.
///
/// This type allows the library to satisfy the compatibility across different features without
/// having to resort to specialization. Simply put, this struct implements `Read` and `Write`:
///
/// * for all types that implement the respective trait from `std` if the `std` feature is active.
/// * on a concrete subset of those types if the `alloc` feature but not the `std` feature has been
///   turned on.
/// * only for types from `core` when neither feature is turned on.
///
/// Note that without this type we couldn't safely introduce a conditionally active, generic impl
/// of our own traits. The reason is that features must only activate SemVer compatible changes.
/// These two sets of impls are not SemVer compatible due to the uncovered generic `T`. In
/// particular in the first case you'd be allowed to implement the trait for your own type that
/// also implements `std::io::Read` while in the second this is an impl conflict.
///
/// * `impl Read for &'_ [u8]`
/// * `impl<T> Read for T where std::io::Read`
///
/// By adding our own private struct as a layer of indirection, you are no longer allowed to make
/// such changes:
///
/// * `impl Read for AllowStd<&'_ [u8]>`
/// * `impl<T> Read for AllowStd<T> where T: std::io::Read`
///
/// This still means there is one impl which will never be added. Instead, the impls for
/// core/standard types are provided separately and individually.
///
/// * `impl<T> Read for AllowStd<T> where T: crate::Read`
pub struct AllowStd<T>(pub T);

#[cfg(not(feature = "alloc"))]
mod impls_on_neither {}

#[cfg(feature = "alloc")]
mod impls_on_alloc {}

#[cfg(feature = "std")]
mod impls_on_std {
    use super::{AllowStd, Error, Result};
    use std::io::{self, IoSlice, IoSliceMut};

    impl<R: io::Read> super::Read for AllowStd<R> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            io::Read::read(&mut self.0, buf).map_err(Error::from)
        }
    }

    impl<R: io::Read> io::Read for AllowStd<R> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.read(buf)
        }
        fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
            self.0.read_vectored(bufs)
        }
        fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
            self.0.read_to_end(buf)
        }
        fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
            self.0.read_to_string(buf)
        }
        fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
            self.0.read_exact(buf)
        }
    }

    impl<W: io::Write> super::Write for AllowStd<W> {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            io::Write::write(&mut self.0, buf).map_err(Error::from)
        }
    }

    impl<W: io::Write> io::Write for AllowStd<W> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(buf)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.0.flush()
        }
        fn write_vectored(&mut self, bufs: &[IoSlice]) -> io::Result<usize> {
            self.0.write_vectored(bufs)
        }
        fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
            self.0.write_all(buf)
        }
    }

    impl From<io::Error> for Error {
        fn from(_: io::Error) -> Error {
            Error { _private: () }
        }
    }
}