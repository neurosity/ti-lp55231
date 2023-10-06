use crate::types::{Channel, Engine, Fader};

// I2C registers.
#[allow(dead_code, non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Register {
  ENABLE_ENGINE_CNTRL1 = 0x00,
  ENGINE_CNTRL_2 = 0x01,
  OUTPUT_DIRECT_RATIOMETRIC_MSB = 0x02,
  OUTPUT_DIRECT_RATIOMETRIC_LSB = 0x03,
  OUTPUT_ON_OFF_CONTROL_MSB = 0x04,
  OUTPUT_ON_OFF_CONTROL_LSB = 0x05,
  D1_CONTROL = 0x06,
  D2_CONTROL = 0x07,
  D3_CONTROL = 0x08,
  D4_CONTROL = 0x09,
  D5_CONTROL = 0x0A,
  D6_CONTROL = 0x0B,
  D7_CONTROL = 0x0C,
  D8_CONTROL = 0x0D,
  D9_CONTROL = 0x0E,
  // 0f to 15 reserved
  D1_PWM = 0x16,
  D2_PWM = 0x17,
  D3_PWM = 0x18,
  D4_PWM = 0x19,
  D5_PWM = 0x1A,
  D6_PWM = 0x1B,
  D7_PWM = 0x1C,
  D8_PWM = 0x1D,
  D9_PWM = 0x1E,
  // 1f to 25 reserved
  D1_CURRENT_CONTROL = 0x26,
  D2_CURRENT_CONTROL = 0x27,
  D3_CURRENT_CONTROL = 0x28,
  D4_CURRENT_CONTROL = 0x29,
  D5_CURRENT_CONTROL = 0x2A,
  D6_CURRENT_CONTROL = 0x2B,
  D7_CURRENT_CONTROL = 0x2C,
  D8_CURRENT_CONTROL = 0x2D,
  D9_CURRENT_CONTROL = 0x2E,
  // 2f to 35 reserved
  MISC = 0x36,
  ENGINE1_PC = 0x37,
  ENGINE2_PC = 0x38,
  ENGINE3_PC = 0x39,
  STATUS_INTERRUPT = 0x3A,
  INT_GPO = 0x3B,
  VARIABLE = 0x3C,
  RESET = 0x3D,
  TEMP_ADC_CONTROL = 0x3E,
  TEMPERATURE_READ = 0x3F,
  TEMPERATURE_WRITE = 0x40,
  LED_TEST_CONTROL = 0x41,
  LED_TEST_ADC = 0x42,
  // 43 and 44 reserved
  ENGINE1_VARIABLE_A = 0x45,
  ENGINE1_VARIABLE_B = 0x46,
  ENGINE1_VARIABLE_C = 0x47,
  MASTER_FADER1 = 0x48,
  MASTER_FADER2 = 0x49,
  MASTER_FADER3 = 0x4A,
  // 4b reserved
  ENG1_PROG_START_ADDR = 0x4C,
  ENG2_PROG_START_ADDR = 0x4D,
  ENG3_PROG_START_ADDR = 0x4E,
  PROG_MEM_PAGE_SEL = 0x4F,
  PROG_MEM_BASE = 0x50,
}

impl Register {
  pub fn control_for(channel: Channel) -> Register {
    match channel {
      Channel::D1 => Register::D1_CONTROL,
      Channel::D2 => Register::D2_CONTROL,
      Channel::D3 => Register::D3_CONTROL,
      Channel::D4 => Register::D4_CONTROL,
      Channel::D5 => Register::D5_CONTROL,
      Channel::D6 => Register::D6_CONTROL,
      Channel::D7 => Register::D7_CONTROL,
      Channel::D8 => Register::D8_CONTROL,
      Channel::D9 => Register::D9_CONTROL,
    }
  }

  pub fn pwm_for(channel: Channel) -> Register {
    match channel {
      Channel::D1 => Register::D1_PWM,
      Channel::D2 => Register::D2_PWM,
      Channel::D3 => Register::D3_PWM,
      Channel::D4 => Register::D4_PWM,
      Channel::D5 => Register::D5_PWM,
      Channel::D6 => Register::D6_PWM,
      Channel::D7 => Register::D7_PWM,
      Channel::D8 => Register::D8_PWM,
      Channel::D9 => Register::D9_PWM,
    }
  }

  pub fn current_control_for(channel: Channel) -> Register {
    match channel {
      Channel::D1 => Register::D1_CURRENT_CONTROL,
      Channel::D2 => Register::D2_CURRENT_CONTROL,
      Channel::D3 => Register::D3_CURRENT_CONTROL,
      Channel::D4 => Register::D4_CURRENT_CONTROL,
      Channel::D5 => Register::D5_CURRENT_CONTROL,
      Channel::D6 => Register::D6_CURRENT_CONTROL,
      Channel::D7 => Register::D7_CURRENT_CONTROL,
      Channel::D8 => Register::D8_CURRENT_CONTROL,
      Channel::D9 => Register::D9_CURRENT_CONTROL,
    }
  }

  pub fn intensity_for(fader: Fader) -> Register {
    match fader {
      Fader::F1 => Register::MASTER_FADER1,
      Fader::F2 => Register::MASTER_FADER2,
      Fader::F3 => Register::MASTER_FADER3,
    }
  }

  pub fn program_start_for(engine: Engine) -> Register {
    match engine {
      Engine::E1 => Register::ENG1_PROG_START_ADDR,
      Engine::E2 => Register::ENG2_PROG_START_ADDR,
      Engine::E3 => Register::ENG3_PROG_START_ADDR,
    }
  }

  pub fn program_counter_for(engine: Engine) -> Register {
    match engine {
      Engine::E1 => Register::ENGINE1_PC,
      Engine::E2 => Register::ENGINE2_PC,
      Engine::E3 => Register::ENGINE3_PC,
    }
  }
}
