#![doc = include_str!("../README.md")]
#![no_std]
#![deny(missing_docs)]

mod register;
pub use register::*;

/// LDC3114 has a fixed I2C address of 0x2A.
const I2C_ADDR: u8 = 0x2A;

/// Driver for the LDC3114.
pub struct Ldc3114<I2C> {
    i2c: I2C,
    sency0: u8,
    sency1: u8,
    sency2: u8,
    sency3: u8,
    lcdiv: u8,
}

impl<I2C, E> Ldc3114<I2C>
where
    I2C: embedded_hal_async::i2c::I2c + embedded_hal::i2c::ErrorType<Error = E>,
{
    /// Creates a new driver instance for the LDC3114.
    pub fn new(i2c: I2C) -> Self {
        Self { 
            i2c,
            sency0: 4,
            sency1: 4,
            sency2: 4,
            sency3: 4,
            lcdiv: 3,
        }
    }

    /// Reads the device ID.
    pub async fn read_device_id(&mut self) -> Result<u8, Error<E>> {
        self.read_register(Register::DeviceIdMsb).await
    }

    /// Reads the manufacturer ID.
    pub async fn read_manufacturer_id(&mut self) -> Result<u16, Error<E>> {
        let mut buffer = [0; 2];
        self.i2c.write_read(I2C_ADDR, &[Register::ManufacturerIdLsb.addr()], &mut buffer).await.map_err(Error::I2c)?;

        let data = u16::from_le_bytes(buffer);
        Ok(data)
    }

    /// Reads the status register.
    pub async fn read_status(&mut self) -> Result<Status, Error<E>> {
        let sr = self.read_register(Register::Status).await?;

        Ok(Status {
            output_status: (sr & OUT_STATUS != 0),
            chip_ready: (sr & CHIP_READY != 0),
            ready_to_write: (sr & RDY_TO_WRITE != 0),
            maximum_output_code: (sr & MAXOUT != 0),
            fsm_watchdog_error: (sr & FSM_WD != 0),
            lc_sensor_watchdog_error: (sr & LC_WD != 0),
            button_timeout: (sr & TIMEOUT != 0),
            register_integrity_bad: (sr & REGISTER_FLAG != 0),
        })
    }

    /// Checks if the registers are ready to be written.
    pub async fn is_ready_to_write(&mut self) -> Result<bool, Error<E>> {
        let sr = self.read_register(Register::Status).await?;
        let is_ready = (sr & RDY_TO_WRITE) != 0;
        Ok(is_ready)
    }

    /// Checks if the chip is ready after internal reset.
    pub async fn is_chip_ready(&mut self) -> Result<bool, Error<E>> {
        let sr = self.read_register(Register::Status).await?;
        let is_ready = (sr & CHIP_READY) != 0;
        Ok(is_ready)
    }

    /// Resets the device and register configurations.
    /// 
    /// All registers will be returned to default values.
    /// Normal operation will not resume until STATUS:CHIP_READY=1.
    pub async fn full_reset(&mut self) -> Result<(), Error<E>> {
        self.write_register(Register::Reset, FULL_RESET).await
    }

    /// Enter configuration mode.
    /// 
    /// Any device configuration changes should be made in this mode.
    pub async fn config_mode(&mut self) -> Result<(), Error<E>> {
        self.write_register(Register::Reset, CONFIG_MODE).await
    }

    /// Enter normal mode (exit configuration mode).
    pub async fn normal_mode(&mut self) -> Result<(), Error<E>> {
        let lcdiv = self.read_register(Register::LcDivider).await?;
        let scfg0 = self.read_register(Register::Sensor0Config).await?;
        let scfg1 = self.read_register(Register::Sensor1Config).await?;
        let scfg2 = self.read_register(Register::Sensor2Config).await?;
        let scfg3 = self.read_register(Register::Sensor3Config).await?;

        self.lcdiv = lcdiv & 0x07;
        self.sency0 = scfg0 & 0x1F;
        self.sency1 = scfg1 & 0x1F;
        self.sency2 = scfg2 & 0x1F;
        self.sency3 = scfg3 & 0x1F;

        self.write_register(Register::Reset, 0).await
    }

    /// Reads the channel output logic states.
    pub async fn read_output_logic_states(&mut self) -> Result<OutputLogicStates, Error<E>> {
        let out = self.read_register(Register::Out).await?;

        Ok(OutputLogicStates {
            new_data_available: (out & DATA_RDY != 0),
            out0: (out & OUT0 != 0), 
            out1: (out & OUT1 != 0), 
            out2: (out & OUT2 != 0), 
            out3: (out & OUT3 != 0), 
        })
    }

    /// Reads the button data for the given channel.
    pub async fn read_button_data(&mut self, ch: impl ChannelRegisters) -> Result<i16, Error<E>> {
        let mut buffer = [0; 2];
        self.i2c.write_read(I2C_ADDR, &[ch.data_lsb() as u8], &mut buffer).await.map_err(Error::I2c)?;

        let data = i16::from_le_bytes(buffer);
        Ok(data)
    }

    /// Reads the pre-processed raw sensor data for the given channel.
    /// 
    /// The value returned is given by the following formula:
    /// ```
    /// f_sensor = 30 * W * 44_000_000 / raw_data
    /// ```
    /// where
    /// ```
    /// W = 128 * (1 + SENCY_n) * (2^LCDIV)
    /// ```
    pub async fn read_raw_data<T: ChannelRegisters>(&mut self, ch: T) -> Result<u32, Error<E>> {
        let mut buffer = [0; 4];
        let slice = &mut buffer[0..=2];
        self.i2c.write_read(I2C_ADDR, &[ch.raw_data_lsb() as u8], slice).await.map_err(Error::I2c)?;

        let data = u32::from_le_bytes(buffer);
        if data == 0 {
            return Ok(0);
        }
        
        let sency = match T::CH {
            0 => self.sency0,
            1 => self.sency1,
            2 => self.sency2,
            3 => self.sency3,
            _ => unreachable!()
        };

        let w = 128 * (1 + sency as u32) * (2 << self.lcdiv as u32);
        let fsensor = 30 * w as u64 * 44_000_000 / data as u64;
        Ok(fsensor as u32)
    }

    /// Sets up the entire device configuration.
    pub async fn set_device_configuration(&mut self, config: &DeviceConfig) -> Result<(), Error<E>> {
        fn en_bits<T: ChannelRegisters>(_ch: T, mode: ChannelMode) -> u8 {
            match mode {
                ChannelMode::Disabled => 0x00,
                ChannelMode::NormalMode => T::EN_BIT,
                ChannelMode::NormalAndLowPowerMode => T::EN_BIT | T::LPEN_BIT
            }
        }

        fn btpause_maxwin_bits<T: ChannelRegisters>(_ch: T, btpause: bool, maxwin: bool) -> u8 {
            match (btpause, maxwin) {
                (false, false) => 0x00,
                (true, false) => T::BTPAUSE_BIT,
                (false, true) => T::MAXWIN_BIT,
                (true, true) => T::BTPAUSE_BIT | T::MAXWIN_BIT,
            }
        }

        fn common_deform_bits<T: ChannelRegisters>(_ch: T, common: bool, deform: bool) -> u8 {
            match (common, deform) {
                (false, false) => 0x00,
                (true, false) => T::ANTICOM_BIT,
                (false, true) => T::ANTIDFORM_BIT,
                (true, true) => T::ANTICOM_BIT | T::ANTIDFORM_BIT,
            }
        }

        fn opol_dpol_bits<T: ChannelRegisters>(_ch: T, opol: OutputPolarity, dpol: DataPolarity) -> u8 {
            match (opol, dpol) {
                (OutputPolarity::ActiveLow, DataPolarity::Inverted) => 0x00,
                (OutputPolarity::ActiveHigh, DataPolarity::Inverted) => T::OPOL_BIT,
                (OutputPolarity::ActiveLow, DataPolarity::Normal) => T::DPOL_BIT,
                (OutputPolarity::ActiveHigh, DataPolarity::Normal) => T::OPOL_BIT | T::DPOL_BIT,
            }
        }
        
        let mut en = en_bits(Channel0, config.ch0.mode);
        en |= en_bits(Channel1, config.ch1.mode);
        en |= en_bits(Channel2, config.ch2.mode);
        en |= en_bits(Channel3, config.ch3.mode);
        self.write_register(Register::En, en).await?;

        self.set_channel_gain(Channel0, config.ch0.gain).await?;
        self.set_channel_gain(Channel1, config.ch1.gain).await?;
        self.set_channel_gain(Channel2, config.ch2.gain).await?;
        self.set_channel_gain(Channel3, config.ch3.gain).await?;

        self.set_normal_scan_rate(config.scan_rate).await?;
        self.set_low_power_scan_rate(config.low_power_scan_rate).await?;

        let mut intpol = (config.enable_reset_of_button_baseline_tracking as u8) << 4;
        intpol |= (config.enable_button_press_detection_algorithm as u8) << 3;
        intpol |= (config.interrupt_polarity as u8) << 2;
        intpol |= ((!config.enable_button_timeout) as u8) << 1;
        intpol |= (!config.enable_max_out_check) as u8;
        self.write_register(Register::IntPol, intpol).await?;

        self.set_baseline_tracking_increment_np(config.baseline_tracking_increment_np).await?;
        self.set_baseline_tracking_increment_lp(config.baseline_tracking_increment_lp).await?;

        let mut btpause_maxwin = btpause_maxwin_bits(Channel0, config.ch0.baseline_tracking_pause, config.ch0.enable_max_win_button_algorithm);
        btpause_maxwin |= btpause_maxwin_bits(Channel1, config.ch1.baseline_tracking_pause, config.ch1.enable_max_win_button_algorithm);
        btpause_maxwin |= btpause_maxwin_bits(Channel2, config.ch2.baseline_tracking_pause, config.ch2.enable_max_win_button_algorithm);
        btpause_maxwin |= btpause_maxwin_bits(Channel3, config.ch3.baseline_tracking_pause, config.ch3.enable_max_win_button_algorithm);
        self.write_register(Register::BtPauseMaxWin, btpause_maxwin).await?;

        self.set_lc_divider(config.lc_divider).await?;
        self.set_hysteresis(config.hysteresis).await?;
        self.set_antitwist(config.antitwist).await?;

        let mut common_deform = common_deform_bits(Channel0, config.ch0.enable_anticommon_algorithm, config.ch0.enable_antideform_algorithm);
        common_deform |= common_deform_bits(Channel1, config.ch1.enable_anticommon_algorithm, config.ch1.enable_antideform_algorithm);
        common_deform |= common_deform_bits(Channel2, config.ch2.enable_anticommon_algorithm, config.ch2.enable_antideform_algorithm);
        common_deform |= common_deform_bits(Channel3, config.ch3.enable_anticommon_algorithm, config.ch3.enable_antideform_algorithm);
        self.write_register(Register::CommonDeform, common_deform).await?;

        let mut opol_dpol = opol_dpol_bits(Channel0, config.ch0.output_polarity, config.ch0.data_polarity);
        opol_dpol |= opol_dpol_bits(Channel1, config.ch1.output_polarity, config.ch1.data_polarity);
        opol_dpol |= opol_dpol_bits(Channel2, config.ch2.output_polarity, config.ch2.data_polarity);
        opol_dpol |= opol_dpol_bits(Channel3, config.ch3.output_polarity, config.ch3.data_polarity);
        self.write_register(Register::OpolDpol, opol_dpol).await?;

        let mut cntsc = (config.ch3.counter_scale as u8) << 6;
        cntsc |= (config.ch2.counter_scale as u8) << 4;
        cntsc |= (config.ch1.counter_scale as u8) << 2;
        cntsc |= config.ch1.counter_scale as u8;
        self.write_register(Register::Cntsc, cntsc).await?;

        self.set_sensor_config(Channel0, &config.ch0.sensor_config).await?;
        self.set_sensor_config(Channel1, &config.ch1.sensor_config).await?;
        self.set_sensor_config(Channel2, &config.ch2.sensor_config).await?;
        self.set_sensor_config(Channel3, &config.ch3.sensor_config).await?;

        self.set_fast_tracking_factor(Channel0, config.ch0.fast_tracking_factor).await?;
        self.set_fast_tracking_factor(Channel3, config.ch3.fast_tracking_factor).await?;

        let mut ftf1_2 = (config.ch2.fast_tracking_factor as u8) << 6;
        ftf1_2 |= (config.ch1.fast_tracking_factor as u8) << 4;
        self.write_register(Register::Ftf1_2, ftf1_2).await?;

        Ok(())
    }

    /// Configures a given channel.
    pub async fn configure_channel<T: ChannelRegisters>(&mut self, ch: T, config: &ChannelConfig) -> Result<(), Error<E>> {
        self.set_channel_mode(ch, config.mode).await?;
        self.set_channel_gain(ch, config.gain).await?;
        self.set_output_polarity(ch, config.output_polarity).await?;
        self.set_counter_scale(ch, config.counter_scale).await?;
        self.set_fast_tracking_factor(ch, config.fast_tracking_factor).await?;
        self.set_data_polarity(ch, config.data_polarity).await?;
        self.set_sensor_config(ch, &config.sensor_config).await?;
        self.include_channel_in_max_win_algorithm(ch, config.enable_max_win_button_algorithm).await?;
        self.include_channel_in_anticommon_algorithm(ch, config.enable_anticommon_algorithm).await?;
        self.include_channel_in_antideform_algorithm(ch, config.enable_antideform_algorithm).await?;
        self.set_baseline_tracking_pause(ch, config.baseline_tracking_pause).await?;
        Ok(())
    }

    /// Sets the operating mode for the given channel.
    pub async fn set_channel_mode<T: ChannelRegisters>(&mut self, _ch: T, mode: ChannelMode) -> Result<(), Error<E>> {
        match mode {
            ChannelMode::Disabled => {
                let bits = T::EN_BIT | T::LPEN_BIT;
                self.clear_register_bits(Register::En, bits).await
            }
            ChannelMode::NormalMode => {
                self.modify_register(Register::En, |mut v| {
                    v &= !T::LPEN_BIT;
                    v |= T::EN_BIT;
                    v
                }).await
            }
            ChannelMode::NormalAndLowPowerMode => {
                let bits = T::EN_BIT | T::LPEN_BIT;
                self.set_register_bits(Register::En, bits).await
            }
        }
    }

    /// Sets the gain for the given channel.
    pub async fn set_channel_gain<T: ChannelRegisters>(&mut self, ch: T, gain: u8) -> Result<(), Error<E>> {
        if gain >= 0x40 {
            return Err(Error::InvalidParameter);
        }
        self.write_register(ch.gain(), gain).await
    }

    /// Sets the scan rate in normal power mode.
    pub async fn set_normal_scan_rate(&mut self, sr: ScanRate) -> Result<(), Error<E>> {
        self.write_register(Register::NpScanRate, sr as u8).await
    }

    /// Sets the scan rate in low power mode.
    pub async fn set_low_power_scan_rate(&mut self, sr: LowPowerScanRate) -> Result<(), Error<E>> {
        self.write_register(Register::LpScanRate, sr as u8).await
    }

    /// Enables/disables setting MAXOUT bit if button algorithm generates codes
    /// outside maximum range.
    pub async fn enable_maxout_check(&mut self, enable: bool) -> Result<(), Error<E>> {
        if enable {
            self.set_register_bits(Register::IntPol, DIS_BTB_MO).await
        } else {
            self.clear_register_bits(Register::IntPol, DIS_BTB_MO).await
        }
    }

    /// Enables/disables button timeout if button is pressed for more than 50 seconds.
    pub async fn enable_button_timeout(&mut self, enable: bool) -> Result<(), Error<E>> {
        if enable {
            self.set_register_bits(Register::IntPol, DIS_BTN_TO).await
        } else {
            self.clear_register_bits(Register::IntPol, DIS_BTN_TO).await
        }
    }

    /// Sets the interrupt polarity of pin INTB.
    pub async fn set_interrupt_polarity(&mut self, polarity: InterruptPolarity) -> Result<(), Error<E>> {
        match polarity {
            InterruptPolarity::ActiveLow => {
                self.clear_register_bits(Register::IntPol, INTPOL).await
            }
            InterruptPolarity::ActiveHigh => {
                self.set_register_bits(Register::IntPol, INTPOL).await
            }
        }
    }

    /// Enables/disables the button press detection algorithm to assert events on OUT_X pins.
    pub async fn enable_button_press_detection_algorithm(&mut self, enable: bool) -> Result<(), Error<E>> {
        if enable {
            self.set_register_bits(Register::IntPol, BTN_ALG_EN).await
        } else {
            self.clear_register_bits(Register::IntPol, BTN_ALG_EN).await
        }
    }

    /// Enables/disables reset of button algorithm baseline tracking value.
    pub async fn enable_reset_of_button_baseline_tracking(&mut self, enable: bool) -> Result<(), Error<E>> {
        if enable {
            self.set_register_bits(Register::IntPol, BTSRT_EN).await
        } else {
            self.clear_register_bits(Register::IntPol, BTSRT_EN).await
        }
    }

    /// Sets the baseline tracking increment in normal power mode.
    pub async fn set_baseline_tracking_increment_np(&mut self, value: u8) -> Result<(), Error<E>> {
        if value >= 0x08 {
            return Err(Error::InvalidParameter);
        }
        self.write_register(Register::NpBaseInc, value).await
    }

    /// Sets the baseline tracking increment in low power mode.
    pub async fn set_baseline_tracking_increment_lp(&mut self, value: u8) -> Result<(), Error<E>> {
        if value >= 0x08 {
            return Err(Error::InvalidParameter);
        }
        self.write_register(Register::LpBaseInc, value).await
    }

    /// Configures baseline tracking to pause or not for the given channel
    /// when its corresponding OUT pin is asserted.
    pub async fn set_baseline_tracking_pause<T: ChannelRegisters>(&mut self, _ch: T, pause: bool) -> Result<(), Error<E>> {
        if pause {
            self.set_register_bits(Register::BtPauseMaxWin, T::BTPAUSE_BIT).await
        } else {
            self.clear_register_bits(Register::BtPauseMaxWin, T::BTPAUSE_BIT).await
        }
    }

    /// Configures whether to include or exclude the given channel
    /// from the Max-Win Button algorithm.
    pub async fn include_channel_in_max_win_algorithm<T:ChannelRegisters>(&mut self, _ch: T, include: bool) -> Result<(), Error<E>> {
        if include {
            self.set_register_bits(Register::BtPauseMaxWin, T::MAXWIN_BIT).await
        } else {
            self.clear_register_bits(Register::BtPauseMaxWin, T::MAXWIN_BIT).await
        }
    }

    /// Sets the LC oscillation frequency divider.
    pub async fn set_lc_divider(&mut self, value: u8) -> Result<(), Error<E>> {
        if value >= 0x08 {
            return Err(Error::InvalidParameter);
        }
        self.write_register(Register::LcDivider, value).await
    }

    /// Hysteresis for threshold for button algorithm.
    pub async fn set_hysteresis(&mut self, value: u8) -> Result<(), Error<E>> {
        if value >= 0x10 {
            return Err(Error::InvalidParameter);
        }
        self.write_register(Register::Hyst, value).await
    }

    /// Sets the anti-twist threshold value for the anti-twist button algorithm.
    pub async fn set_antitwist(&mut self, value: u8) -> Result<(), Error<E>> {
        if value >= 0x08 {
            return Err(Error::InvalidParameter);
        }
        self.write_register(Register::Twist, value).await
    }

    /// Configures whether to include or exclude the given channel
    /// from the Anti-Common Button algorithm.
    pub async fn include_channel_in_anticommon_algorithm<T: ChannelRegisters>(&mut self, _ch: T, include: bool) -> Result<(), Error<E>> {
        if include {
            self.set_register_bits(Register::CommonDeform, T::ANTICOM_BIT).await
        } else {
            self.clear_register_bits(Register::CommonDeform, T::ANTICOM_BIT).await
        }
    }

    /// Configures whether to include or exclude the given channel
    /// from the Anti-Deform Button algorithm.
    pub async fn include_channel_in_antideform_algorithm<T: ChannelRegisters>(&mut self, _ch: T, include: bool) -> Result<(), Error<E>> {
        if include {
            self.set_register_bits(Register::CommonDeform, T::ANTIDFORM_BIT).await
        } else {
            self.clear_register_bits(Register::CommonDeform, T::ANTIDFORM_BIT).await
        }
    }

    /// Sets the output polarity of the given channel.
    pub async fn set_output_polarity<T: ChannelRegisters>(&mut self, _ch: T, polarity: OutputPolarity) -> Result<(), Error<E>> {
        match polarity {
            OutputPolarity::ActiveLow => {
                self.clear_register_bits(Register::OpolDpol, T::OPOL_BIT).await
            }
            OutputPolarity::ActiveHigh => {
                self.set_register_bits(Register::OpolDpol, T::OPOL_BIT).await
            }
        }
    }

    /// Sets the data polarity of the given channel.
    pub async fn set_data_polarity<T: ChannelRegisters>(&mut self, _ch: T, polarity: DataPolarity) -> Result<(), Error<E>> {
        match polarity {
            DataPolarity::Inverted => {
                self.clear_register_bits(Register::OpolDpol, T::DPOL_BIT).await
            }
            DataPolarity::Normal => {
                self.set_register_bits(Register::OpolDpol, T::DPOL_BIT).await
            }
        }
    }

    /// Sets the counter scale for the given channel.
    pub async fn set_counter_scale<T: ChannelRegisters>(&mut self, _ch: T, scale: CounterScale) -> Result<(), Error<E>> {
        self.modify_register(Register::Cntsc, |mut v| {
            v &= !T::CNTSC_MASK;
            v | (scale as u8) << T::CNTSC_OFFSET
        }).await
    }

    /// Sets the sensor configuration for the given channel.
    pub async fn set_sensor_config<T: ChannelRegisters>(&mut self, ch: T, config: &SensorConfig) -> Result<(), Error<E>> {
        if config.cycle_count >= 0x20 {
            return Err(Error::InvalidParameter);
        }

        let mut value = config.cycle_count;
        value |= config.rp_range as u8;
        value |= config.frequency_range as u8;

        self.write_register(ch.sensor_config(), value).await
    }

    /// Sets the Fast Tracking Factor (FTF) for the given channel.
    pub async fn set_fast_tracking_factor<T: ChannelRegisters>(&mut self, ch: T, ftf: FastTrackingFactor) -> Result<(), Error<E>> {
        self.modify_register(ch.ftf(), |mut v| {
            v &= !T::FTF_MASK;
            v | (ftf as u8) << T::FTF_OFFSET
        }).await
    }

    /// Writes a value to a given register.
    pub async fn write_register(&mut self, register: Register, value: u8) -> Result<(), Error<E>> {
        if register.is_read_only() {
            return Err(Error::WriteToReadOnly);
        }

        self.i2c
            .write(I2C_ADDR, &[register.addr(), value])
            .await
            .map_err(Error::I2c)?;
        Ok(())
    }

    /// Reads a value from a given register.
    pub async fn read_register(&mut self, register: Register) -> Result<u8, Error<E>> {
        let mut buffer = [0u8; 1];
        self.i2c
            .write_read(I2C_ADDR, &[register.addr()], &mut buffer)
            .await
            .map_err(Error::I2c)?;
        Ok(buffer[0])
    }

    /// Modifies the value of a given register.
    pub async fn modify_register<F>(&mut self, register: Register, f: F) -> Result<(), Error<E>>
    where
        F: FnOnce(u8) -> u8,
    {
        let value = self.read_register(register).await?;
        self.write_register(register, f(value)).await
    }

    /// Sets some bits of a given register.
    pub async fn set_register_bits(
        &mut self,
        register: Register,
        bits: u8,
    ) -> Result<(), Error<E>> {
        self.modify_register(register, |v| v | bits).await
    }

    /// Clears some bits of a given register.
    pub async fn clear_register_bits(
        &mut self,
        register: Register,
        bits: u8,
    ) -> Result<(), Error<E>> {
        self.modify_register(register, |v| v & !bits).await
    }
}

