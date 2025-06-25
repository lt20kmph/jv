use crate::models::models::SaltedPassword;
use argon2::password_hash;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn verify_password(
    db_salt: String,
    db_password_hash: &str,
    password: &str,
) -> Result<bool, password_hash::Error> {
    let salt = SaltString::from_b64(db_salt.as_str())?;

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash == db_password_hash)
}

pub fn hash_and_salt_password(user_password: &str) -> Result<SaltedPassword, password_hash::Error> {
    let password = user_password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password, &salt)?.to_string();

    let parsed_hash = PasswordHash::new(&password_hash)?;
    assert!(Argon2::default()
        .verify_password(password, &parsed_hash)
        .is_ok());

    Ok(SaltedPassword {
        password_hash,
        salt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::password_hash::SaltString;

    #[test]
    fn test_hash_and_salt_password_basic() {
        let password = "test_password_123";
        let result = hash_and_salt_password(password);
        
        assert!(result.is_ok());
        let salted_password = result.unwrap();
        
        assert!(!salted_password.password_hash.is_empty());
        assert!(!salted_password.salt.as_str().is_empty());
    }

    #[test]
    fn test_hash_and_salt_password_unique_salts() {
        let password = "same_password";
        
        let result1 = hash_and_salt_password(password).unwrap();
        let result2 = hash_and_salt_password(password).unwrap();
        
        assert_ne!(result1.salt.as_str(), result2.salt.as_str());
        assert_ne!(result1.password_hash, result2.password_hash);
    }

    #[test]
    fn test_hash_and_salt_password_empty_password() {
        let password = "";
        let result = hash_and_salt_password(password);
        
        assert!(result.is_ok());
        let salted_password = result.unwrap();
        assert!(!salted_password.password_hash.is_empty());
    }

    #[test]
    fn test_hash_and_salt_password_long_password() {
        let password = "a".repeat(1000);
        let result = hash_and_salt_password(&password);
        
        assert!(result.is_ok());
        let salted_password = result.unwrap();
        assert!(!salted_password.password_hash.is_empty());
    }

    #[test]
    fn test_hash_and_salt_password_special_characters() {
        let password = "p@ssw0rd!#$%^&*()_+-=[]{}|;':\",./<>?~`";
        let result = hash_and_salt_password(password);
        
        assert!(result.is_ok());
        let salted_password = result.unwrap();
        assert!(!salted_password.password_hash.is_empty());
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "test_password_123";
        let salted_password = hash_and_salt_password(password).unwrap();
        
        let result = verify_password(
            salted_password.salt.as_str().to_string(),
            &salted_password.password_hash,
            password,
        );
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";
        let salted_password = hash_and_salt_password(correct_password).unwrap();
        
        let result = verify_password(
            salted_password.salt.as_str().to_string(),
            &salted_password.password_hash,
            wrong_password,
        );
        
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_password_empty_password() {
        let password = "";
        let salted_password = hash_and_salt_password(password).unwrap();
        
        let result = verify_password(
            salted_password.salt.as_str().to_string(),
            &salted_password.password_hash,
            password,
        );
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_round_trip() {
        let passwords = vec![
            "simple".to_string(),
            "with spaces and symbols!@#".to_string(),
            "🦀🔐emoji_password".to_string(),
            "very_long_password_".repeat(10),
        ];
        
        for password in passwords {
            let salted_password = hash_and_salt_password(&password).unwrap();
            let result = verify_password(
                salted_password.salt.as_str().to_string(),
                &salted_password.password_hash,
                &password,
            );
            
            assert!(result.is_ok(), "Failed for password: {}", password);
            assert!(result.unwrap(), "Verification failed for password: {}", password);
        }
    }

    #[test]
    fn test_verify_password_invalid_salt() {
        let password = "test_password";
        let salted_password = hash_and_salt_password(password).unwrap();
        
        let result = verify_password(
            "invalid_salt_string".to_string(),
            &salted_password.password_hash,
            password,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_malformed_salt() {
        let password = "test_password";
        let salted_password = hash_and_salt_password(password).unwrap();
        
        let result = verify_password(
            "not_base64!@#$%".to_string(),
            &salted_password.password_hash,
            password,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_empty_salt() {
        let password = "test_password";
        let salted_password = hash_and_salt_password(password).unwrap();
        
        let result = verify_password(
            "".to_string(),
            &salted_password.password_hash,
            password,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_security_different_passwords_different_hashes() {
        let password1 = "password1";
        let password2 = "password2";
        
        let salted1 = hash_and_salt_password(password1).unwrap();
        let salted2 = hash_and_salt_password(password2).unwrap();
        
        assert_ne!(salted1.password_hash, salted2.password_hash);
        assert_ne!(salted1.salt.as_str(), salted2.salt.as_str());
    }

    #[test]
    fn test_security_same_password_different_salts_different_hashes() {
        let password = "same_password";
        
        let salted1 = hash_and_salt_password(password).unwrap();
        let salted2 = hash_and_salt_password(password).unwrap();
        let salted3 = hash_and_salt_password(password).unwrap();
        
        assert_ne!(salted1.password_hash, salted2.password_hash);
        assert_ne!(salted1.password_hash, salted3.password_hash);
        assert_ne!(salted2.password_hash, salted3.password_hash);
        
        assert_ne!(salted1.salt.as_str(), salted2.salt.as_str());
        assert_ne!(salted1.salt.as_str(), salted3.salt.as_str());
        assert_ne!(salted2.salt.as_str(), salted3.salt.as_str());
    }

    #[test]
    fn test_hash_format_validity() {
        let password = "test_password";
        let salted = hash_and_salt_password(password).unwrap();
        
        assert!(salted.password_hash.starts_with("$argon2"));
        assert!(salted.salt.as_str().len() > 10);
        
        let parsed_hash = argon2::password_hash::PasswordHash::new(&salted.password_hash);
        assert!(parsed_hash.is_ok());
    }
}
