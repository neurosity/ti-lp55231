use crate::Channel;

/// Maximum number of instructions supported by programming engine.
///
/// There are 6 pages, each with 16 instructions.
pub const MAX_INSTRUCTIONS: u8 = 96;
/// Number of instructions per page.
pub const INSTRUCTIONS_PER_PAGE: u8 = 16;
/// Number of pages supported by programming engine.
pub const MAX_PAGES: u8 = 6;
/// Number of variables supported by programming engine.
pub const MAX_VARS: u8 = 4;

/// Programming engine variables.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Variable {
  A = 0,
  B,
  C,
  D,
}

/// Ramp step time span.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PreScale {
  // Cycle time of 0.488ms
  CT0_488 = 0,
  // Cycle time of 15.625ms
  CT15_625 = 1,
}

/// Ramp direction.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Direction {
  Up = 0,
  Down = 1,
}

/// Representation for a programming engine instruction.
///
/// Refer to spec sections 7.6.3 through 7.6.7
pub struct Instruction {
  pub msb: u8,
  pub lsb: u8,
}

impl Instruction {
  /// Word (u16) representation for an instruction.
  pub fn as_u16(&self) -> u16 {
    (self.msb as u16) << 8 | self.lsb as u16
  }
}

impl From<u16> for Instruction {
  /// Convert a word (u16) to an [Instruction].
  fn from(value: u16) -> Self {
    Self {
      msb: ((value >> 8) & 0xFF) as u8,
      lsb: (value & 0xFF) as u8,
    }
  }
}

impl Instruction {
  // Driver instructions

  pub fn ramp(
    cycle_time: PreScale,
    cycles_per_step: u8,
    direction: Direction,
    number_of_steps: u8,
  ) -> Self {
    let mut msb = cycles_per_step << 1; // TODO check bounds
    msb |= (cycle_time as u8) << 6;
    msb |= direction as u8;

    Self {
      msb,
      lsb: number_of_steps,
    }
  }

  pub fn ramp_from_vars(
    pre_scale: bool,
    ascending: bool,
    step_time_var: Variable,
    increments_var: Variable,
  ) -> Self {
    let mut lsb = ((step_time_var as u8) << 2) | (increments_var as u8);
    if pre_scale {
      lsb |= 1 << 6;
    }
    if ascending {
      lsb |= 1 << 5;
    }
    Self {
      msb: 0b1000_0100,
      lsb,
    }
  }

  pub fn set_pwm(value: u8) -> Self {
    Self {
      msb: 0b0100_0000,
      lsb: value,
    }
  }

  pub fn set_pwm_from_var(var: Variable) -> Self {
    Self {
      msb: 0b1000_0100,
      lsb: 0b0110_0000 | (var as u8),
    }
  }

  pub fn wait(cycle_time: PreScale, cycles: u8) -> Self {
    let mut msb = cycles << 1; // TODO check bounds
    msb |= (cycle_time as u8) << 6;
    Self {
      msb,
      lsb: 0b0000_0000,
    }
  }

  // Mapping instructions

  /// Create LED engine-to-LED mapping instruction.
  ///
  /// Associates the supplied [channels](Channel) with the the active engine.
  /// This information is not present in the main spec, but can be found in the
  /// [LP55231 evaluation kit](https://www.ti.com/lit/ug/snvu214b/snvu214b.pdf)
  /// User's Guide, on page 23.
  ///
  /// |Bit    |15|14|13|12|11|10|09|08|07|06|05|04|03|02|01|00|
  /// |-------|--|--|--|--|--|--|--|--|--|--|--|--|--|--|--|--|
  /// |Channel| -| -| -| -| -| -| -|D9|D8|D7|D6|D5|D4|D3|D2|D1|
  pub fn map_channels(channels: &[Channel]) -> Self {
    let mut map_bits = 0b0000_0000_0000_0000;
    for channel in channels.iter() {
      map_bits |= 1 << (*channel as u8);
    }
    Self::from(map_bits)
  }

  pub fn mux_ld_start(sram_address: u8) -> Self {
    Self {
      msb: 0b1001_1110,
      lsb: check_addr(sram_address),
    }
  }

  pub fn mux_map_start(sram_address: u8) -> Self {
    Self {
      msb: 0b1001_1100,
      lsb: check_addr(sram_address),
    }
  }

  pub fn mux_ld_end(sram_address: u8) -> Self {
    Self {
      msb: 0b1001_1100,
      lsb: 0b1000_0000 | check_addr(sram_address),
    }
  }

  pub fn mux_sel(led_select: u8) -> Self {
    Self {
      msb: 0b1001_1101,
      lsb: led_select,
    }
  }

  pub fn mux_clr() -> Self {
    Self {
      msb: 0b1001_1101,
      lsb: 0b0000_0000,
    }
  }

  pub fn mux_map_next() -> Self {
    Self {
      msb: 0b1001_1101,
      lsb: 0b1000_0000,
    }
  }

  pub fn mux_map_prev() -> Self {
    Self {
      msb: 0b1001_1101,
      lsb: 0b1100_0000,
    }
  }

