use bitflags::bitflags;

use crate::{types::Engine, Channel};

bitflags! {
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
  /// Masks.
  ///
  /// Sorted by register they apply to.
  /// *Not* a comprehensive list.
  ///
  /// Refer to spec sections 7.6.1 and 7.6.2
  pub struct Mask: u8 {
    // 00, ENABLE / ENGINE CNTRL1
    const CHIP_EN = 0b0100_0000;
    const ENGINE1_EXEC = 0b0011_0000;
    const ENGINE2_EXEC = 0b0000_1100;
    const ENGINE3_EXEC = 0b0000_0011;

    // 01 ENGINE CNTRL2
    const ENGINE1_MODE = 0b0011_0000;
    const ENGINE2_MODE = 0b0000_1100;
    const ENGINE3_MODE = 0b0000_0011;

    // 02 OUTPUT DIRECT/RATIOMETRIC MSB
    const D9_RATIO_EN = 0b0000_0001;

    // 03 OUTPUT DIRECT/RATIOMETRIC LSB
    const D8_RATIO_EN = 0b1000_0000;
    const D7_RATIO_EN = 0b0100_0000;
    const D6_RATIO_EN = 0b0010_0000;
    const D5_RATIO_EN = 0b0001_0000;
    const D4_RATIO_EN = 0b0000_1000;
    const D3_RATIO_EN = 0b0000_0100;
    const D2_RATIO_EN = 0b0000_0010;
    const D1_RATIO_EN = 0b0000_0001;

    // 04 OUTPUT ON/OFF CONTROL MSB
    const D9_ON = 0b0000_0001;

    // 05 OUTPUT ON/OFF CONTROL LSB
    const D8_ON = 0b1000_0000;
    const D7_ON = 0b0100_0000;
    const D6_ON = 0b0010_0000;
    const D5_ON = 0b0001_0000;
    const D4_ON = 0b0000_1000;
    const D3_ON = 0b0000_0100;
    const D2_ON = 0b0000_0010;
    const D1_ON = 0b0000_0001;

    // 06-0E, D1-D9 CONTROL
    const LOG_EN = 0b0010_0000;
    const MAPPING = 0b1100_0000;

    // 36 MISC
    const EN_AUTO_INCR = 0b0100_0000;
    const POWERSAVE_EN = 0b0010_0000;
    const CP_MODE = 0b0001_1000;
    const PWM_PS_EN = 0b0000_0100;
    const CLK_DET_EN = 0b0000_0011;

    // 3A, STATUS/INTERRUPT
    const LEDTEST_MEAS_DONE = 0b1000_0000;
    const MASK_BUSY = 0b0100_0000;
    const STARTUP_BUSY = 0b0010_0000;
    const ENGINE_BUSY = 0b0001_0000;
    const EXT_CLK_USED = 0b0000_1000;
    const ENG1_INT = 0b0000_0100;
    const ENG2_INT = 0b0000_0010;
    const ENG3_INT = 0b0000_0001;

    // 3D, RESET
    const RESET = 0b1111_1111;

    // 4F, PROG MEM PAGE SELECT
    const PAGE_SEL = 0b0000_0111;
  }
}

impl Mask {
  /// Apply the specified `value` using the [Mask] to `byte`.
  ///
  /// Example; given:
  /// - A mask `0b0000_1100`
  /// - A value `0b10`
  /// - A byte `0b1111_1111`
  /// Then:
  /// - `mask.apply(value, byte)` will produce `0b1111_1011`
  pub fn apply(&self, value: u8, to_byte: u8) -> u8 {
    let byte_with_mask_bits_cleared = to_byte & !self.bits();
    let value_moved_to_mask_bits = value << self.bits().trailing_zeros();

    byte_with_mask_bits_cleared | value_moved_to_mask_bits
  }

  pub fn with(&self, value: u8) -> u8 {
    value << self.bits().trailing_zeros()
  }

  /// Returns the value set at the mask bits.
  ///
  /// Applies [Mask] bits to byte and shifts everything right `n` times, where
  /// `n` is the number of trailing zeroes.
  ///
  /// Example; given:
  /// - A byte `0b1100_1100`
  /// - A mask `0b0000_1100`
  ///
  /// Then:
  /// - `mask.value(byte)` will produce `0b11`
  pub fn value(&self, byte: u8) -> u8 {
    let value_at_mask_bits = byte & self.bits();
    value_at_mask_bits >> self.bits().trailing_zeros()
  }

  pub fn is_set(&self, byte: u8) -> bool {
    self.value(byte) > 0
  }
}

impl Mask {
  pub fn exec_for(engine: Engine) -> Mask {
    match engine {
      Engine::E1 => Mask::ENGINE1_EXEC,
      Engine::E2 => Mask::ENGINE2_EXEC,
      Engine::E3 => Mask::ENGINE3_EXEC,
    }
  }

  pub fn mode_for(engine: Engine) -> Mask {
    match engine {
      Engine::E1 => Mask::ENGINE1_MODE,
      Engine::E2 => Mask::ENGINE2_MODE,
      Engine::E3 => Mask::ENGINE3_MODE,
    }
  }

  pub fn ratiometric_dimming_for(channel: Channel) -> Mask {
    match channel {
      Channel::D1 => Mask::D1_RATIO_EN,
      Channel::D2 => Mask::D2_RATIO_EN,
      Channel::D3 => Mask::D3_RATIO_EN,
      Channel::D4 => Mask::D4_RATIO_EN,
      Channel::D5 => Mask::D5_RATIO_EN,
      Channel::D6 => Mask::D6_RATIO_EN,
      Channel::D7 => Mask::D7_RATIO_EN,
      Channel::D8 => Mask::D8_RATIO_EN,
      Channel::D9 => Mask::D9_RATIO_EN,
    }
  }

  pub fn on_off_for(channel: Channel) -> Mask {
    match channel {
      Channel::D1 => Mask::D1_ON,
      Channel::D2 => Mask::D2_ON,
      Channel::D3 => Mask::D3_ON,
      Channel::D4 => Mask::D4_ON,
      Channel::D5 => Mask::D5_ON,
      Channel::D6 => Mask::D6_ON,
      Channel::D7 => Mask::D7_ON,
      Channel::D8 => Mask::D8_ON,
      Channel::D9 => Mask::D9_ON,
    }
  }
}