/// Error type.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<I2cError> {
    /// I2C bus error.
    I2c(I2cError),
    /// Attempted to write to a read-only register.
    WriteToReadOnly,
    /// Invalid parameter.
    InvalidParameter,
}

/// Status flags.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status {
    /// Logic OR of output OUTx bits.
    /// This field is cleared by reading this register.
    pub output_status: bool,
    /// Chip ready after internal reset.
    pub chip_ready: bool,
    /// Indicates if registers are ready to be written.
    pub ready_to_write: bool,
    /// Indicates if any channel button output data reaches the maximum
    /// value (+0x7FF or -0x800). Cleared by a read of the status register.
    pub maximum_output_code: bool,
    /// Reports an error has occurred and conversions have been halted.
    /// Cleared by a read of the status register.
    pub fsm_watchdog_error: bool,
    /// Reports an error when any LC oscillator fails to start.
    /// Cleared by a read of the status register.
    pub lc_sensor_watchdog_error: bool,
    /// Reports when any button is asserted for more than 50 seconds.
    /// Cleared by a read of the status register.
    /// When `DIS_BTN_TO` is set, no timeout is asserted.
    pub button_timeout: bool,
    /// Reports if any register's value has an unexpected change.
    /// Cleared by a read of the status register.
    pub register_integrity_bad: bool,
}

