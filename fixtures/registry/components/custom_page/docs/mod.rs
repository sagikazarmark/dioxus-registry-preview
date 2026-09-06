use dioxus::prelude::*;
use dioxus_code::{CodeOptions, Language, code_str};
use fixture_docs::chrome::CodeBlock;

#[component]
fn CustomPage() -> Element {
    rsx! {
        DetailsExampleSection {}
        p { "Consumer-authored content between generated sections." }
        OverviewExampleSection {}
        // Code without a preview: plain text always works, and a Consumer may pass its own
        // compile-time highlighted snippet from `dioxus-code`.
        CodeBlock { content: include_str!("../component.json"), label: "component.json" }
        CodeBlock {
            content: code_str!(
                "use crate::examples::custom_page::overview::Example;",
                CodeOptions::builder().with_language(Language::Rust)
            ),
            label: "Importing an Example",
        }
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
