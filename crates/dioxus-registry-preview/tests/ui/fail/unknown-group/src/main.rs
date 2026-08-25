enum Group {
    Fixtures,
}

dioxus_registry_preview::component_pages! {
    manifest: "../registry.json",
    group: Group {
        "fixtures" => Group::Fixtures,
    },
    default: widget,
    catalog: () {
        path: "/components",
        component: PageKind::Component,
        custom: [],
    },
}

fn main() {}
