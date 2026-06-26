use std::collections::HashMap;

use crate::views::{PageMeta, RenderContext, SiteLayout};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegalPageView {
    pub body_html: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LegalPageSpec {
    title: &'static str,
    description: &'static str,
    h1: &'static str,
    body: &'static str,
}

fn legal_specs() -> HashMap<&'static str, LegalPageSpec> {
    HashMap::from([
        (
            "privacy",
            LegalPageSpec {
                title: "Privacy Policy - Pornsok.com",
                description: "Privacy Policy",
                h1: "Privacy Policy - PornsOK.COM",
                body: include_str!("../../templates/legal/privacy_body.html"),
            },
        ),
        (
            "dmca",
            LegalPageSpec {
                title: "DMCA - Pornsok.com",
                description: "DMCA policy for PornsOK.com",
                h1: "DMCA",
                body: include_str!("../../templates/legal/dmca_body.html"),
            },
        ),
        (
            "terms",
            LegalPageSpec {
                title: "Terms - Pornsok.com",
                description: "Terms of use for PornsOK.com",
                h1: "Terms",
                body: include_str!("../../templates/legal/terms_body.html"),
            },
        ),
        (
            "2557",
            LegalPageSpec {
                title: "18 U.S.C. 2257 - Pornsok.com",
                description: "18 U.S.C. 2257 compliance statement",
                h1: "18 U.S.C. 2257",
                body: include_str!("../../templates/legal/2557_body.html"),
            },
        ),
        (
            "contact",
            LegalPageSpec {
                title: "Contact - Pornsok.com",
                description: "Contact PornsOK.com",
                h1: "Contact",
                body: include_str!("../../templates/legal/contact_body.html"),
            },
        ),
    ])
}

pub fn legal_page_view(slug: &str) -> Option<LegalPageView> {
    legal_specs().get(slug).map(|spec| LegalPageView {
        body_html: spec.body.to_string(),
    })
}

pub fn legal_static_context(layout: SiteLayout, slug: &str) -> Option<RenderContext> {
    let specs = legal_specs();
    let spec = specs.get(slug)?;
    let site = layout.site_base_url.clone();
    let canonical_path = format!("/page/{slug}.html");
    let page = PageMeta {
        title: spec.title.into(),
        description: spec.description.into(),
        canonical_path: canonical_path.clone(),
        h1: spec.h1.into(),
        rel_prev_href: None,
        rel_next_href: None,
        og_title: spec.title.into(),
        og_description: spec.description.into(),
        og_url: crate::models::pagination::absolute_url(&site, &canonical_path),
    };
    Some(RenderContext::new(layout, page))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::SiteLayout;

    #[test]
    fn privacy_meta_matches_live_inventory() {
        let layout = SiteLayout::production();
        let ctx = legal_static_context(layout, "privacy").expect("privacy page");
        assert_eq!(ctx.page.title, "Privacy Policy - Pornsok.com");
        assert_eq!(ctx.page.h1, "Privacy Policy - PornsOK.COM");
        assert_eq!(
            ctx.page.canonical_href(&ctx.layout.site_base_url),
            "https://pornsok.com/page/privacy.html"
        );
    }

    #[test]
    fn all_footer_legal_slugs_have_views() {
        for slug in ["privacy", "dmca", "terms", "2557", "contact"] {
            assert!(legal_page_view(slug).is_some(), "missing view for {slug}");
            assert!(
                legal_static_context(SiteLayout::production(), slug).is_some(),
                "missing context for {slug}"
            );
        }
    }
}
