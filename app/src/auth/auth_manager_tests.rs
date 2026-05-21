use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::{AuthManager, AuthManagerEvent};
use crate::auth::{auth_view_modal::AuthViewVariant, AuthStateProvider};
use crate::ServerApiProvider;
use warpui::{App, SingletonEntity};

fn initialize_app(app: &mut App) {
    app.add_singleton_model(|_ctx| ServerApiProvider::new_for_test());
    app.add_singleton_model(|_| AuthStateProvider::new_for_test());
    app.add_singleton_model(AuthManager::new_for_test);
}

#[test]
fn refresh_user_does_not_fetch_remote_user() {
    App::test((), |mut app| async move {
        initialize_app(&mut app);

        AuthManager::handle(&app).update(&mut app, |auth_manager, ctx| {
            auth_manager.refresh_user(ctx);
        });
    });
}

#[test]
fn skip_remote_anonymous_user_creation_emits_skipped_login() {
    App::test((), |mut app| async move {
        initialize_app(&mut app);

        let saw_skipped_login = Arc::new(AtomicBool::new(false));
        let saw_skipped_login_for_closure = saw_skipped_login.clone();

        app.update(|ctx| {
            ctx.subscribe_to_model(&AuthManager::handle(ctx), move |_, event, _| {
                if matches!(event, AuthManagerEvent::SkippedLogin) {
                    saw_skipped_login_for_closure.store(true, Ordering::Relaxed);
                }
            });
        });

        AuthManager::handle(&app).update(&mut app, |auth_manager, ctx| {
            auth_manager.skip_remote_anonymous_user_creation(ctx);
        });

        assert!(
            saw_skipped_login.load(Ordering::Relaxed),
            "local-only fork should emit SkippedLogin instead of creating a Firebase anonymous user"
        );
    });
}

#[test]
fn login_gated_feature_does_not_emit_auth_prompt() {
    App::test((), |mut app| async move {
        initialize_app(&mut app);

        let saw_login_gate = Arc::new(AtomicBool::new(false));
        let saw_login_gate_for_closure = saw_login_gate.clone();

        app.update(|ctx| {
            ctx.subscribe_to_model(&AuthManager::handle(ctx), move |_, event, _| {
                if matches!(event, AuthManagerEvent::AttemptedLoginGatedFeature { .. }) {
                    saw_login_gate_for_closure.store(true, Ordering::Relaxed);
                }
            });
        });

        AuthManager::handle(&app).update(&mut app, |auth_manager, ctx| {
            auth_manager.attempt_login_gated_feature(
                "local-only-test",
                AuthViewVariant::RequireLoginCloseable,
                ctx,
            );
        });

        assert!(
            !saw_login_gate.load(Ordering::Relaxed),
            "local-only fork must not emit auth prompt events"
        );
    });
}

#[test]
fn account_urls_are_local_only_about_blank() {
    App::test((), |mut app| async move {
        initialize_app(&mut app);

        AuthManager::handle(&app).update(&mut app, |auth_manager, _ctx| {
            assert_eq!(
                auth_manager.sign_up_url(),
                "about:blank#local-only-account-disabled"
            );
            assert_eq!(
                auth_manager.sign_in_url(),
                "about:blank#local-only-account-disabled"
            );
            assert_eq!(
                auth_manager.upgrade_url(),
                "about:blank#local-only-account-disabled"
            );
            assert_eq!(
                auth_manager.login_options_url("unused-token"),
                "about:blank#local-only-account-disabled"
            );
            assert_eq!(
                auth_manager.link_sso_url("user@example.com"),
                "about:blank#local-only-account-disabled"
            );
        });
    });
}

#[test]
fn set_user_onboarded_updates_local_state_only() {
    App::test((), |mut app| async move {
        initialize_app(&mut app);

        AuthManager::handle(&app).update(&mut app, |auth_manager, ctx| {
            auth_manager.set_user_onboarded(ctx);
        });

        app.update(|ctx| {
            assert_eq!(
                AuthStateProvider::as_ref(ctx).get().is_onboarded(),
                Some(true)
            );
        });
    });
}
