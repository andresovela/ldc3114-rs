#![doc = include_str!("../README.md")]
#![no_std]
#![deny(missing_docs)]

/// Driver for the LDC3114
pub struct Ldc3114<I2C> {
    i2c: I2C,
    addr: u8,
}

impl<I2C, E> Ldc3114<I2C>
where
    I2C: embedded_hal_async::i2c::I2c + embedded_hal::i2c::ErrorType<Error = E>,
{
    /// Creates a new driver instance for the LDC3114
    pub fn new(i2c: I2C) -> Self {
        Self { i2c, addr: 0x2A }
    }

    /// Writes a value to a given register
    pub async fn write_register(&mut self, register: Register, value: u8) -> Result<(), Error<E>> {
        if register.is_read_only() {
            return Err(Error::WriteToReadOnly);
        }

        self.i2c
            .write(self.addr, &[register.addr(), value])
            .await
            .map_err(Error::I2c)?;
        Ok(())
    }

    /// Reads a value from a given register
    pub async fn read_register(&mut self, register: Register) -> Result<u8, Error<E>> {
        let mut buffer = [0u8; 1];
        self.i2c
            .write_read(self.addr, &[register.addr()], &mut buffer)
            .await
            .map_err(Error::I2c)?;
        Ok(buffer[0])
    }

    /// Modifies the value of a given register
    pub async fn modify_register<F>(&mut self, register: Register, f: F) -> Result<(), Error<E>>
    where
        F: FnOnce(u8) -> u8,
    {
        let value = self.read_register(register).await?;
        self.write_register(register, f(value)).await
    }

    /// Sets some bits of a given register
    pub async fn set_register_bits(&mut self, register: Register, bits: u8) -> Result<(), Error<E>> {
        self.modify_register(register, |v| v | bits).await
    }

    /// Clears some bits of a given register
    pub async fn clear_register_bits(&mut self, register: Register, bits: u8) -> Result<(), Error<E>> {
        self.modify_register(register, |v| v & !bits).await
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

/// LDC3114 registers
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Register {
    Status = 0x00,
}

impl Register {
    /// Get the address of the register
    pub fn addr(self) -> u8 {
        self as u8
    }

    /// Checks if the register is read-only
    pub fn is_read_only(self) -> bool {
        matches!(
            self,
            Register::Status
        )
    }
}