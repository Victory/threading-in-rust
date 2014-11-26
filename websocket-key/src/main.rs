extern crate "rust-crypto" as rust_crypto;
extern crate serialize;

use rust_crypto::sha1::Sha1;
use rust_crypto::digest::Digest;
use serialize::base64::{ToBase64, STANDARD};

fn main () {
    let from_server = b"dGhlIHNhbXBsZSBub25jZQ==";
    let guid = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    let mut sha = Sha1::new();

    sha.input(from_server);
    sha.input(guid);
    let mut out = [0u8, ..20];
    sha.result(out.as_mut_slice());

    println!("{} {}", sha.result_str(), out.to_base64(STANDARD));
}
