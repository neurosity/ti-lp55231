use std::{
  sync::{Arc, Mutex},
  thread::sleep,
  time::Duration,
};

use linux_embedded_hal::i2cdev::{
  core::I2CDevice,
  linux::{LinuxI2CDevice, LinuxI2CError},
};

pub mod debug;
mod mask;
mod program;
mod register;
mod types;

pub use mask::*;
pub use program::*;
pub use register::*;
pub use types::*;

/// Driver for Texas Instruments LP55231 I²C via [embedded-hal].
///
/// For more details, please refer to the [technical specs].
///
/// [embedded-hal]: https://docs.rs/embedded-hal
/// [technical specs]: (https://www.ti.com/lit/ds/symlink/lp55231.pdf).
pub struct LP55231 {
  device: LinuxI2CDevice,
  /// Enable debug output.
  ///
  /// Will print address and values for every I2C read and write instruction.
  pub debug_enabled: bool,
  /// Read-after-write verification
  pub verify_writes: bool,
  #[doc(hidden)]
  pub debug_depth: Arc<Mutex<usize>>,
}

impl LP55231 {
  /// Create a new LP55231 abstraction for the specified path and I2C address.
  pub fn create(path: &str, i2c_addr: u16) -> Result<Self, LinuxI2CError> {
    let device = LinuxI2CDevice::new(path, i2c_addr)?;
    Ok(Self {
      device,
      debug_enabled: false,
      verify_writes: false,
      debug_depth: Arc::new(Mutex::new(0)),
    })
  }

  /// Reset the IC.
  pub fn reset(&mut self) -> Result<(), LinuxI2CError> {
    debug::scope!(self, "reset()");

    // From spec: "Writing 11111111 into this register resets the LP55231"
    self.write_register(Register::RESET, 0b1111_1111)?;

    Ok(())
  }

  /// Test whether the IC is currently enabled.
  pub fn is_enabled(&mut self) -> Result<bool, LinuxI2CError> {
    debug::scope!(self, "is_enabled()");

    let value = self.read_register(Register::ENABLE_ENGINE_CNTRL1)?;
    Ok((value & Mask::CHIP_EN.bits()) > 0)
  }

  /// Enable or disable the IC.
  pub fn set_enabled(&mut self, enabled: bool) -> Result<(), LinuxI2CError> {
    debug::scope!(self, "set_enabled({})", enabled);

    let current_value = self.read_register(Register::ENABLE_ENGINE_CNTRL1)?;
    let new_value = Mask::CHIP_EN.apply(enabled as u8, current_value);
    self.write_register(Register::ENABLE_ENGINE_CNTRL1, new_value)
  }

  /// Read the current [misc](Misc) settings from the IC.
  pub fn get_misc_settings(&mut self) -> Result<Misc, LinuxI2CError> {
    debug::scope!(self, "get_misc_settings()");

    let value = self.read_register(Register::MISC)?;
    let misc = Misc {
      auto_increment_enabled: Mask::EN_AUTO_INCR.is_set(value),
      powersave_enabled: Mask::POWERSAVE_EN.is_set(value),
      charge_pump_mode: ChargePumpMode::from(Mask::CP_MODE.value(value)),
      pwm_powersave_enabled: (value & Mask::PWM_PS_EN.bits()) > 0,
      clock_selection: ClockSelection::from(Mask::CLK_DET_EN.value(value)),
    };

    Ok(misc)
  }

  /// Set [misc](Misc) settings for the IC.
  ///
  /// Overrides all existing settings.
  pub fn set_misc_settings(&mut self, misc: Misc) -> Result<(), LinuxI2CError> {
    debug::scope!(self, "set_misc_settings({:?})", misc);

    let en_auto_incr =
      Mask::EN_AUTO_INCR.with(misc.auto_increment_enabled as u8);
    let powersave_en = Mask::POWERSAVE_EN.with(misc.powersave_enabled as u8);
    let cp_mode = Mask::CP_MODE.with(misc.charge_pump_mode as u8);
    let pwm_ps_en = Mask::PWM_PS_EN.with(misc.pwm_powersave_enabled as u8);
    let clk_det_en_int_clk_en =
      Mask::CLK_DET_EN.with(misc.clock_selection as u8);

    let value =
      en_auto_incr | powersave_en | cp_mode | pwm_ps_en | clk_det_en_int_clk_en;

    self.write_register(Register::MISC, value)
  }

