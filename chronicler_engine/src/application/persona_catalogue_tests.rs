//! Tests for `PersonaCatalogue`.

use std::sync::Arc;

use crate::application::persona_catalogue::PersonaCatalogue;
use crate::test_support::fixtures::TestPersona;

#[test]
fn test_list_personas_empty() {
    let catalogue = PersonaCatalogue::new(Arc::new(
        crate::adapters::driven::storage::Storage::new_in_memory(),
    ));
    assert!(catalogue.list_personas().unwrap().is_empty());
}

#[test]
fn test_list_personas_returns_seeded_personas() {
    let storage = Arc::new(crate::adapters::driven::storage::Storage::new_in_memory());
    let persona = TestPersona::standard();
    storage.seed_persona(&persona.key, &persona).unwrap();

    let catalogue = PersonaCatalogue::new(storage);
    let personas = catalogue.list_personas().unwrap();
    assert_eq!(personas.len(), 1);
    assert_eq!(personas[0].key, "hero");
}
