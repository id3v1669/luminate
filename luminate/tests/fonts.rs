//! The bundled Inter faces resolve under the family name the tokens request,
//! with the same matcher iced's text stack uses (`fontdb`).

#![cfg(feature = "bundled-font")]

use fontdb::{Database, Family, Query, Style};
use iced_luminate::theme::typography::{FAMILY, FONT, FONT_INTER, FONT_INTER_ITALIC};

fn database() -> Database {
    let mut db = Database::new();
    db.load_font_data(FONT_INTER.to_vec());
    db.load_font_data(FONT_INTER_ITALIC.to_vec());
    db
}

#[test]
fn the_family_name_resolves_to_the_upright_face() {
    let db = database();
    let id = db
        .query(&Query {
            families: &[Family::Name(FAMILY)],
            ..Query::default()
        })
        .expect("FAMILY names a bundled face");
    let face = db.face(id).expect("the id came from this database");

    assert_eq!(face.style, Style::Normal);
    assert!(
        face.families.iter().any(|(name, _)| name == FAMILY),
        "family list {:?} does not contain {FAMILY:?}",
        face.families
    );
}

#[test]
fn an_italic_query_resolves_to_the_italic_face() {
    let db = database();
    let upright = db
        .query(&Query {
            families: &[Family::Name(FAMILY)],
            ..Query::default()
        })
        .expect("upright face");
    let italic = db
        .query(&Query {
            families: &[Family::Name(FAMILY)],
            style: Style::Italic,
            ..Query::default()
        })
        .expect("italic face");

    assert_ne!(
        upright, italic,
        "the italic query must pick the second file"
    );
    assert_eq!(db.face(italic).expect("face").style, Style::Italic);
}

#[test]
fn the_default_font_requests_the_family() {
    assert_eq!(FONT.family, iced_luminate::iced::font::Family::Name(FAMILY));
    assert_eq!(FONT.style, iced_luminate::iced::font::Style::Normal);
}
