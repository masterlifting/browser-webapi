use std::env;

pub struct LaunchOptions {
  pub user_data_dir: String,
}

impl LaunchOptions {
  /// Create a `LaunchOptions` by reading environment variables.
  ///
  /// Environment variables used:
  /// - `USER_DATA_DIR` (required): path to the user data directory. If this variable is not set,
  ///   the call will panic via `expect("USER_DATA_DIR")`.
  ///
  /// # Panics
  ///
  /// Panics if the `USER_DATA_DIR` environment variable is not set.
  #[must_use]
  pub fn from_env() -> Self {
    let user_data_dir = env::var("USER_DATA_DIR").expect("USER_DATA_DIR");
    Self { user_data_dir }
  }
}
