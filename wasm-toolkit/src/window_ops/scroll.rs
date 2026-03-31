use web_sys::{ScrollBehavior, ScrollToOptions};

use crate::WasmWindow;

impl WasmWindow {
    pub fn scroll_to_top(&self) {
        let options = ScrollToOptions::new();
        options.set_behavior(ScrollBehavior::Smooth);
        options.set_top(0f64);
        options.set_left(0f64);

        self.scroll_to(options);
    }

    pub fn scroll_to_custom_y(&self, top: f64, left: f64, behavior: ScrollBehavior) {
        let options = ScrollToOptions::new();
        options.set_behavior(behavior);
        options.set_top(top);
        options.set_top(left);

        self.scroll_to(options);
    }

    pub fn scroll_to(&self, options: ScrollToOptions) {
        self.inner().scroll_to_with_scroll_to_options(&options);
    }
}
