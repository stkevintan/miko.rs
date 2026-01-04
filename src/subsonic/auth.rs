use md5;
use crate::crypto::decrypt;

pub fn decrypt_password(stored_password: &str, secret: &[u8]) -> String {
    match decrypt(stored_password, secret) {
        Ok(decrypted) => decrypted,
        Err(_) => stored_password.to_string(),
    }
}

pub fn verify_password(stored_password: &str, password: &str, secret: &[u8]) -> bool {
    let decrypted = decrypt_password(stored_password, secret);
    decrypted == password
}

pub fn verify_token(stored_password: &str, token: &str, salt: &str, secret: &[u8]) -> bool {
    let decrypted = decrypt_password(stored_password, secret);
    let expected_token = format!("{:x}", md5::compute(format!("{}{}", decrypted, salt)));
    expected_token == token
}
