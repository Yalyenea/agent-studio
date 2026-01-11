use gpui::App;

use crate::app::actions::SelectLocale;
use crate::app::app_menus;
use crate::AppState;

pub fn init(cx: &mut App) {
    rust_i18n::set_locale("en");

    cx.on_action(|action: &SelectLocale, cx| {
        change_locale(action.0.as_ref());
        let title = AppState::global(cx).app_title().clone();
        if !title.is_empty() {
            app_menus::init(title, cx);
        }
        cx.refresh_windows();
    });
}

pub fn change_locale(locale: &str) {
    rust_i18n::set_locale(locale);
}
