//! ModelId TDD-First Tests
//!
//! These tests define the executable specifications for ModelId type safety
//! following RED-GREEN-REFACTOR methodology

#[cfg(test)]
mod tests {
    use super::super::super::inference::ModelId;
    use std::collections::HashMap;

    #[test]
    fn test_model_id_new_creates_unique_ids() {
        // RED: This test will fail initially, demonstrating need for ModelId::new()
        let id1 = ModelId::new();
        let id2 = ModelId::new();

        assert_ne!(id1, id2, "ModelId should generate unique identifiers");
    }

    #[test]
    fn test_model_id_display_formatting() {
        // RED: This test requires Display trait implementation
        let id = ModelId::new();
        let display_str = format!("{}", id);

        assert!(!display_str.is_empty(), "ModelId should display as non-empty string");
        assert!(display_str.len() > 30, "ModelId should be UUID-length string");
    }

    #[test]
    fn test_model_id_from_string_conversion() {
        // RED: This test requires From<String> trait for ModelId
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let model_id = ModelId::from(uuid_str.to_string());

        let display_str = format!("{}", model_id);
        assert_eq!(display_str, uuid_str, "ModelId should preserve original string");
    }

    #[test]
    fn test_model_id_to_string_conversion() {
        // RED: This test requires Into<String> trait for ModelId
        let model_id = ModelId::new();
        let string_repr: String = model_id.into();

        assert!(!string_repr.is_empty(), "ModelId should convert to string");
        assert!(string_repr.len() > 30, "String representation should be UUID-length");
    }

    #[test]
    fn test_model_id_hash_map_key() {
        // RED: This test requires Hash + Eq traits for ModelId
        let mut map = HashMap::new();
        let id1 = ModelId::new();
        let id2 = ModelId::new();

        map.insert(id1, "model1".to_string());
        map.insert(id2, "model2".to_string());

        assert_eq!(map.len(), 2, "ModelId should work as HashMap key");
    }

    #[test]
    fn test_model_id_serde_serialization() {
        // RED: This test requires Serialize/Deserialize traits
        let original_id = ModelId::new();

        let json = serde_json::to_string(&original_id)
            .expect("ModelId should serialize to JSON");

        let deserialized_id: ModelId = serde_json::from_str(&json)
            .expect("ModelId should deserialize from JSON");

        assert_eq!(original_id, deserialized_id, "ModelId should round-trip through JSON");
    }

    #[test]
    fn test_model_id_clone_and_copy() {
        // RED: This test requires Clone + Copy traits
        let original = ModelId::new();
        let cloned = original.clone();
        let copied = original;

        assert_eq!(original, cloned, "ModelId should clone correctly");
        assert_eq!(original, copied, "ModelId should copy correctly");
    }

    #[test]
    fn test_model_id_debug_formatting() {
        // RED: This test requires Debug trait
        let id = ModelId::new();
        let debug_str = format!("{:?}", id);

        assert!(debug_str.contains("ModelId"), "Debug should include type name");
        assert!(!debug_str.is_empty(), "Debug should not be empty");
    }

    #[test]
    fn test_model_id_equality_and_ordering() {
        // RED: This test requires PartialEq + Eq traits
        let id1 = ModelId::from("550e8400-e29b-41d4-a716-446655440000".to_string());
        let id2 = ModelId::from("550e8400-e29b-41d4-a716-446655440000".to_string());
        let id3 = ModelId::from("550e8400-e29b-41d4-a716-446655440001".to_string());

        assert_eq!(id1, id2, "ModelId should be equal for same UUID");
        assert_ne!(id1, id3, "ModelId should be different for different UUIDs");
    }
}