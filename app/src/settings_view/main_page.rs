use super::{
    flags,
    settings_page::{
        render_body_item, render_customer_type_badge, AdditionalInfo, LocalOnlyIconState,
        MatchData, PageType, SettingsPageMeta, SettingsPageViewHandle, SettingsWidget, ToggleState,
        HEADER_PADDING,
    },
    SettingsAction, SettingsSection, ToggleSettingActionPair,
};
use crate::auth::AuthStateProvider;
use crate::autoupdate::{self, AutoupdateStage, AutoupdateState};
use crate::send_telemetry_from_ctx;
use crate::{
    appearance::Appearance, auth::auth_state::AuthState, report_if_error,
    settings::cloud_preferences::CloudPreferencesSettings, TelemetryEvent,
};
use crate::{workspace::WorkspaceAction, workspaces::update_manager::TeamUpdateManager};
use ::settings::{Setting, ToggleableSetting};
use lazy_static::lazy_static;
use pathfinder_color::ColorU;
use std::sync::{Arc, Mutex};
use warp_core::features::FeatureFlag;
use warp_core::{channel::ChannelState, context_flag::ContextFlag};
use warpui::keymap::ContextPredicate;
use warpui::{
    assets::asset_cache::AssetSource,
    elements::{Border, Empty, MainAxisAlignment, MainAxisSize},
    id,
    platform::Cursor,
    ui_components::switch::SwitchStateHandle,
};
use warpui::{
    elements::{
        Align, ConstrainedBox, Container, CornerRadius, CrossAxisAlignment, Element, Flex,
        MouseStateHandle, ParentElement, Radius, Shrinkable, Text,
    },
    Action, AppContext,
};
use warpui::{
    elements::{CacheOption, Image},
    ui_components::components::{UiComponent, UiComponentStyles},
};
use warpui::{
    Entity, ModelHandle, SingletonEntity, TypedActionView, View, ViewContext, ViewHandle,
};

const PHOTO_SIZE: f32 = 40.;
const REGULAR_TEXT_FONT_SIZE: f32 = 12.;
const VERTICAL_MARGIN: f32 = 24.;
lazy_static! {
    static ref SETTINGS_SYNC_BINDINGS_ADDED: Arc<Mutex<bool>> = Default::default();
}

pub fn init_actions_from_parent_view<T: Action + Clone>(
    app: &mut AppContext,
    context: &ContextPredicate,
    builder: fn(SettingsAction) -> T,
) {
    let mut toggle_binding_pairs = Vec::new();
    maybe_add_settings_sync_toggle_binding(app, context, builder, &mut toggle_binding_pairs);

    // Add other bindings here in the future.

    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(toggle_binding_pairs, app);
}

fn maybe_add_settings_sync_toggle_binding<T: Action + Clone>(
    app: &mut AppContext,
    context: &ContextPredicate,
    builder: fn(SettingsAction) -> T,
    toggle_binding_pairs: &mut Vec<ToggleSettingActionPair<T>>,
) {
    let mut lock = SETTINGS_SYNC_BINDINGS_ADDED
        .lock()
        .expect("settings sync bindings lock poisoned");
    if !*lock {
        *lock = true;
        toggle_binding_pairs.push(
            ToggleSettingActionPair::new(
                "settings sync",
                builder(SettingsAction::MainPageToggle(
                    MainPageAction::ToggleSettingsSync,
                )),
                context,
                flags::SETTINGS_SYNC_FLAG,
            )
            .is_supported_on_current_platform(
                CloudPreferencesSettings::as_ref(app)
                    .settings_sync_enabled
                    .is_supported_on_current_platform(),
            ),
        );
    }
}

pub fn handle_experiment_change(app: &mut AppContext) {
    let mut toggle_binding_pairs: Vec<ToggleSettingActionPair<WorkspaceAction>> = Vec::new();
    maybe_add_settings_sync_toggle_binding(
        app,
        &id!("Workspace"),
        WorkspaceAction::DispatchToSettingsTab,
        &mut toggle_binding_pairs,
    );
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(toggle_binding_pairs, app);
}

#[derive(Debug, Clone)]
pub enum MainPageAction {
    Relaunch,
    DownloadUpdate,
    CheckForUpdate,
    ToggleSettingsSync,
    OpenUrl(String),
}

#[derive(Clone, Copy)]
pub enum MainSettingsPageEvent {
    CheckForUpdate,
    #[allow(dead_code)]
    OpenWarpDrive,
}