/// Channel output logic states.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OutputLogicStates {
    /// Output Logic State for pre-processed data capture for any enabled channel.
    /// Bit cleared on read.
    pub new_data_available: bool,
    /// Button output logic state for channel 0.
    pub out0: bool,
    /// Button output logic state for channel 1.
    pub out1: bool,
    /// Button output logic state for channel 2.
    pub out2: bool,
    /// Button output logic state for channel 3.
    pub out3: bool,
}

/// Channel operational mode.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChannelMode {
    /// Channel disabled.
    Disabled,
    /// Channel enabled only in normal mode.
    NormalMode,
    /// Channel enabled both in normal and low power mode.
    NormalAndLowPowerMode,
}

/// Scan rate in normal mode.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ScanRate {
    /// Continuous scanning without delay.
    Continuous = 0x04,
    /// 160 SPS
    Highest = 0x08,
    /// 80 SPS
    High = 0x00,
    /// 40 SPS
    Medium = 0x01,
    /// 20 SPS
    Low = 0x02,
    /// 10 SPS
    Lowest = 0x03,
}

/// Scan rate in low power mode.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum LowPowerScanRate {
    /// 5 SPS
    Highest = 0x00,
    /// 2.5 SPS
    High = 0x01,
    /// 1.25 SPS
    Medium = 0x02,
    /// 0.625 SPS
    Low = 0x03,
}

