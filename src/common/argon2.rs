use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

/// Setup an Argon2 instance with weaker params, suitable for short-lived tokens
pub fn setup_weak_argon2() -> Argon2<'static> {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(13 * 1024, 2, 1, Some(64)).unwrap(),
    )
}

/// Setup an Argon2 instance with strong params, suitable for hashing passwords and long-lived tokens
pub fn setup_strong_argon2() -> Argon2<'static> {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(19 * 1024, 3, 2, Some(64)).unwrap(),
    )
}

pub fn argon2_check_key_against_hash(argon2: &Argon2<'_>, key: &str, hashed_key: &str) -> bool {
    argon2
        .verify_password(key.as_bytes(), &PasswordHash::new(hashed_key).unwrap())
        .is_ok()
}

pub fn argon2_hash_key(argon2: &Argon2<'_>, key: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);

    argon2
        .hash_password(key.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_key_and_check_key_against_hash() {
        let argon2 = setup_weak_argon2();

        let key = "password123";
        let hash = argon2_hash_key(&argon2, key);

        assert!(argon2_check_key_against_hash(&argon2, key, &hash));
        assert!(!argon2_check_key_against_hash(
            &argon2,
            "not the key oops",
            &hash
        ));
    }

    #[test]
    fn test_check_invalid_key() {
        let argon2 = setup_weak_argon2();

        let key_hash = argon2_hash_key(&argon2, "test");

        assert!(!argon2_check_key_against_hash(&argon2, "balls", &key_hash));
        assert!(!argon2_check_key_against_hash(&argon2, "", &key_hash));
        assert!(!argon2_check_key_against_hash(&argon2, " ", &key_hash));
    }
}
