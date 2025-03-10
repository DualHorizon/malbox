use crc32fast::Hasher;
use md5::compute;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

pub fn get_md5(buf: &mut [u8]) -> String {
    let digest = compute(buf);
    format!("{:x}", digest)
}

pub fn get_sha1(buf: &mut [u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(buf);
    let result = hasher.finalize();

    let mut hex_string = String::new();
    for byte in result.iter() {
        hex_string.push_str(&format!("{:x}", byte));
    }

    hex_string
}

pub fn get_sha256(buf: &mut [u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(buf);
    let result = hasher.finalize();

    let mut hex_string = String::new();
    for byte in result.iter() {
        hex_string.push_str(&format!("{:02x}", byte));
    }

    hex_string
}

pub fn get_sha512(buf: &mut [u8]) -> String {
    let mut hasher = Sha512::new();
    hasher.update(buf);
    let result = hasher.finalize();

    let mut hex_string = String::new();
    for byte in result.iter() {
        hex_string.push_str(&format!("{:x}", byte));
    }

    hex_string
}

pub fn get_crc32(buf: &mut [u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(buf);
    let result = hasher.finalize();

    format!("{:x}", result)
}

// NOTE: temporarly removing ssdeep crate because of build issues..
// pub fn get_ssdeep(buf: &mut [u8]) -> String {
//    ssdeep::hash(buf).unwrap()
// }
