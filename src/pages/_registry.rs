use resuma::prelude::*;
use resuma::FlowPageRegistry;

pub struct PagesRegistry;

impl FlowPageRegistry for PagesRegistry {
    fn routes(&self) -> &'static [(&'static str, &'static str)] {
        &[
            ("/", "index"),
            ("/lab", "lab"),
            ("/learn", "learn"),
            ("/shelf", "shelf"),
            ("/shelf/:id", "shelf_one"),
            ("/admin", "admin"),
        ]
    }

    fn layout_for(&self, _pattern: &str) -> &'static [&'static str] {
        &["/"]
    }

    fn render(&self, module: &str, req: FlowRequest) -> Option<View> {
        match module {
            "index" => Some(super::index::page(req)),
            "lab" => Some(super::lab::page(req)),
            "learn" => Some(super::learn::page(req)),
            "shelf" => Some(super::shelf::page(req)),
            "shelf_one" => Some(super::shelf::one(req)),
            "admin" => Some(super::admin::page(req)),
            _ => None,
        }
    }
}
