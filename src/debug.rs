/// Macros for low level debugging when developing against I2C devices.

/// Create a debugging scope on the provided context.
///
/// A debugging scope causes any subsequent [`text!`] or [`byte!`] calls to be
/// printed with additional padding and automatically ends once the current code
/// scope ends.
///
/// Scopes can be nested, where the inner scope will further pad any output.
///
/// The provided context (`ctx`) can be any struct that has two fields
/// accessible to these macros:
/// 1. `debug_enabled: bool`
/// 2. `debug_depth: Rc<Cell<usize>>`
///
/// Example:
/// ```
/// debug::scope!(ctx, "entering scope 1");
/// debug::text!(ctx, "message 1")
/// debug::scope!(ctx, "entering scope 2");
/// debug::text!(ctx, "message 2")
/// {
///   debug::scope!(ctx, "entering scope 3");
///   debug::text!(ctx, "message 3")
/// } // scope 3 ends
/// debug::text!("message 4")
/// ```
/// Prints:
/// ```text
/// entering scope 1
///   message 1
///   entering scope 2
///     message 2
///     entering scope 3
///       message 3
///     message 4
/// ```
///
/// The nesting ability of scopes becomes really useful when composing I2C
/// operations; example:
/// ```
/// fn start() {
///   debug::scope!("start()");
///   let byte = 1;
///   debug::byte!(byte, "write start byte", byte)
///   device.write(byte);
/// }
///
/// fn stop() {
///   debug::scope!("stop()");
///   let byte = 0;
///   debug::byte!(byte, "write stop byte", byte)
///   device.write(byte);
/// }
///
/// fn restart() {
///   debug::scope!("restart()");
///   stop();
///   start();
/// }
///
/// restart();
/// // ...
/// stop();
/// ```
///
/// Would result in the output:
/// ```text
/// restart()
///   stop()
///     00000000 write stop byte
///   start()
///     00000001 write start byte
/// stop()
///   00000000 write stop byte
/// ```
#[macro_export]
macro_rules! scope {
  ($ctx:ident, $fmt:expr $(, $arg:expr)* $(,)?) => {
    // _unused is dropped at end of scope where this macro is called, which
    // decrements debug depth.
    let _unused = if $ctx.debug_enabled {
      let mut cur_depth = $ctx.debug_depth.lock().unwrap();
      let padding = "  ".repeat(*cur_depth);
      println!("{}{} {{", padding, format!($fmt $(, $arg)*));
      *cur_depth += 1;
      // lock releases after return

      // debug_depth ArcMutex needs to be cloned since guard() takes ownership
      // of the arg. Code in block below is executed at the end of the code
      // scope where the `scope` macro is called.
      Some(scopeguard::guard($ctx.debug_depth.clone(), |depth| {
        let mut cur_depth = depth.lock().unwrap();
        if *cur_depth > 0 {
          *cur_depth -= 1;
        }
        println!("{}}}", "  ".repeat(*cur_depth));
      }))
    } else {
      None
    };
  };
}
pub use scope;

/// Print a formated message at current debug depth.
///
/// Example:
/// ```
/// debug::text!(ctx, "1. top-level debug text");
/// debug::scope!(ctx, "2. in-scope");
/// debug::text!(ctx, "2a. in-scope debug text");
/// ```
///
/// Prints:
/// ```text
/// 1. top-level debug text
/// 2. in-scope
///   2a. in-scope debug text
/// ```
#[macro_export]
macro_rules! text {
  ($ctx:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
    if $ctx.debug_enabled {
      let padding = "  ".repeat(*$ctx.debug_depth.lock().unwrap());
      println!("{}{}", padding, format!($fmt $(, $arg)*));
    };
  };
}
pub use text;

/// Print the binary representation for the given byte, along with a formatted
/// description at the current debug depth.
///
/// Example:
/// ```
/// let value = 0b0010_1010;
/// debug::byte(value, "is binary for {}", value);
/// ```
/// Prints:
/// ```text
/// 00101010 is binary for 42
/// ```
#[macro_export]
macro_rules! byte {
  ($ctx:expr, $value:expr, $($description:tt)*) => {
    if $ctx.debug_enabled {
      let value: u8 = $value;
      let formatted_string = format!($($description)*);
      debug::text!(
        $ctx,
        "{:08b} {}",
        value,
        formatted_string,
      );
    }
  };
}
pub use byte;