/// Interrupt polarity.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum InterruptPolarity {
    /// Set INTB pin polarity to active low.
    ActiveLow = 0,
    /// Set INTB pin polarity to active high.
    ActiveHigh = 1,
}

/// Button output polarity for pin OUTX.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum OutputPolarity {
    /// Set OUTX polarity to active low.
    ActiveLow = 0,
    /// Set OUTX polarity to active high.
    ActiveHigh = 1,
}

/// Processed button algorithm data polarity for a channel.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum DataPolarity {
    /// Data decreases as sensor increases.
    Inverted,
    /// Data increases as sensor increases.
    Normal,
}

/// Counter scale.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
#[repr(u8)]
pub enum CounterScale {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

/// Channel sensor Rp range selection.
/// Set based on the actual sensor Rp physical parameter.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum RpRange {
    /// 50 Ω ≤ Rp ≤ 4 kΩ
    Rp50OhmTo4kOhm = 0x00,
    /// 800 Ω ≤ Rp ≤ 10 kΩ
    Rp800OhmTo10kOhm = 0x80,
}

/// Channel sensor frequency range selection.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum FrequencyRange {
    /// 1 MHz to 3.3 MHz
    Freq1MHzTo3_3MHz = 0x00,
    /// 3.3 MHz to 10 MHz
    Freq3_3MHzTo10MHz = 0x20,
    /// 10 MHz to 30 MHz
    Freq10MHzTo30MHz = 0x40,
}

