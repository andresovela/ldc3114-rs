use crate::ChannelMode;

/// LDC3114 registers.
#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Register {
    Status = 0x00,
    Out = 0x01,
    Data0Lsb = 0x02,
    Data0Msb = 0x03,
    Data1Lsb = 0x04,
    Data1Msb = 0x05,
    Data2Lsb = 0x06,
    Data2Msb = 0x07,
    Data3Lsb = 0x08,
    Data3Msb = 0x09,
    Reset = 0x0A,
    En = 0x0C,
    NpScanRate = 0x0D,
    Gain0 = 0x0E,
    LpScanRate = 0x0F,
    Gain1 = 0x10,
    IntPol = 0x11,
    Gain2 = 0x12,
    LpBaseInc = 0x13,
    Gain3 = 0x14,
    NpBaseInc = 0x15,
    BtPauseMaxWin = 0x16,
    LcDivider = 0x17,
    Hyst = 0x18,
    Twist = 0x19,
    CommonDeform = 0x1A,
    OpolDpol = 0x1C,
    Cntsc = 0x1E,
    Sensor0Config = 0x20,
    Sensor1Config = 0x22,
    Sensor2Config = 0x24,
    Ftf0 = 0x25,
    Sensor3Config = 0x26,
    Ftf1_2 = 0x28,
    Ftf3 = 0x2B,
    RawData0_3 = 0x59,
    RawData0_2 = 0x5A,
    RawData0_1 = 0x5B,
    RawData1_3 = 0x5C,
    RawData1_2 = 0x5D,
    RawData1_1 = 0x5E,
    RawData2_3 = 0x5F,
    RawData2_2 = 0x60,
    RawData2_1 = 0x61,
    RawData3_3 = 0x62,
    RawData3_2 = 0x63,
    RawData3_1 = 0x64,
    ManufacturerIdLsb = 0xFC,
    ManufacturerIdMsb = 0xFD,
    DeviceIdLsb = 0xFE,
    DeviceIdMsb = 0xFF,
}

impl Register {
    /// Get the address of the register.
    pub fn addr(self) -> u8 {
        self as u8
    }

    /// Checks if the register is read-only.
    pub fn is_read_only(self) -> bool {
        matches!(
            self,
            Register::Status
                | Register::Out
                | Register::Data0Lsb
                | Register::Data0Msb
                | Register::Data1Lsb
                | Register::Data1Msb
                | Register::Data2Lsb
                | Register::Data2Msb
                | Register::Data3Lsb
                | Register::Data3Msb
                | Register::RawData0_1
                | Register::RawData0_2
                | Register::RawData0_3
                | Register::RawData1_1
                | Register::RawData1_2
                | Register::RawData1_3
                | Register::RawData2_1
                | Register::RawData2_2
                | Register::RawData2_3
                | Register::RawData3_1
                | Register::RawData3_2
                | Register::RawData3_3
                | Register::DeviceIdLsb
                | Register::DeviceIdMsb
                | Register::ManufacturerIdLsb
                | Register::ManufacturerIdMsb
        )
    }
}

// STATUS
pub(crate) const OUT_STATUS: u8 = 0x80;
pub(crate) const CHIP_READY: u8 = 0x40;
pub(crate) const RDY_TO_WRITE: u8 = 0x20;
pub(crate) const MAXOUT: u8 = 0x10;
pub(crate) const FSM_WD: u8 = 0x08;
pub(crate) const LC_WD: u8 = 0x04;
pub(crate) const TIMEOUT: u8 = 0x02;
pub(crate) const REGISTER_FLAG: u8 = 0x01;

// OUT
pub(crate) const DATA_RDY: u8 = 0x10;
pub(crate) const OUT3: u8 = 0x08;
pub(crate) const OUT2: u8 = 0x04;
pub(crate) const OUT1: u8 = 0x02;
pub(crate) const OUT0: u8 = 0x01;

// RESET
pub(crate) const FULL_RESET: u8 = 0x10;
pub(crate) const CONFIG_MODE: u8 = 0x01;

// EN
pub(crate) const LPEN3: u8 = 0x80;
pub(crate) const LPEN2: u8 = 0x40;
pub(crate) const LPEN1: u8 = 0x20;
pub(crate) const LPEN0: u8 = 0x10;
pub(crate) const EN3: u8 = 0x08;
pub(crate) const EN2: u8 = 0x04;
pub(crate) const EN1: u8 = 0x02;
pub(crate) const EN0: u8 = 0x01;

