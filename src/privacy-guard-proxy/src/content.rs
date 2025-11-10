/// Content-Type detection and handling for Privacy Guard Proxy
/// Task B.6: Document & Media Handling

use serde_json::Value;

/// Supported content types for PII masking
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    /// Text content (text/plain, text/html, text/markdown, etc.)
    Text,
    /// JSON content (application/json)
    Json,
    /// Image content (image/png, image/jpeg, etc.)
    Image,
    /// PDF documents (application/pdf)
    Pdf,
    /// Multipart form data (multipart/form-data)
    Multipart,
    /// Unknown or unsupported content type
    Unknown,
}

impl ContentType {
    /// Detect content type from HTTP Content-Type header
    pub fn from_header(content_type: &str) -> Self {
        let content_type_lower = content_type.to_lowercase();
        
        if content_type_lower.starts_with("text/") {
            ContentType::Text
        } else if content_type_lower.starts_with("application/json") {
            ContentType::Json
        } else if content_type_lower.starts_with("image/") {
            ContentType::Image
        } else if content_type_lower.contains("application/pdf") {
            ContentType::Pdf
        } else if content_type_lower.starts_with("multipart/form-data") {
            ContentType::Multipart
        } else {
            ContentType::Unknown
        }
    }
    
    /// Check if this content type supports PII masking
    pub fn is_maskable(&self) -> bool {
        matches!(self, ContentType::Text | ContentType::Json)
    }
    
    /// Get a human-readable name for this content type
    pub fn name(&self) -> &str {
        match self {
            ContentType::Text => "text",
            ContentType::Json => "json",
            ContentType::Image => "image",
            ContentType::Pdf => "pdf",
            ContentType::Multipart => "multipart",
            ContentType::Unknown => "unknown",
        }
    }
}

/// Recursively scan JSON for PII and collect all text fields
/// This allows us to mask PII in nested JSON structures
pub fn extract_json_text_fields(value: &Value) -> Vec<String> {
    let mut fields = Vec::new();
    extract_json_text_recursive(value, &mut fields);
    fields
}

/// Helper function to recursively extract text from JSON
fn extract_json_text_recursive(value: &Value, fields: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            fields.push(s.clone());
        }
        Value::Array(arr) => {
            for item in arr {
                extract_json_text_recursive(item, fields);
            }
        }
        Value::Object(obj) => {
            for (_key, val) in obj {
                extract_json_text_recursive(val, fields);
            }
        }
        _ => {
            // Numbers, booleans, null - skip
        }
    }
}

/// Replace text fields in JSON with masked versions
/// Uses a mapping of original → masked text
pub fn replace_json_text_fields(value: &mut Value, replacements: &[(String, String)]) {
    replace_json_text_recursive(value, replacements);
}

/// Helper function to recursively replace text in JSON
fn replace_json_text_recursive(value: &mut Value, replacements: &[(String, String)]) {
    match value {
        Value::String(s) => {
            // Check if this string matches any original text
            for (original, masked) in replacements {
                if s == original {
                    *s = masked.clone();
                    break;
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                replace_json_text_recursive(item, replacements);
            }
        }
        Value::Object(obj) => {
            for (_key, val) in obj {
                replace_json_text_recursive(val, replacements);
            }
        }
        _ => {
            // Numbers, booleans, null - skip
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_content_type_detection() {
        assert_eq!(ContentType::from_header("text/plain"), ContentType::Text);
        assert_eq!(ContentType::from_header("text/html"), ContentType::Text);
        assert_eq!(ContentType::from_header("text/markdown"), ContentType::Text);
        assert_eq!(ContentType::from_header("application/json"), ContentType::Json);
        assert_eq!(ContentType::from_header("application/json; charset=utf-8"), ContentType::Json);
        assert_eq!(ContentType::from_header("image/png"), ContentType::Image);
        assert_eq!(ContentType::from_header("image/jpeg"), ContentType::Image);
        assert_eq!(ContentType::from_header("application/pdf"), ContentType::Pdf);
        assert_eq!(ContentType::from_header("multipart/form-data"), ContentType::Multipart);
        assert_eq!(ContentType::from_header("application/octet-stream"), ContentType::Unknown);
    }
    
    #[test]
    fn test_is_maskable() {
        assert!(ContentType::Text.is_maskable());
        assert!(ContentType::Json.is_maskable());
        assert!(!ContentType::Image.is_maskable());
        assert!(!ContentType::Pdf.is_maskable());
        assert!(!ContentType::Multipart.is_maskable());
        assert!(!ContentType::Unknown.is_maskable());
    }
    
    #[test]
    fn test_extract_json_text_simple() {
        let data = json!({
            "message": "Hello John",
            "count": 42
        });
        
        let fields = extract_json_text_fields(&data);
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0], "Hello John");
    }
    
    #[test]
    fn test_extract_json_text_nested() {
        let data = json!({
            "user": {
                "name": "John Doe",
                "email": "john@example.com"
            },
            "messages": [
                "Hello",
                "World"
            ]
        });
        
        let fields = extract_json_text_fields(&data);
        assert_eq!(fields.len(), 4);
        assert!(fields.contains(&"John Doe".to_string()));
        assert!(fields.contains(&"john@example.com".to_string()));
        assert!(fields.contains(&"Hello".to_string()));
        assert!(fields.contains(&"World".to_string()));
    }
    
    #[test]
    fn test_replace_json_text() {
        let mut data = json!({
            "user": {
                "name": "John Doe",
                "ssn": "123-45-6789"
            },
            "message": "Hello world"
        });
        
        // Note: Privacy Guard returns the entire masked text, not just PII tokens
        // So we replace whole string values, not substrings
        let replacements = vec![
            ("John Doe".to_string(), "PERSON_1".to_string()),
            ("123-45-6789".to_string(), "SSN_REDACTED".to_string()),
            ("Hello world".to_string(), "Hello world".to_string()), // No PII
        ];
        
        replace_json_text_fields(&mut data, &replacements);
        
        assert_eq!(data["user"]["name"], "PERSON_1");
        assert_eq!(data["user"]["ssn"], "SSN_REDACTED");
        assert_eq!(data["message"], "Hello world");
    }
    
    #[test]
    fn test_content_type_names() {
        assert_eq!(ContentType::Text.name(), "text");
        assert_eq!(ContentType::Json.name(), "json");
        assert_eq!(ContentType::Image.name(), "image");
        assert_eq!(ContentType::Pdf.name(), "pdf");
        assert_eq!(ContentType::Multipart.name(), "multipart");
        assert_eq!(ContentType::Unknown.name(), "unknown");
    }
}