/// Sensor configuration struct.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SensorConfig {
    /// Channel sensor Rp range selection.
    pub rp_range: RpRange,
    /// Channel sensor frequency range selection.
    pub frequency_range: FrequencyRange,
    /// Channel sensor cycle count.
    pub cycle_count: u8,
}

impl SensorConfig {
    /// Default value for [`SensorConfig`].
    pub const fn const_default() -> Self {
        Self {
            rp_range: RpRange::Rp50OhmTo4kOhm,
            frequency_range: FrequencyRange::Freq1MHzTo3_3MHz,
            cycle_count: 4,
        }
    }
}

/// Fast Tracking Factor for button algorithm.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))] 
#[allow(missing_docs)]
#[repr(u8)]
pub enum FastTrackingFactor {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

/// Channel configuration struct.
#[derive(Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ChannelConfig {
    /// Channel operating mode.
    pub mode: ChannelMode,
    /// Channel gain.
    pub gain: u8,
    /// Channel output polarity.
    pub output_polarity: OutputPolarity,
    /// Channel data polarity.
    pub data_polarity: DataPolarity,
    /// Channel counter scale.
    pub counter_scale: CounterScale,
    /// Channel sensor configuration.
    pub sensor_config: SensorConfig,
    /// Channel FTF for button algorithm.
    pub fast_tracking_factor: FastTrackingFactor,
    /// Whether to include the channel in the anticommon group.
    pub enable_anticommon_algorithm: bool,
    /// Whether to include the channel in the antideform group.
    pub enable_antideform_algorithm: bool,
    /// Whether to include the channel in the max-win group.
    pub enable_max_win_button_algorithm: bool,
    /// Whether to pause baseline tracking when OUTX is asserted.
    pub baseline_tracking_pause: bool,
}

