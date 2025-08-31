use std::env;

pub struct LaunchOptions {
  pub headless: bool,
  pub user_data_dir: String,
  pub idle_timeout: std::time::Duration,
}

impl LaunchOptions {
  /// Create a `LaunchOptions` by reading environment variables.
  ///
  /// Environment variables used:
  /// - `USER_DATA_DIR` (required): path to the user data directory. If this variable is not set,
  ///   the call will panic via `expect("USER_DATA_DIR")`.
  /// - `USE_UI` (optional): interpreted as a `bool`. Missing or unparsable values fall back to `false`.
  ///   The resulting `headless` field is set to `!USE_UI`.
  /// - `IDLE_TIMEOUT_DAYS` (optional): interpreted as a `u64`. Missing or unparsable values fall back to `1`.
  ///   The value is converted to a `Duration` of that many days.
  ///
  /// Notes:
  /// - The only guaranteed panic is from the required `USER_DATA_DIR` lookup. Parsing errors for `USE_UI`
  ///   and `IDLE_TIMEOUT_DAYS` are handled by using sensible defaults.
  pub fn from_env() -> Self {
    let user_data_dir = env::var("USER_DATA_DIR").expect("USER_DATA_DIR");

    let use_ui = env::var("USE_UI")
      .unwrap_or_else(|_| "false".to_string())
      .parse::<bool>()
      .unwrap_or(false);

    let idle_timeout_days = env::var("IDLE_TIMEOUT_DAYS")
      .unwrap_or_else(|_| "1".to_string())
      .parse::<u64>()
      .unwrap_or(1);

    Self {
      headless: !use_ui,
      user_data_dir,
      idle_timeout: std::time::Duration::from_secs(idle_timeout_days * 60 * 60 * 24),
    }
  }
}
