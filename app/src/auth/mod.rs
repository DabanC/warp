pub mod anonymous_id;
pub mod auth_manager;
mod auth_override_warning_body;
pub mod auth_override_warning_modal;
pub mod auth_state;
mod auth_view_body;
pub mod auth_view_modal;
mod auth_view_shared_helpers;
pub mod credentials;
mod login_error_modal;
mod login_failure_notification;
pub mod login_slide;
pub mod needs_sso_link_view;
pub mod paste_auth_token_modal;
pub mod user;
pub mod user_uid;
#[cfg(target_family = "wasm")]
pub mod web_handoff;

pub use auth_manager::AuthManager;
pub use auth_state::AuthStateProvider;
pub use user_uid::UserUid;

use warpui::AppContext;

/// Prefix for API keys used in authentication
#[cfg_attr(target_family = "wasm", allow(dead_code))]
pub const API_KEY_PREFIX: &str = "wk-";

pub fn init(app: &mut AppContext) {
    auth_view_modal::init(app);
    auth_view_body::init(app);
    auth_override_warning_body::init(app);
    login_slide::init(app);
    paste_auth_token_modal::init(app);
}

/// Logout is disabled in the local-only fork because there is no Warp account session.
pub fn maybe_log_out(_app: &mut AppContext) {
    log::info!("Ignoring logout request in local-only build");
}

/// Logout is disabled in the local-only fork because there is no Warp account session.
pub fn log_out(_app: &mut AppContext) {
    log::info!("Ignoring logout request in local-only build");
}
