/*!
# Serialization Tests

Comprehensive tests for the Vela serialization system.
Tests @serializable, @serialize, @ignore, and @custom decorators.
*/

use crate::serialization_decorators::{
    FieldConfig, SerializationCodeGenerator, SerializationDecoratorProcessor, SerializableClass
};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serializable_class_creation() {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldConfig::Include { serialized_name: "id".to_string() });
        fields.insert("name".to_string(), FieldConfig::Include { serialized_name: "name".to_string() });
        fields.insert("password".to_string(), FieldConfig::Ignore);

        let class = SerializableClass {
            name: "User".to_string(),
            fields,
            custom_serializer: None,
        };

        assert_eq!(class.name, "User");
        assert_eq!(class.fields.len(), 3);
    }

    #[test]
    fn test_to_json_generation() {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldConfig::Include { serialized_name: "user_id".to_string() });
        fields.insert("name".to_string(), FieldConfig::Include { serialized_name: "full_name".to_string() });
        fields.insert("password".to_string(), FieldConfig::Ignore);

        let class = SerializableClass {
            name: "User".to_string(),
            fields,
            custom_serializer: None,
        };

        let generator = SerializationCodeGenerator;
        let code = generator.generate_to_json(&class);

        // Check that generated code includes the expected structure
        assert!(code.contains("fn toJson(self) -> String"));
        assert!(code.contains("\"user_id\""));
        assert!(code.contains("\"full_name\""));
        assert!(!code.contains("password")); // Ignored field should not be included
    }

    #[test]
    fn test_from_json_generation() {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldConfig::Include { serialized_name: "id".to_string() });

        let class = SerializableClass {
            name: "User".to_string(),
            fields,
            custom_serializer: None,
        };

        let generator = SerializationCodeGenerator;
        let code = generator.generate_from_json(&class);

        assert!(code.contains("fn fromJson(json: String) -> Result<User, Error>"));
    }

    #[test]
    fn test_custom_serializer_field() {
        let mut fields = HashMap::new();
        fields.insert("birthDate".to_string(), FieldConfig::Custom { serializer: "DateSerializer".to_string() });

        let class = SerializableClass {
            name: "Person".to_string(),
            fields,
            custom_serializer: None,
        };

        let generator = SerializationCodeGenerator;
        let code = generator.generate_to_json(&class);

        assert!(code.contains("DateSerializer::serialize(self.birthDate)"));
    }

    #[test]
    fn test_ignore_field_not_in_json() {
        let mut fields = HashMap::new();
        fields.insert("public".to_string(), FieldConfig::Include { serialized_name: "public".to_string() });
        fields.insert("secret".to_string(), FieldConfig::Ignore);

        let class = SerializableClass {
            name: "Data".to_string(),
            fields,
            custom_serializer: None,
        };

        let generator = SerializationCodeGenerator;
        let code = generator.generate_to_json(&class);

        assert!(code.contains("\"public\""));
        assert!(!code.contains("secret"));
    }

    #[test]
    fn test_field_name_mapping() {
        let mut fields = HashMap::new();
        fields.insert("userId".to_string(), FieldConfig::Include { serialized_name: "user_id".to_string() });
        fields.insert("emailAddress".to_string(), FieldConfig::Include { serialized_name: "email".to_string() });

        let class = SerializableClass {
            name: "User".to_string(),
            fields,
            custom_serializer: None,
        };

        let generator = SerializationCodeGenerator;
        let code = generator.generate_to_json(&class);

        // Check that serialized names appear in JSON keys
        assert!(code.contains("\"user_id\""));
        assert!(code.contains("\"email\""));
        // Check that original field names appear in property access
        assert!(code.contains("self.userId.toJson()"));
        assert!(code.contains("self.emailAddress.toJson()"));
    }

    #[test]
    fn test_empty_serializable_class() {
        let fields = HashMap::new();

        let class = SerializableClass {
            name: "Empty".to_string(),
            fields,
            custom_serializer: None,
        };

        let generator = SerializationCodeGenerator;
        let code = generator.generate_to_json(&class);

        assert!(code.contains("return \"{}\";"));
    }

    #[test]
    fn test_multiple_custom_serializers() {
        let mut fields = HashMap::new();
        fields.insert("date".to_string(), FieldConfig::Custom { serializer: "DateSerializer".to_string() });
        fields.insert("address".to_string(), FieldConfig::Custom { serializer: "AddressSerializer".to_string() });

        let class = SerializableClass {
            name: "Person".to_string(),
            fields,
            custom_serializer: None,
        };

        let generator = SerializationCodeGenerator;
        let code = generator.generate_to_json(&class);

        assert!(code.contains("DateSerializer::serialize(self.date)"));
        assert!(code.contains("AddressSerializer::serialize(self.address)"));
    }

    #[test]
    fn test_mixed_field_types() {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), FieldConfig::Include { serialized_name: "id".to_string() });
        fields.insert("password".to_string(), FieldConfig::Ignore);
        fields.insert("createdAt".to_string(), FieldConfig::Custom { serializer: "DateTimeSerializer".to_string() });

        let class = SerializableClass {
            name: "User".to_string(),
            fields,
            custom_serializer: None,
        };

        let generator = SerializationCodeGenerator;
        let code = generator.generate_to_json(&class);

        assert!(code.contains("\"id\""));
        assert!(!code.contains("password"));
        assert!(code.contains("DateTimeSerializer::serialize(self.createdAt)"));
    }
}