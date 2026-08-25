fixture_docs::component_pages! {
    manifest: "component.json",
    group: ComponentGroup {
        "shared" => ComponentGroup::Fixtures,
        "fixtures" => ComponentGroup::Fixtures,
    },
    default: widget,
    catalog: PageDescriptor {
        path: "/components",
        component: PageKind::Component,
        custom: [],
    },
}
