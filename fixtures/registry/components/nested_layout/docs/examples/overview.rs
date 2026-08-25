use dioxus::prelude::*;

use crate::examples::default_page::overview::Example as DefaultExample;

#[component]
pub fn Example() -> Element {
    rsx! {
        div { "Nested layout Example" }
        DefaultExample {}
    }
}
