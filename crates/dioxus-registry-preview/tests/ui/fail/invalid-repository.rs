#[derive(Copy, Clone)]
enum Group {
    Fixtures,
}

enum PageKind {
    Component(ComponentPage),
}

type PageDescriptor = dioxus_registry_preview::PageDescriptor<Group, PageKind>;

dioxus_registry_preview::component_pages! {
    manifest: "../pass/catalog/registry.json",
    repository: "../catalog-fixture",
    group: Group {
        "shared" => Group::Fixtures,
        "fixtures" => Group::Fixtures,
    },
    default: widget,
    catalog: PageDescriptor {
        path: "/components",
        component: PageKind::Component,
        custom: [],
    },
}

fn main() {}