  /// Set the Pulse-Width Modulation (PWM) value for the specified [`Channel`].
  ///
  /// PWM controls luminance.
  pub fn set_channel_pwm(
    &mut self,
    channel: Channel,
    pwm: u8,
  ) -> Result<(), LinuxI2CError> {
    debug::scope!(
      self,
      "set_channel_pwm(channel: {:?}, pwm: {})",
      channel,
      pwm
    );

    self.write_register(Register::pwm_for(channel), pwm)?;

    Ok(())
  }

  /// Set the current value for the specified [`Channel`].
  ///
  /// Current controls luminous intensity (brightness).
  pub fn set_channel_current(
    &mut self,
    channel: Channel,
    current: u8,
  ) -> Result<(), LinuxI2CError> {
    debug::scope!(
      self,
      "set_channel_current(channel: {:?}, current: {})",
      channel,
      current
    );

    self.write_register(Register::current_control_for(channel), current)?;

    Ok(())
  }

  /// Enable or disable logarithmic brightness for the specified [`Channel`].
  pub fn set_log_brightness(
    &mut self,
    channel: Channel,
    enabled: bool,
  ) -> Result<(), LinuxI2CError> {
    debug::scope!(
      self,
      "set_log_brightness(channel: {:?}, enabled: {})",
      channel,
      enabled
    );

    // LOG_EN is a bit in D1_CTL; to change only that bit, the current value of
    // the byte must be read, modified, and written back (if different).
    let current_value = self.read_register(Register::control_for(channel))?;
    let new_value = Mask::LOG_EN.apply(enabled as u8, current_value);
    if new_value != current_value {
      self.write_register(Register::control_for(channel), new_value)?;
    }

    Ok(())
  }

  /// Enable or disable radiometric dimming for the specified [`Channel`].
  pub fn set_ratiometric_dimming(
    &mut self,
    channel: Channel,
    enabled: bool,
  ) -> Result<(), LinuxI2CError> {
    debug::scope!(
      self,
      "set_ratiometric_dimming(channel: {:?}, enabled: {})",
      channel,
      enabled
    );

    // Since D9 has its own register, there's no need to read-modify-write.
    if channel == Channel::D9 {
      return self.write_register(
        Register::OUTPUT_DIRECT_RATIOMETRIC_MSB,
        enabled as u8,
      );
    }

    // Registers D1 through D8 share same registry (bit 0 = D1, bit 7 = D8). To
    // change only the specified channel, whole register must be read and the
    // appropriate bit changed (if different).
    let current_value =
      self.read_register(Register::OUTPUT_DIRECT_RATIOMETRIC_LSB)?;
    let new_value = Mask::ratiometric_dimming_for(channel)
      .apply(enabled as u8, current_value);
    if new_value != current_value {
      self
        .write_register(Register::OUTPUT_DIRECT_RATIOMETRIC_LSB, new_value)?;
    }

    Ok(())
  }

  /// Enable or disable the specified [`Channel`].
  pub fn set_channel_enabled(
    &mut self,
    channel: Channel,
    enabled: bool,
  ) -> Result<(), LinuxI2CError> {
    debug::scope!(
      self,
      "set_channel_enabled(channel: {:?}, enabled: {})",
      channel,
      enabled
    );

    // D9 has its own register; no need to read-modify-write.
    if channel == Channel::D9 {
      return self
        .write_register(Register::OUTPUT_ON_OFF_CONTROL_MSB, enabled as u8);
    }

    let current_value =
      self.read_register(Register::OUTPUT_ON_OFF_CONTROL_LSB)?;
    let new_value =
      Mask::on_off_for(channel).apply(enabled as u8, current_value);
    if new_value != current_value {
      self.write_register(Register::OUTPUT_ON_OFF_CONTROL_LSB, new_value)?;
    }

    Ok(())
  }