// INTPOL
pub(crate) const BTSRT_EN: u8 = 0x10;
pub(crate) const BTN_ALG_EN: u8 = 0x08;
pub(crate) const INTPOL: u8 = 0x04;
pub(crate) const DIS_BTN_TO: u8 = 0x02;
pub(crate) const DIS_BTB_MO: u8 = 0x01;

// BTPAUSE_MAXWIN
pub(crate) const BTPAUSE3: u8 = 0x80;
pub(crate) const BTPAUSE2: u8 = 0x40;
pub(crate) const BTPAUSE1: u8 = 0x20;
pub(crate) const BTPAUSE0: u8 = 0x10;
pub(crate) const MAXWIN3: u8 = 0x08;
pub(crate) const MAXWIN2: u8 = 0x04;
pub(crate) const MAXWIN1: u8 = 0x02;
pub(crate) const MAXWIN0: u8 = 0x01;

// OPOL_DPOL
pub(crate) const OPOL3: u8 = 0x80;
pub(crate) const OPOL2: u8 = 0x40;
pub(crate) const OPOL1: u8 = 0x20;
pub(crate) const OPOL0: u8 = 0x10;
pub(crate) const DPOL3: u8 = 0x08;
pub(crate) const DPOL2: u8 = 0x04;
pub(crate) const DPOL1: u8 = 0x02;
pub(crate) const DPOL0: u8 = 0x01;

// COMMON_DEFORM
pub(crate) const ANTICOM3: u8 = 0x80;
pub(crate) const ANTICOM2: u8 = 0x40;
pub(crate) const ANTICOM1: u8 = 0x20;
pub(crate) const ANTICOM0: u8 = 0x10;
pub(crate) const ANTIDFORM3: u8 = 0x08;
pub(crate) const ANTIDFORM2: u8 = 0x04;
pub(crate) const ANTIDFORM1: u8 = 0x02;
pub(crate) const ANTIDFORM0: u8 = 0x01;

// CNTSC
pub(crate) const CNTSC3_MASK: u8 = 0xC0;
pub(crate) const CNTSC2_MASK: u8 = 0x30;
pub(crate) const CNTSC1_MASK: u8 = 0x0C;
pub(crate) const CNTSC0_MASK: u8 = 0x03;
pub(crate) const CNTSC3_OFFSET: u8 = 6;
pub(crate) const CNTSC2_OFFSET: u8 = 4;
pub(crate) const CNTSC1_OFFSET: u8 = 2;
pub(crate) const CNTSC0_OFFSET: u8 = 0;

// FTF0
pub(crate) const FTF0_MASK: u8 = 0x06;
pub(crate) const FTF0_OFFSET: u8 = 1;

// FTF1_2
pub(crate) const FTF2_MASK: u8 = 0xC0;
pub(crate) const FTF1_MASK: u8 = 0x30;
pub(crate) const FTF2_OFFSET: u8 = 6;
pub(crate) const FTF1_OFFSET: u8 = 4;

// FTF3
pub(crate) const FTF3_MASK: u8 = 0x03;
pub(crate) const FTF3_OFFSET: u8 = 0;

/// Channel registers.
pub trait ChannelRegisters: Copy {
    /// Channel number.
    const CH: u8;
    /// EN bit in the EN register.
    const EN_BIT: u8;
    /// LP_EN bit in the EN register.
    const LPEN_BIT: u8;
    /// BTPAUSE bit in the BTPAUSE_MAXWIN register.
    const BTPAUSE_BIT: u8;
    /// MAXWIN bit in the BTPAUSE_MAXWIN register.
    const MAXWIN_BIT: u8;
    /// OPOL bit in the OPOL_DPOL register.
    const OPOL_BIT: u8;
    /// DPOL bit in the OPOL_DPOL register.
    const DPOL_BIT: u8;
    /// ANTICOM bit in the COMMON_DEFORM register.
    const ANTICOM_BIT: u8;
    /// ANTIDFORM bit in the COMMON_DEFORM register.
    const ANTIDFORM_BIT: u8;
    /// Mask for the CNTSC bits in the CNTSC register.
    const CNTSC_MASK: u8;
    /// Offset to the CNTSC bits in the CNTSC register.
    const CNTSC_OFFSET: u8;
    /// Mask for the FTF bits in the FTF register for this channel.
    const FTF_MASK: u8;
    /// Offset to the FTF bits in the FTF register for this channel.
    const FTF_OFFSET: u8;
    /// Default mode.
    const DEFAULT_MODE: super::ChannelMode;
    /// Get the DATA_LSB register for this channel.
    fn data_lsb(&self) -> Register;
    /// Get the RAW_DATA_LSB register for this channel.
    fn raw_data_lsb(&self) -> Register;
    /// Get the GAIN register for this channel.
    fn gain(&self) -> Register;
    /// Get the SENSOR_CONFIG register for this channel.
    fn sensor_config(&self) -> Register;
    /// Get the FTF register for this channel.
    fn ftf(&self) -> Register;
}

