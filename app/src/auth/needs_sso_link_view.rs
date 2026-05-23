use crate::auth::login_error_modal::LoginErrorModal;
use warpui::{AppContext, Element, Entity, TypedActionView, View};

pub struct NeedsSsoLinkView;

impl NeedsSsoLinkView {
    pub fn new() -> Self {
        Self
    }

    pub fn set_email(&mut self, _email: String) {}
}

impl Entity for NeedsSsoLinkView {
    type Event = ();
}

impl View for NeedsSsoLinkView {
    fn ui_name() -> &'static str {
        "NeedsSsoLinkView"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        LoginErrorModal::new(app)
            .with_header("Warp account SSO is disabled in this local-only build")
            .with_detail(
                "This fork does not connect to Warp account or SSO services. Restart Warp to continue with the local installation identity.",
            )
            .build()
            .finish()
    }
}

impl TypedActionView for NeedsSsoLinkView {
    type Action = ();
}