  /// Assign the specified [`Channel`] to the specified [`Fader`].
  /// Removes [`Fader`] associations if `None` is supplied as an argument.
  ///
  /// [`Channel`] and [`Fader`] can be associated many-to-many, and any
  /// subsequent intensity adjustments to the fader will result in the same
  /// change to all of its assigned channels.
  pub fn assign_to_fader(
    &mut self,
    channel: Channel,
    fader: Option<Fader>,
  ) -> Result<(), LinuxI2CError> {
    debug::scope!(
      self,
      "assign_to_fader(channel: {:?}, fader: {:?})",
      channel,
      fader
    );

    let current_value = self.read_register(Register::control_for(channel))?;

    // 00 - none, 01 - F1, 02 - F2, 03 - F3
    let fader_assignment_bits = fader.map(|f| f as u8 + 1).unwrap_or(0b00);
    let new_value = Mask::MAPPING.apply(fader_assignment_bits, current_value);
    if new_value != current_value {
      self.write_register(Register::control_for(channel), new_value)?;
    }

    Ok(())
  }

  /// Adjust the intensity of the specified [`Fader`].
  ///
  /// Will result in the adjustment of the intensity of every [`Channel`]
  /// previously associated with the fader.
  pub fn set_fader_intensity(
    &mut self,
    fader: Fader,
    intensity: u8,
  ) -> Result<(), LinuxI2CError> {
    debug::scope!(
      self,
      "set_fader_intensity(fader: {:?}, intensity: {})",
      fader,
      intensity
    );

    self.write_register(Register::intensity_for(fader), intensity)
  }

  pub fn clear_interrupt(&mut self) -> Result<(), LinuxI2CError> {
    debug::scope!(self, "clear_interrupt()");

    self.read_register(Register::STATUS_INTERRUPT)?;

    Ok(())
  }

  /// Load the specified program.
  ///
  /// Accepts up to [`MAX_INSTRUCTIONS`], writing them over as many pages as
  /// necessary to fit the whole program.
  ///
  /// Since programming registers can only be accessed while the programming
  /// engines are in LOAD PROGRAM, this method:
  /// 1. Puts all engines in LOAD PROGRAM mode
  /// 2. Waits for the engine busy bit to clear
  /// 3. Writes program instructions to programming registers
  /// 4. Puts all engines in disabled mode
  ///
  /// After this method is called, relevant engines must be manually switched
  /// to run mode.
  ///
  /// After the program is loaded all `ENG* PROG START ADDR` values reset to
  /// default (see [`Self::set_engine_entry_point`]).
  pub fn load_program(
    &mut self,
    instructions: &[Instruction],
  ) -> Result<(), LinuxI2CError> {
    debug::scope!(self, "load_program([{} instructions])", instructions.len());

    validate_total_instruction_count(instructions)?;

    // 1. Set all engines to _load program_ mode.
    //
    // From the spec (section 7.6.2, page 28):
    //  "Load program mode can be entered from the disabled mode only.
    //  Entering load program mode from the run program mode is not allowed."
    //
    // Not clear in spec, but all engines must be disabled.
    self.set_all_engines_mode(EngineMode::Disabled)?;
    // From the spec (section 7.6.3, page 37):
    //  "in order to access program memory the operation mode needs to be
    //  load program"
    //
    // Not clear in spec, but all engines must be in load mode, otherwise
    // writes do not work (read-after-write returns empty program registers).
    self.set_all_engines_mode(EngineMode::LoadProgram)?;

    // 2. Wait until clear to enter load mode; from the spec (7.6.2, pg 28):
    //  "Serial bus master should check the busy bit before writing to program
    //  memory or allow at least 1ms delay after entering to load mode before
    //  memory write (...)"
    let poll_interval = Duration::from_millis(1);
    self.wait_while_engine_busy(poll_interval)?;
    sleep(poll_interval * 10);

    // optional step: ensure auto-increment is set to allow single I2C write
    // per program page (vs `2 * instructions.len()` writes if writing
    // instructions one-by-one).
    //
    // From the spec (section 7.5.2.3, page 20):
    //  "The auto-increment feature allows writing several consecutive
    //  registers within one transmission"
    let auto_incr = false;
    // TODO uncomment and change above to true.
    // let mut misc = self.get_misc_settings()?;
    // if !misc.auto_increment_enabled {
    //   misc.auto_increment_enabled = true;
    //   self.set_misc_settings(misc)?;
    // }

    // 3. Break program into pages of 16 instructions and write each page.
    let pages: Vec<&[Instruction]> = instructions.chunks(16).collect();
    for (page_num, page_instructions) in pages.iter().enumerate() {
      self.write_program_page(page_num as u8, page_instructions, auto_incr)?;
    }

    // 4. Set all engines back to disabled.
    self.set_all_engines_mode(EngineMode::Disabled)
  }

