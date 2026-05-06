//! Authentication utilities
//!
//! Provides password hashing and verification functions.

use bcrypt::{hash, verify, DEFAULT_COST};

/// Hash a password using bcrypt
///
/// # Arguments
/// * `password` - The plain text password to hash
///
/// # Returns
/// The hashed password string, or an error
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

/// Verify a password against a hash
///
/// # Arguments
/// * `password` - The plain text password to verify
/// * `hash` - The bcrypt hash to verify against
///
/// # Returns
/// `true` if the password matches, `false` otherwise
pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

/// Check if a channel requires password verification
///
/// # Arguments
/// * `has_password` - Whether the channel has a password
/// * `provided_password` - The password provided by the user
/// * `stored_hash` - The stored password hash (if any)
///
/// # Returns
/// `Ok(true)` if access is granted, `Ok(false)` if password is required but not provided,
/// `Err` if the password is wrong or verification failed
pub fn check_channel_access(
    has_password: bool,
    provided_password: Option<&str>,
    stored_hash: Option<&str>,
) -> Result<bool, String> {
    if !has_password {
        // No password required
        return Ok(true);
    }

    match (provided_password, stored_hash) {
        (Some(password), Some(hash)) => {
            // Verify password
            verify_password(password, hash)
                .map_err(|e| format!("Password verification failed: {}", e))
        }
        (Some(_), None) => {
            // Password provided but no hash stored (should not happen)
            Err("Channel configuration error".to_string())
        }
        (None, _) => {
            // Password required but not provided
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test_password_123";
        let hashed = hash_password(password).unwrap();

        // Hash should be different from plain password
        assert_ne!(password, hashed);

        // Hash should start with bcrypt identifier
        assert!(hashed.starts_with("$2b$"));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "test_password_123";
        let hashed = hash_password(password).unwrap();

        let result = verify_password(password, &hashed).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_password_wrong() {
        let password = "correct_password";
        let hashed = hash_password(password).unwrap();

        let result = verify_password("wrong_password", &hashed).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_check_channel_access_no_password_required() {
        let result = check_channel_access(false, None, None).unwrap();
        assert!(result);
    }

    #[test]
    fn test_check_channel_access_password_correct() {
        let password = "channel_password";
        let hashed = hash_password(password).unwrap();

        let result = check_channel_access(true, Some(password), Some(&hashed)).unwrap();
        assert!(result);
    }

    #[test]
    fn test_check_channel_access_password_wrong() {
        let password = "correct_password";
        let hashed = hash_password(password).unwrap();

        let result = check_channel_access(true, Some("wrong_password"), Some(&hashed)).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_check_channel_access_password_not_provided() {
        let hashed = hash_password("some_password").unwrap();

        let result = check_channel_access(true, None, Some(&hashed)).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let hash1 = hash_password("password1").unwrap();
        let hash2 = hash_password("password2").unwrap();

        // Different passwords should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_password_different_hashes() {
        let password = "same_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Same password should produce different hashes (due to salt)
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }
}
