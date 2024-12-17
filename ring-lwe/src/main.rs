mod keygen;
mod encrypt;
mod decrypt;

use crate::keygen::{keygen,keygen_string};
use crate::encrypt::{encrypt,encrypt_string};
use crate::decrypt::{decrypt,decrypt_string};
use std::env;
use ring_lwe::Parameters;
use polynomial_ring::Polynomial;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Initialize struct with default values
    let mut params = Parameters::default();
    // Check for --params flag and get the updated values directly
    if let Some(pos) = args.iter().position(|x| x == "--params") {
        if args.len() > pos + 3 {
            params.n = args.get(pos + 1).and_then(|s| s.parse().ok()).unwrap_or(params.n);
            params.q = args.get(pos + 2).and_then(|s| s.parse().ok()).unwrap_or(params.q);
            params.t = args.get(pos + 3).and_then(|s| s.parse().ok()).unwrap_or(params.t);
            let mut poly_vec = vec![0i64;params.n+1];
            poly_vec[0] = 1;
            poly_vec[params.n] = 1;
            params.poly_mod = Polynomial::new(poly_vec);
        }
    }

    let method = if args.len() > 1 {&args[1]} else {""};

    //perform a basis keygen/encrypt/decrypt test on single message
    if method == "test" {
        if args.len() != 3 && args.len() != 7 {
            println!("Usage: cargo run -- test <message>");
            return;
        }
        let message_string = &args[2];
        let keypair = keygen_string(&params);
        let pk_string = keypair.get("public").unwrap();
        let sk_string = keypair.get("secret").unwrap();
        let ciphertext_string = encrypt_string(&pk_string,message_string,&params);
        let decrypted_message = decrypt_string(&sk_string,&ciphertext_string,&params);
        let test_passed = *message_string == decrypted_message;
        println!("{} =? {}",*message_string,decrypted_message);
        println!("{}",test_passed);
    }

    //test (partially) homomorphic property on two integers
    if method == "test_hom" {
        if args.len() != 4 && args.len() != 8 {
            println!("Usage: cargo run -- test <message_0> <message_1>");
            return;
        }
        //read the message strings
        let m0_string = &args[2];
        let m1_string = &args[3];
        //create polynomials from message strings
        let m0_int: i64 = m0_string.parse().expect("Failed to parse integer.");
        let m1_int: i64 = m1_string.parse().expect("Failed to parse integer.");
        let m0_poly = Polynomial::new({
            let mut v = vec![0i64; params.n + 1];
            v[0] = m0_int;
            v
        });
        let m1_poly = Polynomial::new({
            let mut v = vec![0i64; params.n + 1];
            v[0] = m1_int;
            v
        });
        //generate the keypair
        let keypair = keygen(params.n,params.q as i64,&params.poly_mod);
        //get public and secret keys
        let pk = keypair.0;
        let sk = keypair.1;
        //encrypt plaintext messages
        let u = encrypt(&pk,params.n,params.q as i64,params.t as i64,&params.poly_mod,&m0_poly);
        let v = encrypt(&pk,params.n,params.q as i64,params.t as i64,&params.poly_mod,&m1_poly);
        //compute sum of encrypted data
        let ciphertext_sum = [&u.0 + &v.0, &u.1 + &v.1];
        //compute product of encrypted data, using non-standard multiplication
        let _c = [&v.0 * &v.1, -(&u.0 * &v.1 + &u.1 * v.0), &u.0 * &u.1];
        //decrypt encrypted sum
        let decrypted_sum = decrypt(&sk,params.n,params.q as i64,params.t as i64,&params.poly_mod,&ciphertext_sum);
        //decrypt product using relinearization
        let delta = params.q as f64 / params.t as f64;
        println!("delta = {}",delta);
        //let decrypted_product = decrypt(&sk,params.n,params.q as i64,params.t as i64,&params.poly_mod,&c_prod);
        //print plaintext sum/product v. decrypted sum/products
        println!("plaintext sum = {}", m0_int + m1_int);
        println!("decrypted_sum = {}",decrypted_sum);
        println!("plaintext product = {}", m0_int * m1_int);
        //println!("decrypted_product = {}",decrypted_product);
    }

    //generate public and secret keys (parameters optional)
    if method == "keygen"{
        if args.len() != 2 && args.len() != 6 {
            println!("Usage: cargo run -- keygen");
            return;
        }
        let keypair = keygen_string(&params);
        println!("{:?}",keypair);
    }

    //encrypt given public key and message as args (parameters optional)
    if method == "encrypt" {
        if args.len() != 4 && args.len() != 8 {
            println!("Usage: cargo run -- encrypt <public_key> <message_string>");
            return;
        }
        let pk_string = &args[2];
        let message = &args[3];
        let ciphertext_string = encrypt_string(pk_string,message,&params);
        println!("{}", ciphertext_string);
    }

    //decrypt a messsage (parameters optional)
    if method == "decrypt" {
        if args.len() != 4 && args.len() != 8 {
            println!("Usage: cargo run -- decrypt <secret_key> <ciphertext>");
            return;
        }
        let sk_string = &args[2];
        let ciphertext_string = &args[3];
        let decrypted_message = decrypt_string(sk_string, ciphertext_string,&params);
        // Print the decrypted message
        println!("{}", decrypted_message);
    }

}