  /// Read a single program [`Instruction`] at the specified `index`, from the
  /// current page (i.e. the page selected via
  /// [PROG MEM PAGE SEL](Register::PROG_MEM_PAGE_SEL) register).
  pub fn read_program_instruction(
    &mut self,
    index: u8,
  ) -> Result<Instruction, LinuxI2CError> {
    validate_instruction_index(index)?;

    let register = (Register::PROG_MEM_BASE as u8) + (index * 2);
    let msb = self.device.smbus_read_byte_data(register)?;
    let lsb = self.device.smbus_read_byte_data(register + 1)?;
    debug::text!(
      self,
      "[{:02}] << {:02x} & {:02x} {:08b} {:08b} (0x{:02x}{:02x})",
      index,
      register,
      register + 1,
      msb,
      lsb,
      msb,
      lsb,
    );
    Ok(Instruction { msb, lsb })
  }

  /// Write up to [`INSTRUCTIONS_PER_PAGE`] program [instructions](Instruction)
  /// to the specified `page`.
  ///
  /// Arguments:
  /// * `page` - The page number to write. Must be in range \[0:5\]
  /// * `instructions` - List of instructions to write.
  /// * `at_once` - Whether to write all instructions in a single I2C write
  /// or use individual writes (each instruction is 16 bytes, which could result
  /// in up to 32 writes).
  ///
  /// `at_once` Should only be set to true if the device is configured with
  /// `EN_AUTO_INCR` (see [`Self::set_misc_settings`]).
  pub fn write_program_page(
    &mut self,
    page: u8,
    instructions: &[Instruction],
    at_once: bool,
  ) -> Result<(), LinuxI2CError> {
    validate_page(page)?;
    validate_per_page_instruction_count(instructions)?;

    debug::scope!(
      self,
      "write_program_page(page: {}, [{} instructions])",
      page,
      instructions.len()
    );

    // Set the page number...
    self.write_register(Register::PROG_MEM_PAGE_SEL, page)?;
    // ... and write the instructions.
    if at_once {
      panic!("not yet implemented");
      // TODO test single I2C writes relying on auto-increment
      // self.device.smbus_write_block_data(Register::PROG_MEM_BASE, ???)
    } else {
      for (index, instruction) in instructions.iter().enumerate() {
        self.write_program_instruction(index as u8, instruction)?;
      }
    }

    Ok(())
  }

  /// Write a single program [`Instruction`] at the specified index, to the
  /// current page (i.e. the page currently selected via
  /// [PROG MEM PAGE SEL](Register::PROG_MEM_PAGE_SEL)).
  pub fn write_program_instruction(
    &mut self,
    index: u8,
    instr: &Instruction,
  ) -> Result<(), LinuxI2CError> {
    validate_instruction_index(index)?;

    let register = (Register::PROG_MEM_BASE as u8) + (index * 2);
    // TODO single u16 write (requires auto-increment)
    self.device.smbus_write_byte_data(register, instr.msb)?;
    self.device.smbus_write_byte_data(register + 1, instr.lsb)?;
    debug::text!(
      self,
      "[{:02}] >> {:02x} & {:02x} {:08b} {:08b} (0x{:02x}{:02x})",
      index,
      register,
      register + 1,
      instr.msb,
      instr.lsb,
      instr.msb,
      instr.lsb,
    );
    Ok(())
  }

