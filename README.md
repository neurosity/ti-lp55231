LP55231 Linux Rust Driver
-------------------------

[![Open in Dev Containers](https://img.shields.io/static/v1?label=Dev%20Containers&message=Open&color=blue&logo=visualstudiocode)](https://vscode.dev/redirect?url=vscode://ms-vscode-remote.remote-containers/cloneInVolume?url=https://github.com/neurosity/ti-lp55231)

Linux driver for [Texas Instruments LP55231](https://www.ti.com/product/LP55231),
a 9 channel RGB/White LED controller with internal program memory and integrated
charge Pump.

**Features:**
- Full implementation of I2C control interface in [datasheet](https://www.ti.com/lit/ds/symlink/lp55231.pdf)
- Ergonomic API to leverage the [programming engine](#example-effect-blinking)
- Easy to debug with optional features:
    - [read-after-write checks](#read-after-write-verifications) to validate register writes
    - [debug output](#debug-output) to see the value in a register before and after a write

## Initialization, setup, and preparing animations

This example covers a typical initialization of the driver, preparing three
LEDs (R, G, B) for animated effects.

```rust
use ti_lp55231::{
  Channel,
  ChargePumpMode,
  ClockSelection,
  Direction,
  Engine,
  EngineExec,
  EngineMode,
  Instruction,
  LP55231,
  PreScale,
}

// Create the driver
let path = "/dev/i2c-2";
let i2c_addr = 0x32;
let ic = LP55231::create(path, i2c_addr)?;

// Power and configure the driver.
ic.set_enabled(true)?;
ic.set_misc_settings(Misc {
  auto_increment_enabled: true,
  powersave_enabled: true,
  charge_pump_mode: ChargePumpMode::Auto,
  pwm_powersave_enabled: true,
  clock_selection: ClockSelection::ForceInternal,
})?;

// Channel assignment.
let (r, g, b) = (Channel::D7, Channel::D1, Channel::D2);

// Enable logarithmic brightness for a smoother ramp up effect.
ic.set_log_brightness(r, true)?;
ic.set_log_brightness(g, true)?;
ic.set_log_brightness(b, true)?;

// Enable ratiometric dimming to preserve the ratio between the
// RGB components of all mapped channels during animations
ic.set_ratiometric_dimming(r, true)?;
ic.set_ratiometric_dimming(g, true)?;
ic.set_ratiometric_dimming(b, true)?;

// Set color to orange
ic.set_channel_pwm(r, 255)?;
ic.set_channel_pwm(g, 128)?;
ic.set_channel_pwm(b, 0)?;

// Program the IC (see other example for implementations of `create_program`)
let instructions = create_program(&[r, g, b])?;
ic.load_program(&instructions)?;

// Wait for the ENGINE_BUSY bit to clear,
// indicating that all instructions have been loaded.
ic.wait_while_engine_busy(Duration::from_millis(10))?;

// Set up one of the programming engines to Halt & Hold (ready to execute).
let engine = Engine::E1;
ic.set_engine_exec(engine, EngineExec::Hold)?;
ic.set_engine_mode(engine, EngineMode::Halt)?;

// Run the effect
ic.set_engine_exec(engine, EngineExec::Free)?;
ic.set_engine_mode(engine, EngineMode::RunProgram)?;
```

## Example effect: blinking

This example is an implementation of `create_program` that prepares a blinking
effect to run in an endless loop.

```rust
fn create_program(channels_to_control: &[Channel]) -> [Instruction; 8] {
  [
    // ----- LED-to-Engine mapping table
    // 00. Map all target output channels to the programming engine for control.
    Instruction::map_channels(channels_to_control),

    // ----- blink effect start
    // 01-02. Set LED mapping table start/end index + activation.
    Instruction::mux_map_start(0),
    Instruction::mux_ld_end(0),
    // 03. Power all mapped LEDs off.
    Instruction::set_pwm(0),
    // 04. Wait ~0.5 seconds (15.625ms * 30).
    Instruction::wait(PreScale::CT15_625, 30),
    // 05. Set all LEDs to max brightness.
    Instruction::set_pwm(255),
    // 06. Wait ~0.5 seconds (15.625ms * 30).
    Instruction::wait(PreScale::CT15_625, 30),
    // 07. Loop back to beginning of blink effect index.
    Instruction::branch(1, 0),
  ]
}
```

## Example effect: glow

```rust
fn create_program(channels_to_control: &[Channel]) -> [Instruction; 9] {
  [
    // ----- LED-to-Engine mapping table
    // 00. Map all target output channels to the programming engine for control.
    Instruction::map_channels(channels_to_control),

    // ----- glow effect start
    // 01-02. Set LED mapping table start/end index + activation.
    Instruction::mux_map_start(0),
    Instruction::mux_ld_end(0),
    // 03. Quickly ramp up to max brightness.
    Instruction::ramp(PreScale::CT0_488, 4, Direction::Up, 255),
    // 04. Wait ~0.5 seconds (15.625ms * 30 = 468.75ms).
    Instruction::wait(PreScale::CT15_625, 30),
    // 05. Begin ramping brightness down to half (255 - 127 = 128).
    Instruction::ramp(PreScale::CT15_625, 4, Direction::Down, 127),
    // 06. Wait ~0.5 seconds (15.625ms * 30 = 468.75ms).
    Instruction::wait(PreScale::CT15_625, 30),
    // 07. Begin ramping brightness up to max (128 + 127 = 255).
    Instruction::ramp(PreScale::CT15_625, 4, Direction::Up, 127),
    // 08. Loop back to first step of effect.
    Instruction::branch(1, 0),
  ]
}
```

## Switching between effects

The programming engine supports up to 96 instructions, which gives you plenty of
room to set up multiple effects. To switch between effects:

- pause the programming engine
- update the program counter to first index of next effect
- unpause the programming engine.

Example:

```rust
// Pause engine execution.
ic.set_engine_exec(Engine::E1, EngineExec::Hold)?;
ic.wait_while_engine_busy(Duration::from_millis(1))?;

// Update the program counter to the starting instruction of the desired effect
// This example assumes we're jumping to instruction 42, of the possible 96
// programming memory addresses.
ic.set_engine_program_counter(Engine::E1, 42)?;

// Unpause the engine and begin the new animation.
ic.set_engine_exec(Engine::E1, EngineExec::Free)?;
ic.set_engine_mode(Engine::E1, EngineMode::RunProgram)?;
```

# Debugging

## Read-after-write verifications

Read-after-write checks can be enabled with:

```rust
let ic = LP55231::create(...)?;
ic.verify_writes = true;
```

This will cause the driver to perform a read after every I2C write instruction
to compare the value in the register. It will throw an exception if the read
value does not match the written value.

> [!NOTE]
> This is useful during development, especially around using the programming
> engines which must be in the correct internal state in order to allow changes.

## Debug output

When enabled via `debug_enabled` property, the driver will emit useful (but
rather verbose) output to help you understand the state of registers with every
read and write operation. Example:

```rust
let ic = LP55231::create(...)?;
ic.debug_enabled = true;
ic.set_enabled(true)?;
```

Will produce output:

```
set_enabled(true) {
  00000000 << 0x00 ENABLE_ENGINE_CNTRL1
  00100000 >> 0x00 ENABLE_ENGINE_CNTRL1
}
```

### Scoping debug output for multiple I2C calls

Scope for multiple debug calls can be combined with the `debug::scope!` macro:

```rust
fn multiple_i2c_calls(
  ic: &mut LP55231,
  value: bool,
) -> Result<(), LinuxI2CError> {
  debug::scope!(ic, "example({})", value);
  ic.set_enabled(value)?;
  ic.set_enabled(!value)?;
  Ok(())
}

multiple_i2_calls(true)?;
```

Would result in the following output:
```
example(true) {
  set_enabled(true) {
    00000000 << 0x00 ENABLE_ENGINE_CNTRL1
    00100000 >> 0x00 ENABLE_ENGINE_CNTRL1
  }
  set_enabled(false) {
    00100000 << 0x00 ENABLE_ENGINE_CNTRL1
    00000000 >> 0x00 ENABLE_ENGINE_CNTRL1
  }
}
```

See [debug.rs](src/debug.rs) docs for more details.

## Getting started with development

1. Clone the project and open the folder in VS Code
2. Accept plugin suggestions (dev container required in non-linux envs)
3. Re-open in dev container

> [!NOTE]
> This project uses [hermit](https://cashapp.github.io/hermit/) to manage the
> Rust toolchain for this project. No prior installation of Rust required.

## TODO

- [ ] Read/write pages in blocks (`at_once` param in `read/write_program_page`)
