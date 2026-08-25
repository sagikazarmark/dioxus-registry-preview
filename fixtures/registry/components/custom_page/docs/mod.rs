use dioxus::prelude::*;

#[component]
fn CustomPage() -> Element {
    rsx! {
        DetailsExampleSection {}
        p { "Consumer-authored content between generated sections." }
        OverviewExampleSection {}
    }
}

fixture_docs::component! {
    group: "fixtures",
    listed: false,
    page: CustomPage,
    examples: {
        overview { description: "Declared first but rendered second." },
        details { description: "Declared second but rendered first." },
    },
}