impl ChannelConfig {
    /// Default value for [`ChannelConfig`].
    pub const fn const_default<T: ChannelRegisters>(_ch: T) -> Self {
        Self {
            mode: T::DEFAULT_MODE,
            gain: 0x28,
            output_polarity: OutputPolarity::ActiveLow,
            data_polarity: DataPolarity::Normal,
            counter_scale: CounterScale::One,
            sensor_config: SensorConfig::const_default(),
            fast_tracking_factor: FastTrackingFactor::One,
            enable_anticommon_algorithm: false,
            enable_antideform_algorithm: false,
            baseline_tracking_pause: false,
            enable_max_win_button_algorithm: false,
        }
    }
}

/// Device configuration struct.
#[derive(Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceConfig {
    /// Configuration for channel 0.
    pub ch0: ChannelConfig,
    /// Configuration for channel 1.
    pub ch1: ChannelConfig,
    /// Configuration for channel 2.
    pub ch2: ChannelConfig,
    /// Configuration for channel 3.
    pub ch3: ChannelConfig,
    /// Scan rate in normal power mode.
    pub scan_rate: ScanRate,
    /// Scan rate in low power mode.
    pub low_power_scan_rate: LowPowerScanRate,
    /// Check if button algorithm generates codes outside maximum range.
    pub enable_max_out_check: bool,
    /// Enable button time-out if if button pressed for more than 50 seconds.
    pub enable_button_timeout: bool,
    /// Interrupt polarity.
    pub interrupt_polarity: InterruptPolarity,
    /// Enable button press detection algorithm to assert events on OUTX pins.
    pub enable_button_press_detection_algorithm: bool,
    /// Enable reset of button algorithm baseline tracking value.
    pub enable_reset_of_button_baseline_tracking: bool,
    /// Normal power base increment for button algorithm.
    pub baseline_tracking_increment_np: u8,
    /// Low power base increment for button algorithm.
    pub baseline_tracking_increment_lp: u8,
    /// LC oscillation frequency divider.
    pub lc_divider: u8,
    /// Hysteresis for threshold for button algorithm.
    pub hysteresis: u8,
    /// Anti-twist for button algorithm.
    pub antitwist: u8,
}

impl DeviceConfig {
    /// Default value for [`DeviceConfig`].
    pub const fn const_default() -> Self {
        Self {
            ch0: ChannelConfig::const_default(Channel0),
            ch1: ChannelConfig::const_default(Channel1),
            ch2: ChannelConfig::const_default(Channel2),
            ch3: ChannelConfig::const_default(Channel3),
            scan_rate: ScanRate::Medium,
            low_power_scan_rate: LowPowerScanRate::Highest,
            enable_max_out_check: true,
            enable_button_timeout: true,
            interrupt_polarity: InterruptPolarity::ActiveLow,
            enable_button_press_detection_algorithm: true,
            enable_reset_of_button_baseline_tracking: true,
            baseline_tracking_increment_np: 0x03,
            baseline_tracking_increment_lp: 0x05,
            lc_divider: 0x03,
            hysteresis: 0x08,
            antitwist: 0x00,
        }
    }
}
