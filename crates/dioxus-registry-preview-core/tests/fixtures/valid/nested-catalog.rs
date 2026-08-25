mod pages {
    fixture_docs::component_pages! {
        manifest: "component.json",
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
}