pub struct MainSettingsPageView {
    page: PageType<Self>,
    auth_state: Arc<AuthState>,
}

impl Entity for MainSettingsPageView {
    type Event = MainSettingsPageEvent;
}

impl TypedActionView for MainSettingsPageView {
    type Action = MainPageAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            MainPageAction::Relaunch => {
                autoupdate::initiate_relaunch_for_update(ctx);
            }
            MainPageAction::DownloadUpdate => {
                autoupdate::manually_download_new_version(ctx);
            }
            MainPageAction::CheckForUpdate => {
                ctx.emit(MainSettingsPageEvent::CheckForUpdate);
                ctx.notify();
            }
            MainPageAction::ToggleSettingsSync => {
                let new_value =
                    CloudPreferencesSettings::handle(ctx).update(ctx, |prefs_settings, ctx| {
                        report_if_error!(prefs_settings
                            .settings_sync_enabled
                            .toggle_and_save_value(ctx));
                        *prefs_settings.settings_sync_enabled
                    });
                send_telemetry_from_ctx!(
                    TelemetryEvent::ToggleSettingsSync {
                        is_settings_sync_enabled: new_value,
                    },
                    ctx
                );
                ctx.notify();
            }
            MainPageAction::OpenUrl(url) => {
                ctx.open_url(url);
            }
        }
    }
}

impl View for MainSettingsPageView {
    fn ui_name() -> &'static str {
        "MainSettingsPage"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        self.page.render(self, app)
    }
}

impl MainSettingsPageView {
    pub fn new(ctx: &mut ViewContext<MainSettingsPageView>) -> Self {
        let auth_state = AuthStateProvider::as_ref(ctx).get().clone();

        let autoupdate_state_handle = AutoupdateState::handle(ctx);
        ctx.observe(
            &autoupdate_state_handle,
            Self::handle_autoupdate_state_change,
        );

        ctx.subscribe_to_model(&CloudPreferencesSettings::handle(ctx), |_, _, _, ctx| {
            ctx.notify();
        });

        let mut widgets: Vec<Box<dyn SettingsWidget<View = Self>>> = vec![
            Box::new(AccountWidget::default()),
            Box::new(DividerWidget {}),
        ];

        widgets.push(Box::new(SettingsSyncWidget::default()));

        if ChannelState::app_version().is_some() {
            widgets.push(Box::new(VersionInfoWidget::default()));
        }

        let page = PageType::new_uncategorized(widgets, Some("Account"));

        MainSettingsPageView { page, auth_state }
    }

    fn handle_autoupdate_state_change(
        &mut self,
        _: ModelHandle<AutoupdateState>,
        ctx: &mut ViewContext<Self>,
    ) {
        ctx.notify();
    }
}

#[derive(Default)]
struct AccountWidget;

impl AccountWidget {
    fn render_anonymous_account_info(
        &self,
        auth_state: &AuthState,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let user_info = Text::new_inline("Local-only identity", appearance.ui_font_family(), 16.)
            .with_color(appearance.theme().active_ui_text_color().into())
            .finish();

        let mut plan_info = Flex::column()
            .with_main_axis_alignment(MainAxisAlignment::SpaceEvenly)
            .with_cross_axis_alignment(CrossAxisAlignment::End);

        plan_info.add_child(render_customer_type_badge(appearance, "Local".into()));

        if let Some(user_id) = auth_state.user_id() {
            plan_info.add_child(
                Text::new_inline(
                    user_id.to_string(),
                    appearance.ui_font_family(),
                    REGULAR_TEXT_FONT_SIZE,
                )
                .with_color(
                    appearance
                        .theme()
                        .active_ui_text_color()
                        .with_opacity(60)
                        .into(),
                )
                .finish(),
            );
        }

        Flex::row()
            .with_child(
                Shrinkable::new(
                    1.0,
                    Flex::row()
                        .with_child(user_info)
                        .with_main_axis_alignment(MainAxisAlignment::Start)
                        .with_main_axis_size(MainAxisSize::Max)
                        .finish(),
                )
                .finish(),
            )
            .with_child(Align::new(plan_info.finish()).right().finish())
            .with_cross_axis_alignment(CrossAxisAlignment::Start)
            .finish()
    }