  /// Read a program page.
  ///
  /// Each page contains up to [`INSTRUCTIONS_PER_PAGE`]
  /// [instructions](Instruction).
  pub fn read_program_page(
    &mut self,
    page: u8,
    at_once: bool,
  ) -> Result<Vec<Instruction>, LinuxI2CError> {
    validate_page(page)?;

    debug::scope!(self, "read_program_page(page: {})", page);

    self.write_register(Register::PROG_MEM_PAGE_SEL, page)?;
    let mut instructions: Vec<Instruction> = vec![];
    if at_once {
      // TODO read whole page at once
      panic!("not implemented")
    } else {
      for i in 0..16 {
        let instruction = self.read_program_instruction(i)?;
        instructions.push(instruction);
      }
    }

    Ok(instructions)
  }

  /// Set the starting address for the specified [`Engine`] program instructions.
  ///
  /// Defaults:
  /// - Engine 1: 0
  /// - Engine 2: 8
  /// - Engine 3: 16
  pub fn set_engine_entry_point(
    &mut self,
    engine: Engine,
    entry_point: u8,
  ) -> Result<(), LinuxI2CError> {
    // TODO validate entry_point value fits 7 bits (i.e. <= MASK_ADDR)
    debug::scope!(
      self,
      "set_engine_entry_point(engine: {:?}, entry_point: {})",
      engine,
      entry_point
    );

    self.write_register(Register::program_start_for(engine), entry_point)
  }

  /// Set program counter value for the specified [`Engine`].
  ///
  /// NB: Program counter can only be modified if the engines are not running.
  pub fn set_engine_program_counter(
    &mut self,
    engine: Engine,
    pc: u8,
  ) -> Result<(), LinuxI2CError> {
    validate_program_counter(pc)?;

    debug::scope!(
      self,
      "set_engine_program_counter(engine: {:?}, pc: {})",
      engine,
      pc
    );

    let register = Register::program_counter_for(engine);
    self.write_register(register, pc)?;

    Ok(())
  }

  /// Set [program execution control](EngineExec) for the specified [`Engine`].
  pub fn set_engine_exec(
    &mut self,
    engine: Engine,
    exec_mode: EngineExec,
  ) -> Result<(), LinuxI2CError> {
    debug::scope!(
      self,
      "set_engine_exec(engine: {:?}, exec_mode: {:?})",
      engine,
      exec_mode
    );

    let current_value = self.read_register(Register::ENABLE_ENGINE_CNTRL1)?;
    let new_value =
      Mask::exec_for(engine).apply(exec_mode as u8, current_value);
    if new_value != current_value {
      self.write_register(Register::ENABLE_ENGINE_CNTRL1, new_value)?;
    }

    Ok(())
  }

  /// Convenience alias for [`Self::set_engine_modes`]
  /// that applies the same mode to all engines.
  pub fn set_all_engines_mode(
    &mut self,
    op_mode: EngineMode,
  ) -> Result<(), LinuxI2CError> {
    self.set_engine_modes(op_mode, op_mode, op_mode)
  }

  /// Set [`EngineMode`] for each of the programming engines.
  pub fn set_engine_modes(
    &mut self,
    engine1: EngineMode,
    engine2: EngineMode,
    engine3: EngineMode,
  ) -> Result<(), LinuxI2CError> {
    debug::scope!(
      self,
      "set_engine_modes(engine1: {:?}, engine2: {:?}, engine3: {:?})",
      engine1,
      engine2,
      engine3
    );

    let e1_bits = Mask::ENGINE1_MODE.with(engine1 as u8);
    let e2_bits = Mask::ENGINE2_MODE.with(engine2 as u8);
    let e3_bits = Mask::ENGINE2_MODE.with(engine3 as u8);

    let value = e1_bits | e2_bits | e3_bits;

    self.write_register(Register::ENGINE_CNTRL_2, value)
  }

