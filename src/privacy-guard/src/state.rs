// In-memory mapping state for pseudonym ↔ original lookups
// This module implements session-scoped state management

use dashmap::DashMap;
use std::sync::Arc;

/// Thread-safe in-memory state for pseudonym mappings
///
/// Stores bidirectional mappings:
/// - pseudonym -> original text
/// - original text -> pseudonym
///
/// Session-scoped: maps are cleared per session to avoid unbounded growth
#[derive(Clone)]
pub struct MappingState {
    /// Map: pseudonym -> original text
    forward: Arc<DashMap<String, String>>,
    /// Map: original text -> pseudonym
    reverse: Arc<DashMap<String, String>>,
}

impl MappingState {
    /// Create a new empty mapping state
    pub fn new() -> Self {
        Self {
            forward: Arc::new(DashMap::new()),
            reverse: Arc::new(DashMap::new()),
        }
    }

    /// Store a mapping: pseudonym -> original
    ///
    /// # Arguments
    /// * `pseudonym` - The pseudonym generated for the PII
    /// * `original` - The original PII text
    pub fn insert(&self, pseudonym: String, original: String) {
        self.forward.insert(pseudonym.clone(), original.clone());
        self.reverse.insert(original, pseudonym);
    }

    /// Look up the original text from a pseudonym
    ///
    /// # Arguments
    /// * `pseudonym` - The pseudonym to look up
    ///
    /// # Returns
    /// `Some(original)` if found, `None` otherwise
    pub fn get_original(&self, pseudonym: &str) -> Option<String> {
        self.forward.get(pseudonym).map(|v| v.clone())
    }

    /// Look up the pseudonym from original text
    ///
    /// # Arguments
    /// * `original` - The original PII text
    ///
    /// # Returns
    /// `Some(pseudonym)` if found, `None` otherwise
    pub fn get_pseudonym(&self, original: &str) -> Option<String> {
        self.reverse.get(original).map(|v| v.clone())
    }

    /// Check if a pseudonym exists
    pub fn contains_pseudonym(&self, pseudonym: &str) -> bool {
        self.forward.contains_key(pseudonym)
    }

    /// Check if an original text has been mapped
    pub fn contains_original(&self, original: &str) -> bool {
        self.reverse.contains_key(original)
    }

    /// Clear all mappings (session flush)
    pub fn clear(&self) {
        self.forward.clear();
        self.reverse.clear();
    }

    /// Get the number of mappings stored
    pub fn len(&self) -> usize {
        self.forward.len()
    }

    /// Check if the state is empty
    pub fn is_empty(&self) -> bool {
        self.forward.is_empty()
    }
}

impl Default for MappingState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_lookup() {
        let state = MappingState::new();

        state.insert("PERSON_abc123".to_string(), "John Doe".to_string());

        // Forward lookup: pseudonym -> original
        assert_eq!(
            state.get_original("PERSON_abc123"),
            Some("John Doe".to_string())
        );