    fn render_account_info(
        &self,
        profile_image_source: Option<&AssetSource>,
        auth_state: &AuthState,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let mut user_info = Flex::row().with_cross_axis_alignment(CrossAxisAlignment::Center);
        if let Some(profile_image_source) = profile_image_source {
            // Only continue if profile_image_source is a source with a non empty url/path
            if matches!(profile_image_source, AssetSource::Async { ref id, .. } if !id.key().is_empty())
                || matches!(profile_image_source, AssetSource::Bundled { path, .. } if !path.is_empty())
                || matches!(profile_image_source, AssetSource::LocalFile { path, .. } if !path.is_empty())
            {
                let photo = Image::new(profile_image_source.clone(), CacheOption::BySize)
                    .with_corner_radius(CornerRadius::with_all(Radius::Percentage(50.)));
                user_info.add_child(
                    Container::new(
                        ConstrainedBox::new(photo.finish())
                            .with_height(PHOTO_SIZE)
                            .with_width(PHOTO_SIZE)
                            .finish(),
                    )
                    .with_margin_right(HEADER_PADDING)
                    .finish(),
                );
            }
        }

        let display_name = auth_state.username_for_display().map(|screen_name| {
            let email = auth_state.user_email();
            match email {
                Some(email) => {
                    if !screen_name.is_empty() && screen_name != email {
                        Flex::column()
                            .with_main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                            .with_cross_axis_alignment(CrossAxisAlignment::Start)
                            .with_child(
                                Text::new_inline(screen_name, appearance.ui_font_family(), 16.)
                                    .with_color(appearance.theme().active_ui_text_color().into())
                                    .finish(),
                            )
                            .with_child(
                                appearance
                                    .ui_builder()
                                    .paragraph(email)
                                    .with_style(UiComponentStyles {
                                        font_color: Some(
                                            appearance
                                                .theme()
                                                .active_ui_text_color()
                                                .with_opacity(60)
                                                .into(),
                                        ),
                                        font_size: Some(REGULAR_TEXT_FONT_SIZE),
                                        ..Default::default()
                                    })
                                    .build()
                                    .finish(),
                            )
                            .finish()
                    } else {
                        Text::new_inline(email, appearance.ui_font_family(), 16.)
                            .with_color(appearance.theme().active_ui_text_color().into())
                            .finish()
                    }
                }
                _ => Text::new_inline(screen_name, appearance.ui_font_family(), 16.)
                    .with_color(appearance.theme().active_ui_text_color().into())
                    .finish(),
            }
        });

        if let Some(display_name) = display_name {
            user_info.add_child(display_name);
        }

        let mut plan_info = Flex::column()
            .with_main_axis_alignment(MainAxisAlignment::SpaceEvenly)
            .with_cross_axis_alignment(CrossAxisAlignment::End);
        plan_info.add_child(render_customer_type_badge(appearance, "Local".into()));

        if let Some(user_id) = auth_state.user_id() {
            plan_info.add_child(
                Text::new_inline(
                    user_id.to_string(),
                    appearance.ui_font_family(),
                    REGULAR_TEXT_FONT_SIZE,
                )
                .with_color(
                    appearance
                        .theme()
                        .active_ui_text_color()
                        .with_opacity(60)
                        .into(),
                )
                .finish(),
            );
        }

        let mut row = Flex::row()
            .with_child(
                Shrinkable::new(1.0, Align::new(user_info.finish()).left().finish()).finish(),
            )
            .with_cross_axis_alignment(CrossAxisAlignment::Start);

        if !FeatureFlag::UsageBasedPricing.is_enabled() {
            row.add_child(Align::new(plan_info.finish()).right().finish());
        }

        row.finish()
    }
}

impl SettingsWidget for AccountWidget {
    type View = MainSettingsPageView;

    fn search_terms(&self) -> &str {
        "account local identity"
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        _app: &AppContext,
    ) -> Box<dyn Element> {
        let account_info = if view.auth_state.is_anonymous_or_logged_out() {
            self.render_anonymous_account_info(view.auth_state.as_ref(), appearance)
        } else {
            let profile_image_source = view.auth_state.user_photo_url().map(|url| {
                asset_cache::url_source_with_persistence(url, &warp_core::paths::cache_dir())
            });
            self.render_account_info(
                profile_image_source.as_ref(),
                view.auth_state.as_ref(),
                appearance,
            )
        };

        Flex::column()
            .with_child(
                Container::new(account_info)
                    .with_margin_top(VERTICAL_MARGIN)
                    .finish(),
            )
            .finish()
    }
}

struct DividerWidget {}

impl SettingsWidget for DividerWidget {
    type View = MainSettingsPageView;