  /// Set the [`EngineMode`] for the specified [`Engine`].
  pub fn set_engine_mode(
    &mut self,
    engine: Engine,
    op_mode: EngineMode,
  ) -> Result<(), LinuxI2CError> {
    debug::scope!(self, "set_engine_mode({:?}, {:?})", engine, op_mode);

    let current_value = self.read_register(Register::ENGINE_CNTRL_2)?;
    let new_value = Mask::mode_for(engine).apply(op_mode as u8, current_value);
    if new_value != current_value {
      self.write_register(Register::ENGINE_CNTRL_2, new_value)?;
    }

    Ok(())
  }

  /// Read a byte from the specified [`Register`].
  pub fn read_register(
    &mut self,
    register: Register,
  ) -> Result<u8, LinuxI2CError> {
    let value = self.device.smbus_read_byte_data(register as u8)?;
    debug::byte!(self, value, "<< {:02x} {:?}", register as u8, register);
    Ok(value)
  }

  /// Write a byte to the specified [`Register`].
  pub fn write_register(
    &mut self,
    register: Register,
    value: u8,
  ) -> Result<(), LinuxI2CError> {
    debug::byte!(self, value, ">> {:02x} {:?}", register as u8, register);
    self.device.smbus_write_byte_data(register as u8, value)?;

    if self.verify_writes {
      let post_write_value =
        self.device.smbus_read_byte_data(register as u8)?;
      if post_write_value != value {
        return Err(LinuxI2CError::Io(std::io::Error::new(
          std::io::ErrorKind::Other,
          format!(
            "write to register {:02x} {:?} failed; read-after-write expecting {:08b} but got {:08b}",
            register as u8, register, value, post_write_value,
          ),
        )));
      }
    }

    Ok(())
  }

  /// Wait for the `ENGINE_BUSY` bit to clear, polling at intervals of
  /// specified duration.
  ///
  /// Returns immediately if busy bit is not set.
  pub fn wait_while_engine_busy(
    &mut self,
    poll_interval: Duration,
  ) -> Result<(), LinuxI2CError> {
    loop {
      let value = self.read_register(Register::STATUS_INTERRUPT)?;
      if !Mask::ENGINE_BUSY.is_set(value) {
        return Ok(());
      }
      sleep(poll_interval);
    }
  }
}

fn validate_page(page: u8) -> Result<(), LinuxI2CError> {
  if page < 6 {
    return Ok(());
  }

  Err(LinuxI2CError::Io(std::io::Error::new(
    std::io::ErrorKind::Other,
    format!("invalid page ({}); must be in range [0:5]", page),
  )))
}

fn validate_instruction_index(index: u8) -> Result<(), LinuxI2CError> {
  if index < 16 {
    return Ok(());
  }

  Err(LinuxI2CError::Io(std::io::Error::new(
    std::io::ErrorKind::Other,
    format!(
      "invalid instruction index ({}); must be in range [0:15]",
      index
    ),
  )))
}

fn validate_per_page_instruction_count(
  instructions: &[Instruction],
) -> Result<(), LinuxI2CError> {
  if instructions.len() <= INSTRUCTIONS_PER_PAGE as usize {
    return Ok(());
  }

  Err(LinuxI2CError::Io(std::io::Error::new(
    std::io::ErrorKind::Other,
    format!(
      "too many instructions for a page ({}); limit is {}",
      instructions.len(),
      INSTRUCTIONS_PER_PAGE
    ),
  )))
}

fn validate_total_instruction_count(
  instructions: &[Instruction],
) -> Result<(), LinuxI2CError> {
  if instructions.len() <= MAX_INSTRUCTIONS as usize {
    return Ok(());
  }

  Err(LinuxI2CError::Io(std::io::Error::new(
    std::io::ErrorKind::Other,
    format!(
      "too many instructions ({}); limit is {}",
      instructions.len(),
      MAX_INSTRUCTIONS
    ),
  )))
}

fn validate_program_counter(counter: u8) -> Result<(), LinuxI2CError> {
  if counter < MAX_INSTRUCTIONS {
    return Ok(());
  }

  Err(LinuxI2CError::Io(std::io::Error::new(
    std::io::ErrorKind::Other,
    format!(
      "invalid program counter ({}); must be in range [0:{}]",
      counter, MAX_INSTRUCTIONS
    ),
  )))
}