/// Representation of registers for channel 0.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Channel0;

/// Representation of registers for channel 1.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Channel1;

/// Representation of registers for channel 2.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Channel2;

/// Representation of registers for channel 3.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Channel3;

macro_rules! impl_channel_registers {
    ($ChType:ident: $Ch:expr, $Data:ident, $RawData:ident, $Gain:ident, $Sensor:ident, $Ftf:ident, $En:expr, $Lpen:expr, $Btpause:expr, $Maxwin:expr, $Opol:expr, $Dpol:expr, $Anticom:expr, $Antidform:expr, $CntscMask:expr, $CntscOffset:expr, $FtfMask:expr, $FtfOffset:expr, $DefaultMode:ident) => {
        impl ChannelRegisters for $ChType {
            const CH: u8 = $Ch;
            const EN_BIT: u8 = $En;
            const LPEN_BIT: u8 = $Lpen;
            const BTPAUSE_BIT: u8 = $Btpause;
            const MAXWIN_BIT: u8 = $Maxwin;
            const OPOL_BIT: u8 = $Opol;
            const DPOL_BIT: u8 = $Dpol;
            const ANTICOM_BIT: u8 = $Anticom;
            const ANTIDFORM_BIT: u8 = $Antidform;
            const CNTSC_MASK: u8 = $CntscMask;
            const CNTSC_OFFSET: u8 = $CntscOffset;
            const FTF_MASK: u8 = $FtfMask;
            const FTF_OFFSET: u8 = $FtfOffset;
            const DEFAULT_MODE: super::ChannelMode = ChannelMode::$DefaultMode;
            fn data_lsb(&self) -> Register {
                Register::$Data
            }
            fn raw_data_lsb(&self) -> Register {
                Register::$RawData
            }
            fn gain(&self) -> Register {
                Register::$Gain
            }
            fn sensor_config(&self) -> Register {
                Register::$Sensor
            }
            fn ftf(&self) -> Register {
                Register::$Ftf
            }
        }
    };
}

impl_channel_registers!(Channel0: 0, Data0Lsb, RawData0_3, Gain0, Sensor0Config, Ftf0, EN0, LPEN0, BTPAUSE0, MAXWIN0, OPOL0, DPOL0, ANTICOM0, ANTIDFORM0, CNTSC0_MASK, CNTSC0_OFFSET, FTF0_MASK, FTF0_OFFSET, NormalAndLowPowerMode);
impl_channel_registers!(Channel1: 1, Data1Lsb, RawData1_3, Gain1, Sensor1Config, Ftf1_2, EN1, LPEN1, BTPAUSE1, MAXWIN1, OPOL1, DPOL1, ANTICOM1, ANTIDFORM1, CNTSC1_MASK, CNTSC1_OFFSET, FTF1_MASK, FTF1_OFFSET, NormalMode);
impl_channel_registers!(Channel2: 2, Data2Lsb, RawData2_3, Gain2, Sensor2Config, Ftf1_2, EN2, LPEN2, BTPAUSE2, MAXWIN2, OPOL2, DPOL2, ANTICOM2, ANTIDFORM2, CNTSC2_MASK, CNTSC2_OFFSET, FTF2_MASK, FTF2_OFFSET, NormalMode);
impl_channel_registers!(Channel3: 3, Data3Lsb, RawData3_3, Gain3, Sensor3Config, Ftf3, EN3, LPEN3, BTPAUSE3, MAXWIN3, OPOL3, DPOL3, ANTICOM3, ANTIDFORM3, CNTSC3_MASK, CNTSC3_OFFSET, FTF3_MASK, FTF3_OFFSET, NormalMode);