    fn search_terms(&self) -> &str {
        ""
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        _app: &AppContext,
    ) -> Box<dyn Element> {
        Container::new(
            Container::new(Empty::new().finish())
                .with_border(Border::bottom(1.).with_border_fill(appearance.theme().outline()))
                .finish(),
        )
        .with_margin_top(VERTICAL_MARGIN)
        .finish()
    }
}

#[derive(Default)]
struct SettingsSyncWidget {
    tooltip_state: MouseStateHandle,
    switch_state: SwitchStateHandle,
}

impl SettingsWidget for SettingsSyncWidget {
    type View = MainSettingsPageView;

    fn search_terms(&self) -> &str {
        "settings sync"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        !AuthStateProvider::as_ref(app)
            .get()
            .is_anonymous_or_logged_out()
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let preferences_settings = CloudPreferencesSettings::as_ref(app);

        let label_info = AdditionalInfo {
            mouse_state: self.tooltip_state.clone(),
            on_click_action: Some(MainPageAction::OpenUrl(
                "https://docs.warp.dev/terminal/more-features/settings-sync".into(),
            )),
            secondary_text: None,
            tooltip_override_text: None,
        };

        Container::new(render_body_item::<MainPageAction>(
            "Settings sync".to_string(),
            Some(label_info),
            // Cloud prefs are always synced, so no need to show the local-only icon.
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
            appearance
                .ui_builder()
                .switch(self.switch_state.clone())
                .check(*preferences_settings.settings_sync_enabled.value())
                .build()
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(MainPageAction::ToggleSettingsSync)
                })
                .finish(),
            None,
        ))
        .with_margin_top(VERTICAL_MARGIN)
        .finish()
    }
}

#[derive(Default)]
struct VersionInfoWidget {
    copy_version_button_mouse_state: MouseStateHandle,
    version_info_cta_link_mouse_state: MouseStateHandle,
}

