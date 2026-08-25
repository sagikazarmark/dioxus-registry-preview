component_pages! {
    manifest: "catalog-errors/malformed.json",
    group: ComponentGroup {
        "fixtures" => ComponentGroup::Fixtures,
    },
    default: widget,
    catalog: PageDescriptor {
        path: "/components",
        component: PageKind::Component,
        custom: [],
    },
}
