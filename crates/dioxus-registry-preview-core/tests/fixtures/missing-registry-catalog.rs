component_pages! {
    manifest: "does-not-exist.json",
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
