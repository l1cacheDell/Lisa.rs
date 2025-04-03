use aptos_sdk::rest_client::{Client as aptos_client, Transaction as RestTransaction, Transaction};
use aptos_sdk::crypto::HashValue;
use url::Url;
use std::str::FromStr;

pub async fn verify_tx(tx_hash_str: &str) -> Result<bool, anyhow::Error> {
    // test aptos zone
    let base_url = Url::parse("https://fullnode.testnet.aptoslabs.com").expect("Invalid URL");
    let client = aptos_client::new(base_url);
    let txn_hash_str_proc = tx_hash_str.trim_start_matches("0x");

    let tx_hash = match HashValue::from_str(txn_hash_str_proc) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Invalid transaction hash: {:?}", e);
            return Err(e.into());
        }
    };

    // 查询交易
    if let Ok(response) = client.get_transaction_by_hash(tx_hash).await {
        let transaction = response.inner();
        if let Transaction::UserTransaction(user_txn) = transaction {
            println!("Transaction fetched!");
            let tx_success = user_txn.info.success;
            let vm_status = &user_txn.info.vm_status;
            if tx_success && vm_status == "Executed successfully" {
                return Ok(true);
            } else {
                return Ok(false);
            }
        } else {
            println!("Transaction found but it's not a user transaction.");
            return Ok(true);
        }
    } else {
        println!("Error fetching transaction: {}", tx_hash_str);
        return Ok(false);
    }
}