impl VersionInfoWidget {
    fn render_version_info(
        &self,
        version: &'static str,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let faded_text_color = appearance
            .theme()
            .active_ui_text_color()
            .with_opacity(60)
            .into();
        struct StatusContent {
            text: &'static str,
            color: ColorU,
        }
        struct CallToActionContent {
            text: &'static str,
            action: MainPageAction,
        }

        let (status_content, call_to_action_content) =
            if ContextFlag::PromptForVersionUpdates.is_enabled() {
                let ansi_red: ColorU = appearance.theme().terminal_colors().bright.red.into();
                match autoupdate::get_update_state(app) {
                    AutoupdateStage::NoUpdateAvailable => (
                        Some(StatusContent {
                            text: "Up to date",
                            color: faded_text_color,
                        }),
                        Some(CallToActionContent {
                            text: "Check for updates",
                            action: MainPageAction::CheckForUpdate,
                        }),
                    ),
                    AutoupdateStage::CheckingForUpdate => (
                        Some(StatusContent {
                            text: "checking for update...",
                            color: faded_text_color,
                        }),
                        None,
                    ),
                    AutoupdateStage::DownloadingUpdate => (
                        Some(StatusContent {
                            text: "downloading update...",
                            color: faded_text_color,
                        }),
                        None,
                    ),
                    AutoupdateStage::UpdateReady { .. } => (
                        Some(StatusContent {
                            text: "Update available",
                            color: ansi_red,
                        }),
                        Some(CallToActionContent {
                            text: "Relaunch Warp",
                            action: MainPageAction::Relaunch,
                        }),
                    ),
                    AutoupdateStage::Updating { .. } => (
                        Some(StatusContent {
                            text: "Updating...",
                            color: faded_text_color,
                        }),
                        None,
                    ),
                    AutoupdateStage::UpdatedPendingRestart { .. } => (
                        Some(StatusContent {
                            text: "Installed update",
                            color: faded_text_color,
                        }),
                        Some(CallToActionContent {
                            text: "Relaunch Warp",
                            action: MainPageAction::Relaunch,
                        }),
                    ),
                    AutoupdateStage::UnableToUpdateToNewVersion { .. } => (
                        Some(StatusContent {
                            text: "A new version of Warp is available but can't be installed",
                            color: ansi_red,
                        }),
                        Some(CallToActionContent {
                            text: "Update Warp manually",
                            // note: the handler for this action is a no-op
                            action: MainPageAction::DownloadUpdate,
                        }),
                    ),
                    AutoupdateStage::UnableToLaunchNewVersion { .. } => (
                        Some(StatusContent {
                            text: "A new version of Warp is installed but can't be launched.",
                            color: ansi_red,
                        }),
                        Some(CallToActionContent {
                            text: "Update Warp manually",
                            // note: the handler for this action is a no-op
                            action: MainPageAction::DownloadUpdate,
                        }),
                    ),
                }
            } else {
                (None, None)
            };

        let mut first_row = Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(
                Shrinkable::new(
                    1.0,
                    Align::new(
                        Text::new_inline(
                            "Version".to_string(),
                            appearance.ui_font_family(),
                            REGULAR_TEXT_FONT_SIZE,
                        )
                        .with_color(faded_text_color)
                        .finish(),
                    )
                    .left()
                    .finish(),
                )
                .finish(),
            );
        if let Some(call_to_action_content) = call_to_action_content {
            first_row.add_child(
                appearance
                    .ui_builder()
                    .link(
                        call_to_action_content.text.into(),
                        None,
                        Some(Box::new(move |ctx| {
                            ctx.dispatch_typed_action(call_to_action_content.action.clone());
                        })),
                        self.version_info_cta_link_mouse_state.clone(),
                    )
                    .soft_wrap(false)
                    .build()
                    .finish(),
            );
        }

        let mut second_row = Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(
                Shrinkable::new(
                    1.0,
                    Align::new(
                        Flex::row()
                            .with_cross_axis_alignment(CrossAxisAlignment::Start)
                            .with_child(
                                appearance
                                    .ui_builder()
                                    .copy_button(16., self.copy_version_button_mouse_state.clone())
                                    .build()
                                    .with_cursor(Cursor::PointingHand)
                                    .on_click(move |ctx, _, _| {
                                        ctx.dispatch_typed_action(WorkspaceAction::CopyVersion(
                                            version,
                                        ));
                                    })
                                    .finish(),
                            )
                            .with_child(
                                Container::new(
                                    Text::new_inline(
                                        version.to_string(),
                                        appearance.ui_font_family(),
                                        REGULAR_TEXT_FONT_SIZE,
                                    )
                                    .with_color(appearance.theme().active_ui_text_color().into())
                                    .finish(),
                                )
                                .with_margin_left(8.)
                                .finish(),
                            )
                            .finish(),
                    )
                    .left()
                    .finish(),
                )
                .finish(),
            );
        if let Some(status_content) = status_content {
            second_row.add_child(
                Text::new_inline(
                    status_content.text.to_string(),
                    appearance.ui_font_family(),
                    REGULAR_TEXT_FONT_SIZE,
                )
                .with_color(status_content.color)
                .finish(),
            );
        }

        let mut version_info = Flex::column();
        version_info.add_child(first_row.finish());
        version_info.add_child(
            Container::new(second_row.finish())
                .with_margin_top(5.)
                .finish(),
        );
        version_info.finish()
    }
}

impl SettingsWidget for VersionInfoWidget {
    type View = MainSettingsPageView;

    fn search_terms(&self) -> &str {
        "version update"
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        if let Some(version) = ChannelState::app_version() {
            Container::new(self.render_version_info(version, appearance, app))
                .with_margin_top(VERTICAL_MARGIN)
                .finish()
        } else {
            log::error!("Shouldn't render VersionInfoWidget without GIT_RELEASE_TAG");
            Empty::new().finish()
        }
    }
}

impl SettingsPageMeta for MainSettingsPageView {
    fn section() -> SettingsSection {
        SettingsSection::Account
    }

    fn should_render(&self, _ctx: &AppContext) -> bool {
        true
    }

    fn on_page_selected(&mut self, _: bool, ctx: &mut ViewContext<Self>) {
        // We want to immediately see if the user is part of a workspace rather than wait for the next poll.
        std::mem::drop(
            TeamUpdateManager::handle(ctx)
                .update(ctx, |manager, ctx| manager.refresh_workspace_metadata(ctx)),
        );
    }

    fn update_filter(&mut self, query: &str, ctx: &mut ViewContext<Self>) -> MatchData {
        self.page.update_filter(query, ctx)
    }

    fn scroll_to_widget(&mut self, widget_id: &'static str) {
        self.page.scroll_to_widget(widget_id)
    }

    fn clear_highlighted_widget(&mut self) {
        self.page.clear_highlighted_widget();
    }
}

impl From<ViewHandle<MainSettingsPageView>> for SettingsPageViewHandle {
    fn from(view_handle: ViewHandle<MainSettingsPageView>) -> Self {
        SettingsPageViewHandle::Main(view_handle)
    }
}
