use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum CustomHomeGroup {}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum CustomHomePage {
    Home,
}

impl fixture_docs::chrome::AppPage for CustomHomePage {
    fn kind(self) -> &'static str {
        "rsx"
    }

    fn render(self) -> Element {
        match self {
            Self::Home => rsx! {
                h1 { "Consumer-owned home" }
                p { "This authored root page replaces the default home." }
            },
        }
    }
}

type CustomHomeDescriptor = fixture_docs::PageDescriptor<CustomHomeGroup, CustomHomePage>;

const PAGES: &[CustomHomeDescriptor] = &[CustomHomeDescriptor::new(
    "custom-home",
    "/",
    "Custom home",
    "A Consumer-owned Registry home.",
    fixture_docs::NavigationPlacement::Standalone("Fixture"),
    fixture_docs::ListingPolicy::Unlisted,
    fixture_docs::BrowserTestPolicy::Enabled,
    CustomHomePage::Home,
)];

fn group_id(_: &CustomHomeDescriptor) -> &'static str {
    unreachable!("the custom-home fixture has no grouped pages")
}

pub(crate) const CATALOG: fixture_docs::chrome::AppCatalog<CustomHomeDescriptor> =
    fixture_docs::chrome::AppCatalog::new(PAGES, group_id);
