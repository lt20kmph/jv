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
