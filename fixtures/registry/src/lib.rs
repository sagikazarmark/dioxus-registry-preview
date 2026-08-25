use dioxus::prelude::*;

mod custom_home;
mod pages;

pub(crate) mod example {
    pub(crate) use fixture_docs::chrome::{ExampleSection, ReadmeSection};
}

pub(crate) use pages::example_modules as examples;

#[component]
#[allow(non_snake_case)]
pub fn Fixture() -> Element {
    rsx! {
        fixture_docs::chrome::App { catalog: pages::APP_CATALOG }
    }
}

#[component]
#[allow(non_snake_case)]
pub fn CustomHomeFixture() -> Element {
    rsx! {
        fixture_docs::chrome::App {
            catalog: custom_home::CATALOG,
            title: "Custom Home Registry",
        }
    }
}
