
use sha2::{Digest, Sha256};
use hex;
use std::string::FromUtf8Error;

fn hex_to_string(hex_str: &str) -> Result<String, FromUtf8Error> {
    let _bytes = hex::decode(hex_str);
    match _bytes{
        Ok(bytes) =>  String::from_utf8(bytes),
        _ => panic!("invalid hex string")
    }
    
}

fn checksum(geohash_hex: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(geohash_hex);
    let hash_result = hasher.finalize();
    let hash_hex = format!("{:x}", hash_result);
    let checksum = &hash_hex[0..2];
    checksum.to_string()
}

fn get_identifier(id: &str) -> char{
    match hex_to_string(id){
        Ok(prefix) => prefix.chars().next().unwrap(),
        _ => panic!("invalid ID")
    }
}

fn hex_to_geohash(hex: &str) -> String {
    let base32_chars = "0123456789bcdefghjkmnpqrstuvwxyz";
    let mut geohash = String::new();

    let mut iter = hex.chars().step_by(2);

    while let Some(hex_pair) = iter.next() {
        if let Some(decimal_value) = u8::from_str_radix(&hex_pair.to_string(), 16).ok() {
            let base32_char = base32_chars.as_bytes()[usize::from(decimal_value % 32)] as char;
            geohash.push(base32_char);
        }
    }

    geohash
}

pub fn geohash_from_id(id: &String) -> Result<String, &'static str>{
    let id: &str = id;
    let _prefix = get_identifier(&id[0..2]);
    let geohex: &str;
    match _prefix{
        'c'=>geohex = &id[30..],
        'C'=>geohex = &id[36..],
        'W'=>geohex = &id[36..],
        _ => panic!("Invalid prefix")
    }
    let chk = checksum(geohex);
    if chk != id[2..4]{
        panic!("Invalid checksum");
    }
    Ok(hex_to_geohash(geohex))
}