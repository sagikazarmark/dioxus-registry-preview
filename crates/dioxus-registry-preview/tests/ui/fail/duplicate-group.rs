enum Group {
    Fixtures,
}

dioxus_registry_preview::component_pages! {
    manifest: "unused.json",
    group: Group {
        "fixtures" => Group::Fixtures,
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
