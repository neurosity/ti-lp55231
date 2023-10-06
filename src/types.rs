/// Output channels.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Channel {
  D1 = 0,
  D2,
  D3,
  D4,
  D5,
  D6,
  D7,
  D8,
  D9,
}

/// Master faders.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Fader {
  F1 = 0,
  F2,
  F3,
}

/// Programming engines.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Engine {
  E1 = 0,
  E2,
  E3,
}

/// Engine execution control modes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EngineExec {
  Hold = 0,
  Step,
  Free,
  ExecuteOnce,
}

/// Engine modes (i.e. state).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EngineMode {
  Disabled = 0,
  LoadProgram,
  RunProgram,
  Halt,
}

/// Charge pump modes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ChargePumpMode {
  Off = 0,
  Bypass,
  Boosted,
  Auto,
}

impl From<u8> for ChargePumpMode {
  fn from(value: u8) -> Self {
    match value {
      0b00 => Self::Off,
      0b01 => Self::Bypass,
      0b10 => Self::Boosted,
      0b11 => Self::Auto,
      _ => panic!("invalid value for ChargePumpMode {:b}", value),
    }
  }
}

/// IC clock selection.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ClockSelection {
  ForceExternal = 0,
  ForceInternal,
  Automatic,
  PreferInternal,
}

impl From<u8> for ClockSelection {
  fn from(value: u8) -> Self {
    match value {
      0b00 => Self::ForceExternal,
      0b01 => Self::ForceInternal,
      0b10 => Self::Automatic,
      0b11 => Self::PreferInternal,
      _ => panic!("invalid value for ClockSelection {:b}", value),
    }
  }
}

/// Miscellaneous settings.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Misc {
  /// EN_AUTO_INCR
  pub auto_increment_enabled: bool,
  /// POWERSAVE_EN
  pub powersave_enabled: bool,
  /// CHARGE_PUMP_EN
  pub charge_pump_mode: ChargePumpMode,
  /// PWM_PS_EN
  pub pwm_powersave_enabled: bool,
  /// CLK_DET_EN and INT_CLK_EN
  pub clock_selection: ClockSelection,
}
