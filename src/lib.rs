#![doc = include_str!("../README.md")]
#![no_std]
#![deny(missing_docs)]

#[cfg(feature = "async")]
mod asynch;
mod register;
pub use register::*;
#[cfg(not(feature = "async"))]
mod sync;

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

/// Error type.
#[derive(Debug)]
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
