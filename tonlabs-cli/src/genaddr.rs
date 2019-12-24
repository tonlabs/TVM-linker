use crate::config::Config;
use crate::helpers::read_keys;
use crc16::*;
use hex;
use std::fs::File;
use std::io::Write;
use ton_client_rs::{TonClient, TonAddress};
pub fn generate_address(conf: Config, tvc: &str, _wc_str: Option<&str>, keys_file: Option<&str>, new_keys: bool) -> Result<(), String> {
    let ton = TonClient::new_with_base_url(&conf.url)
        .map_err(|e| format!("failed to create tonclient: {}", e.to_string()))?;

    let contract = std::fs::read(tvc)
        .map_err(|e| format!("failed to read smart contract file: {}", e.to_string()))?;

    let mut new_keys_file = None;
    let keys = if let Some(filename) = keys_file {
        if new_keys {
            new_keys_file = Some(filename);
            ton.crypto.generate_ed25519_keys()
            .map_err(|e| format!("keypair generation failed: {}", e.to_string()))?
        } else {
            read_keys(filename)?
        }
    } else {
        ton.crypto.generate_ed25519_keys()
        .map_err(|e| format!("failed to generate keypair: {}", e.to_string()))?
    };
        
    //TODO: use wc_str in address.
    let addr = ton.contracts.get_deploy_address(&contract, &keys)
        .map_err(|e| format!("failed to generate address: {}", e.to_string()))?;

    println!("Raw address: {}", addr);

    if new_keys_file.is_some() {
        let mut file = File::create(new_keys_file.unwrap().to_string()).unwrap();
        file.write_all(&keys.to_bytes()).unwrap();
    }

    if let TonAddress::Std(wc, addr256) = addr {
        println!("testnet:");
        println!("Non-bounceable address (for init): {}", &calc_userfriendly_address(wc, &addr256, false, true));
        println!("Bounceable address (for later access): {}", &calc_userfriendly_address(wc, &addr256, true, true));
        println!("mainnet:");
        println!("Non-bounceable address (for init): {}", &calc_userfriendly_address(wc, &addr256, false, false));
        println!("Bounceable address (for later access): {}", &calc_userfriendly_address(wc, &addr256, true, false));
    }
    Ok(())
}

fn calc_userfriendly_address(wc: i8, addr: &[u8], bounce: bool, testnet: bool) -> String {
    let mut bytes: Vec<u8> = vec![];
    bytes.push(if bounce { 0x11 } else { 0x51 } + if testnet { 0x80 } else { 0 });
    bytes.push(wc as u8);
    bytes.extend_from_slice(addr);
    let crc = State::<XMODEM>::calculate(&bytes);
    bytes.extend_from_slice(&crc.to_be_bytes());
    hex::encode(&bytes)
}