use dioxus::prelude::*;

mod example {
    use dioxus::prelude::*;
    use dioxus_registry_preview::ExampleDocumentation;

    #[component]
    pub fn ExampleSection(documentation: ExampleDocumentation, children: Element) -> Element {
        rsx! {
            section { "data-example": documentation.slug, {children} }
        }
    }

    #[component]
    pub fn ReadmeSection(html: &'static str) -> Element {
        rsx! { section { dangerous_inner_html: html } }
    }
}

dioxus_registry_preview::component! {
    group: "fixtures",
    render_readme: true,
    examples: {
        overview { description: "The default example." },
        content_appearance { description: "A multiword generated section name." },
    },
}

fn main() {
    let _: Element = rsx! {
        ReadmeSection {}
        OverviewExampleSection {}
        ContentAppearanceExampleSection {}
        DocumentationPage {}
    };
}
