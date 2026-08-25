use dioxus::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum ComponentGroup {
    Fixtures,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PageKind {
    Component(ComponentPage),
    ExactPath,
    Rsx,
}

impl fixture_docs::chrome::AppPage for PageKind {
    fn kind(self) -> &'static str {
        match self {
            Self::Component(_) => "generated-component",
            Self::ExactPath | Self::Rsx => "rsx",
        }
    }

    fn render(self) -> Element {
        match self {
            Self::Component(page) => page.view(),
            Self::ExactPath => rsx! { h1 { "Exact catalog path" } },
            Self::Rsx => rsx! {
                h1 { "Manual assembly" }
                default_page::ReadmeSection {}
                default_page::OverviewExampleSection {}
            },
        }
    }
}

pub(crate) type PageDescriptor = fixture_docs::PageDescriptor<ComponentGroup, PageKind>;

const MANUAL_PAGE: PageDescriptor = PageDescriptor::new(
    "manual-page",
    "/manual-page",
    "Manual assembly",
    "A handwritten fixture page.",
    fixture_docs::NavigationPlacement::Standalone("Fixture"),
    fixture_docs::ListingPolicy::Listed,
    fixture_docs::BrowserTestPolicy::Disabled,
    PageKind::Rsx,
);

const EXACT_PATH_PAGE: PageDescriptor = PageDescriptor::new(
    "exact-path",
    "/exact/a%20b/",
    "Exact path",
    "A fixture for literal catalog path routing.",
    fixture_docs::NavigationPlacement::Group(ComponentGroup::Fixtures),
    fixture_docs::ListingPolicy::Unlisted,
    fixture_docs::BrowserTestPolicy::Disabled,
    PageKind::ExactPath,
);

fixture_docs::component_pages! {
    manifest: "../../../component.json",
    group: ComponentGroup {
        "shared" => ComponentGroup::Fixtures,
        "fixtures" => ComponentGroup::Fixtures,
    },
    default: default_page,
    catalog: PageDescriptor {
        path: "",
        component: PageKind::Component,
        custom: [MANUAL_PAGE, EXACT_PATH_PAGE],
    },
}
