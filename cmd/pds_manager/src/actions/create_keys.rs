use secp256k1::{rand, generate_keypair};

pub fn create_keys() {
    let (secret_key, public_key) = generate_keypair(&mut rand::rng());

    let secret_key_bytes: [u8; 32] = secret_key.secret_bytes();
    println!("\
        Private key: {}\n\
        Public key: {}\
    ", format_u8_array(secret_key_bytes), public_key)
}

fn format_u8_array(bytes: [u8; 32]) -> String {
    let string: &mut String = &mut String::new();
    for byte in &bytes {
        (string).extend(format!("{:x}", byte).chars());
    }
    string.to_string()
}