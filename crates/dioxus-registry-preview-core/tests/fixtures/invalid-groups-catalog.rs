component_pages! {
    manifest: "valid/component.json",
    group: ComponentGroup {
        "Bad" => ComponentGroup::Fixtures,
        "Bad" => ComponentGroup::Fixtures,
    },
    default: widget,
    catalog: PageDescriptor {
        path: "/components",
        component: PageKind::Component,
        custom: [],
    },
}
