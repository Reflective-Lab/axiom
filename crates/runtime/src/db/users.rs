//! User repository

use chrono::{DateTime, Utc};
use firestore::*;
use serde::{Deserialize, Serialize};

use crate::gcp::FirestoreError;

/// Generate a unique document ID (timestamp + random suffix)
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let random: u32 = rand::random();
    format!("{timestamp:x}{random:08x}")
}

/// User document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID (Firebase Auth UID or custom ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Email address
    pub email: String,

    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Profile photo URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,

    /// Stripe customer ID (for billing integration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stripe_customer_id: Option<String>,

    /// Account status
    #[serde(default)]
    pub status: UserStatus,

    /// User role
    #[serde(default)]
    pub role: UserRole,

    /// API usage quota (jobs per day)
    #[serde(default = "default_quota")]
    pub quota: u32,

    /// Jobs created today
    #[serde(default)]
    pub jobs_today: u32,

    /// Last quota reset date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_reset_at: Option<DateTime<Utc>>,

    /// Account creation time
    pub created_at: DateTime<Utc>,

    /// Last update time
    pub updated_at: DateTime<Utc>,

    /// Last login time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_at: Option<DateTime<Utc>>,
}

fn default_quota() -> u32 {
    100
}

/// User account status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    #[default]
    Active,
    Suspended,
    Deleted,
}

/// User role
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    #[default]
    User,
    Admin,
    Service,
}

impl User {
    /// Create a new user with email
    pub fn new(email: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            email: email.into(),
            display_name: None,
            photo_url: None,
            stripe_customer_id: None,
            status: UserStatus::Active,
            role: UserRole::User,
            quota: default_quota(),
            jobs_today: 0,
            quota_reset_at: Some(now),
            created_at: now,
            updated_at: now,
            last_login_at: None,
        }
    }

    /// Check if user can create a new job
    pub fn can_create_job(&self) -> bool {
        self.status == UserStatus::Active && self.jobs_today < self.quota
    }
}

/// User repository for Firestore
pub struct UserRepository {
    db: FirestoreDb,
    collection: String,
}

impl UserRepository {
    const COLLECTION: &'static str = "users";

    /// Create a new user repository
    pub fn new(db: FirestoreDb) -> Self {
        Self {
            db,
            collection: Self::COLLECTION.to_string(),
        }
    }

    /// Create a new user
    pub async fn create(&self, user: &User) -> Result<String, FirestoreError> {
        let id = generate_id();
        let mut user_with_id = user.clone();
        user_with_id.id = Some(id.clone());

        self.db
            .fluent()
            .insert()
            .into(&self.collection)
            .document_id(&id)
            .object(&user_with_id)
            .execute::<User>()
            .await?;
        Ok(id)
    }

    /// Create a user with a specific ID
    pub async fn create_with_id(&self, id: &str, user: &User) -> Result<(), FirestoreError> {
        self.db
            .fluent()
            .insert()
            .into(&self.collection)
            .document_id(id)
            .object(user)
            .execute::<User>()
            .await?;
        Ok(())
    }

    /// Get a user by ID
    pub async fn get(&self, id: &str) -> Result<Option<User>, FirestoreError> {
        Ok(self
            .db
            .fluent()
            .select()
            .by_id_in(&self.collection)
            .obj()
            .one(id)
            .await?)
    }

    /// Get a user by email
    pub async fn get_by_email(&self, email: &str) -> Result<Option<User>, FirestoreError> {
        let users: Vec<User> = self
            .db
            .fluent()
            .select()
            .from(self.collection.as_str())
            .filter(|q| q.field("email").eq(email))
            .limit(1)
            .obj()
            .query()
            .await?;
        Ok(users.into_iter().next())
    }

    /// Update a user
    pub async fn update(&self, id: &str, user: &User) -> Result<(), FirestoreError> {
        let mut updated = user.clone();
        updated.updated_at = Utc::now();

        let _: User = self
            .db
            .fluent()
            .update()
            .in_col(&self.collection)
            .document_id(id)
            .object(&updated)
            .execute()
            .await?;
        Ok(())
    }

    /// Increment jobs_today counter
    pub async fn increment_jobs(&self, id: &str) -> Result<(), FirestoreError> {
        // Note: For true atomic increment, use FieldValue::Increment
        // This is a simplified version
        if let Some(mut user) = self.get(id).await? {
            user.jobs_today += 1;
            user.updated_at = Utc::now();
            self.update(id, &user).await?;
        }
        Ok(())
    }

    /// Reset quota for a user
    pub async fn reset_quota(&self, id: &str) -> Result<(), FirestoreError> {
        if let Some(mut user) = self.get(id).await? {
            user.jobs_today = 0;
            user.quota_reset_at = Some(Utc::now());
            user.updated_at = Utc::now();
            self.update(id, &user).await?;
        }
        Ok(())
    }

    /// Record login
    pub async fn record_login(&self, id: &str) -> Result<(), FirestoreError> {
        if let Some(mut user) = self.get(id).await? {
            user.last_login_at = Some(Utc::now());
            user.updated_at = Utc::now();
            self.update(id, &user).await?;
        }
        Ok(())
    }

    /// Delete a user (soft delete)
    pub async fn delete(&self, id: &str) -> Result<(), FirestoreError> {
        if let Some(mut user) = self.get(id).await? {
            user.status = UserStatus::Deleted;
            user.updated_at = Utc::now();
            self.update(id, &user).await?;
        }
        Ok(())
    }

    /// List all active users
    pub async fn list_active(&self, limit: u32) -> Result<Vec<User>, FirestoreError> {
        Ok(self
            .db
            .fluent()
            .select()
            .from(self.collection.as_str())
            .filter(|q| q.field("status").eq("active"))
            .order_by([("created_at", FirestoreQueryDirection::Descending)])
            .limit(limit)
            .obj()
            .query()
            .await?)
    }
}
