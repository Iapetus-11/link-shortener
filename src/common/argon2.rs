use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

fn setup_argon2() -> Argon2<'static> {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(19 * 1024, 3, 2, Some(64)).unwrap(),
    )
}

pub fn check_key_against_hash(key: &str, hashed_key: &str) -> bool {
    setup_argon2()
        .verify_password(key.as_bytes(), &PasswordHash::new(hashed_key).unwrap())
        .is_ok()
}

pub fn hash_key(key: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);

    setup_argon2()
        .hash_password(key.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_key_and_check_key_against_hash() {
        let key = "password123";
        let hash = hash_key(key);

        assert!(check_key_against_hash(key, &hash));
        assert!(!check_key_against_hash("not the key oops", &hash));
    }

    #[test]
    fn test_check_key_on_invalid_hash() {
        let key_hash = hash_key("test");

        assert!(!check_key_against_hash("balls", &key_hash));
        assert!(!check_key_against_hash("", &key_hash));
        assert!(!check_key_against_hash(" ", &key_hash));
    }
}