  pub fn mux_ld_next() -> Self {
    Self {
      msb: 0b1001_1101,
      lsb: 0b1000_0001,
    }
  }

  pub fn mux_ld_prev() -> Self {
    Self {
      msb: 0b1001_1101,
      lsb: 0b1100_0001,
    }
  }

  pub fn mux_ld_addr(sram_address: u8) -> Self {
    Self {
      msb: 0b1001_1111,
      lsb: check_addr(sram_address),
    }
  }

  pub fn mux_map_addr(sram_address: u8) -> Self {
    Self {
      msb: 0b1001_1111,
      lsb: 0b1000_0000 | check_addr(sram_address),
    }
  }

  // Branch instructions

  pub fn rst() -> Self {
    Self {
      msb: 0b0000_0000,
      lsb: 0b0000_0000,
    }
  }

  pub fn branch(step_number: u8, loop_count: u8) -> Self {
    let mut bits: u16 = 0b1010_0000_0000_0000;
    bits |= step_number as u16; // TODO validate bounds
    bits |= (loop_count as u16) << 7; // TODO validate bounds
    Self::from(bits)
  }

  pub fn branch_vars(step_number: u8, loop_count_var: Variable) -> Self {
    let mut bits: u16 = 0b1000_0110_0000_0000;
    bits |= loop_count_var as u16;
    bits |= (step_number as u16) << 2; // TODO check bounds
    Self::from(bits)
  }

  pub fn int() -> Self {
    Self {
      msb: 0b1100_0100,
      lsb: 0b0000_0000,
    }
  }

  pub fn end(interrupt: bool, reset_program_counter: bool) -> Self {
    let mut msb = 0b1100_0000;
    if interrupt {
      msb |= 1 << 4;
    }
    if reset_program_counter {
      msb |= 1 << 3
    }
    Self {
      msb,
      lsb: 0b0000_0000,
    }
  }

  pub fn jne(
    num_instructions_to_skip: u8,
    var_1: Variable,
    var_2: Variable,
  ) -> Self {
    let mut instr: u16 = 0b1000_1000_0000_0000;
    instr |= var_1 as u16;
    instr |= (var_2 as u16) << 2;
    instr |= (num_instructions_to_skip as u16) << 4; // TODO check bounds.

    Self::from(instr)
  }

  pub fn jl(
    num_instructions_to_skip: u8,
    var_1: Variable,
    var_2: Variable,
  ) -> Self {
    let mut instr: u16 = 0b1000_1010_0000_0000;
    instr |= var_2 as u16;
    instr |= (var_1 as u16) << 2;
    instr |= (num_instructions_to_skip as u16) << 4; // TODO check bounds.

    Self::from(instr)
  }

  pub fn jge(
    num_instructions_to_skip: u8,
    var_1: Variable,
    var_2: Variable,
  ) -> Self {
    let mut instr: u16 = 0b1000_1100_0000_0000;
    instr |= var_2 as u16;
    instr |= (var_1 as u16) << 2;
    instr |= (num_instructions_to_skip as u16) << 4; // TODO check bounds.

    Self::from(instr)
  }

  pub fn je(
    num_instructions_to_skip: u8,
    var_1: Variable,
    var_2: Variable,
  ) -> Self {
    let mut instr: u16 = 0b1000_1110_0000_0000;
    instr |= var_2 as u16;
    instr |= (var_1 as u16) << 2;
    instr |= (num_instructions_to_skip as u16) << 4; // TODO check bounds.

    Self::from(instr)
  }

  pub fn ld(target_var: Variable, value: u8) -> Self {
    Self {
      msb: 0b1001_0000 | ((target_var as u8) << 2),
      lsb: value,
    }
  }

  pub fn add_numerical(target_var: Variable, value: u8) -> Self {
    Self {
      msb: 0b1001_0001 | ((target_var as u8) << 2),
      lsb: value,
    }
  }

  pub fn add_vars(
    target_var: Variable,
    var_1: Variable,
    var_2: Variable,
  ) -> Self {
    Self {
      msb: 0b1001_0011 | ((target_var as u8) << 2),
      lsb: ((var_1 as u8) << 2) | (var_2 as u8),
    }
  }

  pub fn sub_numerical(target_var: Variable, value: Variable) -> Self {
    Self {
      msb: 0b1001_0010 | ((target_var as u8) << 2),
      lsb: value as u8,
    }
  }

  pub fn sub_vars(
    target_var: Variable,
    var_1: Variable,
    var_2: Variable,
  ) -> Self {
    Self {
      msb: 0b1001_0011 | ((target_var as u8) << 2),
      lsb: 0b0001_0000 | ((var_1 as u8) << 2) | (var_2 as u8),
    }
  }
}

fn check_addr(addr: u8) -> u8 {
  if addr > MAX_INSTRUCTIONS {
    panic!(
      "invalid sram_address {} - max is {}",
      addr, MAX_INSTRUCTIONS
    )
  }
  addr
}