        // Reverse lookup: original -> pseudonym
        assert_eq!(
            state.get_pseudonym("John Doe"),
            Some("PERSON_abc123".to_string())
        );
    }

    #[test]
    fn test_missing_lookup() {
        let state = MappingState::new();

        state.insert("PERSON_abc123".to_string(), "John Doe".to_string());

        // Lookup nonexistent pseudonym
        assert_eq!(state.get_original("PERSON_xyz789"), None);

        // Lookup nonexistent original
        assert_eq!(state.get_pseudonym("Jane Smith"), None);
    }

    #[test]
    fn test_contains() {
        let state = MappingState::new();

        state.insert("EMAIL_123abc".to_string(), "test@example.com".to_string());

        assert!(state.contains_pseudonym("EMAIL_123abc"));
        assert!(state.contains_original("test@example.com"));

        assert!(!state.contains_pseudonym("EMAIL_xyz789"));
        assert!(!state.contains_original("other@example.com"));
    }

    #[test]
    fn test_multiple_mappings() {
        let state = MappingState::new();

        state.insert("PERSON_111".to_string(), "Alice".to_string());
        state.insert("PERSON_222".to_string(), "Bob".to_string());
        state.insert("EMAIL_333".to_string(), "alice@test.com".to_string());

        assert_eq!(state.len(), 3);
        assert!(!state.is_empty());

        // Verify all lookups work
        assert_eq!(state.get_original("PERSON_111"), Some("Alice".to_string()));
        assert_eq!(state.get_original("PERSON_222"), Some("Bob".to_string()));
        assert_eq!(
            state.get_original("EMAIL_333"),
            Some("alice@test.com".to_string())
        );

        assert_eq!(
            state.get_pseudonym("Alice"),
            Some("PERSON_111".to_string())
        );
        assert_eq!(state.get_pseudonym("Bob"), Some("PERSON_222".to_string()));
        assert_eq!(
            state.get_pseudonym("alice@test.com"),
            Some("EMAIL_333".to_string())
        );
    }

    #[test]
    fn test_clear() {
        let state = MappingState::new();

        state.insert("PERSON_111".to_string(), "Alice".to_string());
        state.insert("PERSON_222".to_string(), "Bob".to_string());

        assert_eq!(state.len(), 2);

        state.clear();

        assert_eq!(state.len(), 0);
        assert!(state.is_empty());
        assert_eq!(state.get_original("PERSON_111"), None);
        assert_eq!(state.get_pseudonym("Alice"), None);
    }

    #[test]
    fn test_overwrite_mapping() {
        let state = MappingState::new();

        // Insert initial mapping
        state.insert("PERSON_111".to_string(), "Alice".to_string());

        assert_eq!(state.get_original("PERSON_111"), Some("Alice".to_string()));

        // Overwrite with same pseudonym, different original
        // Note: This shouldn't happen in normal operation (HMAC is deterministic)
        // but the state structure should handle it
        state.insert("PERSON_111".to_string(), "Alice Updated".to_string());

        assert_eq!(
            state.get_original("PERSON_111"),
            Some("Alice Updated".to_string())
        );
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let state = MappingState::new();

        // Clone state for multiple threads
        let state1 = state.clone();
        let state2 = state.clone();
        let state3 = state.clone();

        // Spawn threads that insert concurrently
        let handle1 = thread::spawn(move || {
            for i in 0..100 {
                state1.insert(format!("PERSON_{}", i), format!("Person {}", i));
            }
        });

        let handle2 = thread::spawn(move || {
            for i in 100..200 {
                state2.insert(format!("EMAIL_{}", i), format!("email{}@test.com", i));
            }
        });

        let handle3 = thread::spawn(move || {
            for i in 200..300 {
                state3.insert(format!("PHONE_{}", i), format!("555-{:04}", i));
            }
        });

        // Wait for all threads
        handle1.join().unwrap();
        handle2.join().unwrap();
        handle3.join().unwrap();

        // Verify all insertions succeeded
        assert_eq!(state.len(), 300);

        // Verify lookups work
        assert_eq!(state.get_original("PERSON_50"), Some("Person 50".to_string()));
        assert_eq!(
            state.get_original("EMAIL_150"),
            Some("email150@test.com".to_string())
        );
        assert_eq!(state.get_original("PHONE_250"), Some("555-0250".to_string()));
    }

    #[test]
    fn test_bidirectional_consistency() {
        let state = MappingState::new();

        let mappings = vec![
            ("SSN_aaa", "123-45-6789"),
            ("EMAIL_bbb", "test@example.com"),
            ("PHONE_ccc", "555-1234"),
        ];

        for (pseudo, orig) in &mappings {
            state.insert(pseudo.to_string(), orig.to_string());
        }

        // Verify bidirectional consistency
        for (pseudo, orig) in &mappings {
            assert_eq!(state.get_original(pseudo), Some(orig.to_string()));
            assert_eq!(state.get_pseudonym(orig), Some(pseudo.to_string()));
        }
    }

    #[test]
    fn test_empty_state() {
        let state = MappingState::new();

        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
        assert_eq!(state.get_original("anything"), None);
        assert_eq!(state.get_pseudonym("anything"), None);
    }
}

