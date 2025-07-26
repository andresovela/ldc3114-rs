#![doc = include_str!("../README.md")]
#![no_std]
#![deny(missing_docs)]

/// Driver for the LDC3114
pub struct Ldc3114<I2C> {
    i2c: I2C,
}

impl<I2C, E> Ldc3114<I2C>
where
    I2C: embedded_hal_async::i2c::I2c + embedded_hal::i2c::ErrorType<Error = E>,
{
    /// Creates a new driver instance for the LDC3114
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }
}

/// Error type
#[derive(Debug)]
pub enum Error<I2cError> {
    /// I2C bus error
    I2c(I2cError),
    /// Attempted to write to a read-only register
    WriteToReadOnly,
    /// Invalid parameter
    InvalidParameter,
}
