// Phase 5 Workstream D: CSV Parser for org chart imports
// Validates CSV format, role existence, circular references, email uniqueness

use csv::Reader;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use sqlx::PgPool;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrgUserRow {
    pub user_id: i32,
    pub reports_to_id: Option<i32>,
    pub name: String,
    pub role: String,
    pub email: String,
    pub department: String,
}

#[derive(Debug)]
pub enum CsvError {
    ParseError(String),
    ValidationError(String),
    CircularReference(Vec<i32>),
    InvalidRole(String),
    DuplicateEmail(String),
    DatabaseError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::ParseError(msg) => write!(f, "CSV parse error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            CsvError::CircularReference(chain) => write!(f, "Circular reference detected: {:?}", chain),
            CsvError::InvalidRole(role) => write!(f, "Invalid role '{}' (not found in profiles table)", role),
            CsvError::DuplicateEmail(email) => write!(f, "Duplicate email: {}", email),
            CsvError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for CsvError {}

pub struct CsvParser {
    pool: PgPool,
}

impl CsvParser {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Parse CSV content and validate structure
    pub fn parse_csv(&self, csv_content: &str) -> Result<Vec<OrgUserRow>, CsvError> {
        let mut reader = Reader::from_reader(csv_content.as_bytes());
        let mut rows = Vec::new();

        for result in reader.deserialize::<OrgUserRow>() {
            let row: OrgUserRow = result.map_err(|e| {
                error!(message = "csv.parse.error", error = %e);
                CsvError::ParseError(format!("Failed to parse CSV row: {}", e))
            })?;
            rows.push(row);
        }

        info!(message = "csv.parsed", row_count = rows.len());
        Ok(rows)
    }

    /// Validate all rows (roles, circular refs, email uniqueness)
    pub async fn validate_rows(&self, rows: &[OrgUserRow]) -> Result<(), CsvError> {
        // 1. Validate roles exist in profiles table
        self.validate_roles(rows).await?;

        // 2. Check for circular references
        self.detect_circular_references(rows)?;

        // 3. Check email uniqueness within CSV
        self.validate_email_uniqueness(rows)?;

        info!(message = "csv.validation.complete", row_count = rows.len());
        Ok(())
    }

    /// Check all roles exist in profiles table
    async fn validate_roles(&self, rows: &[OrgUserRow]) -> Result<(), CsvError> {
        let unique_roles: HashSet<String> = rows.iter().map(|r| r.role.clone()).collect();
        
        for role in unique_roles {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM profiles WHERE role = $1)"
            )
            .bind(&role)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                error!(message = "role.validation.error", role = %role, error = %e);
                CsvError::DatabaseError(format!("Failed to validate role: {}", e))
            })?;

            if !exists {
                warn!(message = "role.not.found", role = %role);
                return Err(CsvError::InvalidRole(role));
            }
        }

        info!(message = "roles.validated", count = rows.len());
        Ok(())
    }

    /// Detect circular references in reports_to_id chain
    fn detect_circular_references(&self, rows: &[OrgUserRow]) -> Result<(), CsvError> {
        // Build adjacency map: user_id -> reports_to_id
        let mut reports_map: HashMap<i32, Option<i32>> = HashMap::new();
        for row in rows {
            reports_map.insert(row.user_id, row.reports_to_id);
        }

        // For each user, traverse up the chain and detect cycles
        for row in rows {
            let mut visited = HashSet::new();
            let mut current = row.user_id;
            let mut chain = vec![current];

            loop {
                if visited.contains(&current) {
                    // Cycle detected
                    error!(message = "circular.reference.detected", chain = ?chain);
                    return Err(CsvError::CircularReference(chain));
                }
                visited.insert(current);

                match reports_map.get(&current) {
                    Some(Some(manager_id)) => {
                        current = *manager_id;
                        chain.push(current);
                    }
                    Some(None) => {
                        // Reached root (no manager)
                        break;
                    }
                    None => {
                        // Manager not in CSV (could be existing user in DB)
                        break;
                    }
                }
            }
        }

        info!(message = "circular.reference.check.complete");
        Ok(())
    }

    /// Validate email uniqueness within CSV
    fn validate_email_uniqueness(&self, rows: &[OrgUserRow]) -> Result<(), CsvError> {
        let mut seen_emails = HashSet::new();

        for row in rows {
            let email_lower = row.email.to_lowercase();
            if seen_emails.contains(&email_lower) {
                warn!(message = "duplicate.email", email = %email_lower);
                return Err(CsvError::DuplicateEmail(row.email.clone()));
            }
            seen_emails.insert(email_lower);
        }

        info!(message = "email.uniqueness.validated");
        Ok(())
    }

    /// Insert or update users in database
    pub async fn upsert_users(&self, rows: &[OrgUserRow]) -> Result<(i32, i32), CsvError> {
        let mut created = 0;
        let mut updated = 0;

        for row in rows {
            // Check if user exists
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM org_users WHERE user_id = $1)"
            )
            .bind(row.user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CsvError::DatabaseError(format!("Failed to check user existence: {}", e)))?;

            if exists {
                // Update existing user
                sqlx::query(
                    r#"
                    UPDATE org_users
                    SET reports_to_id = $1, name = $2, role = $3, email = $4, department = $5, updated_at = NOW()
                    WHERE user_id = $6
                    "#
                )
                .bind(row.reports_to_id)
                .bind(&row.name)
                .bind(&row.role)
                .bind(&row.email)
                .bind(&row.department)
                .bind(row.user_id)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    error!(message = "user.update.error", user_id = row.user_id, error = %e);
                    CsvError::DatabaseError(format!("Failed to update user {}: {}", row.user_id, e))
                })?;
                updated += 1;
            } else {
                // Insert new user
                sqlx::query(
                    r#"
                    INSERT INTO org_users (user_id, reports_to_id, name, role, email, department, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
                    "#
                )
                .bind(row.user_id)
                .bind(row.reports_to_id)
                .bind(&row.name)
                .bind(&row.role)
                .bind(&row.email)
                .bind(&row.department)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    error!(message = "user.insert.error", user_id = row.user_id, error = %e);
                    CsvError::DatabaseError(format!("Failed to insert user {}: {}", row.user_id, e))
                })?;
                created += 1;
            }
        }

        info!(message = "users.upserted", created = created, updated = updated);
        Ok((created, updated))
    }
}

// Tests will be added in D13
