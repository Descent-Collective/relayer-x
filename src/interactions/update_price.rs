use core::panic;
use dotenv::dotenv;
use ethers::{
    abi::{encode, FixedBytes, Token},
    prelude::abigen,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, Bytes, H256, U256},
    utils::{hex::FromHex, parse_units, ParseUnits},
};
use std::{str::FromStr, sync::Arc};

use crate::prices;

pub async fn update_price() -> eyre::Result<()> {
    dotenv().ok();

    abigen!(
        IOracleModule,
        r#"[
            function update(uint256[] calldata _prices, uint64[] calldata _timestamps, bytes[] calldata _signatures) external
            function read() external view returns (uint256, uint256) 

        ]"#
    );

    // define rpc url and oracle module address
    let rpc_url: String = std::env::var("RPC_URL").expect("RPC_URL must be set in your .env file");
    let oracle_module_address: Address = std::env::var("ORACLE_MODULE_ADDRESS")
        .expect("ORACLE_MODULE_ADDRESS must be set in your .env file")
        .parse()
        .expect("failed to parse ORACLE_MODULE_ADDRESS as an address");

    // define provider to use from rpc url
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);

    // create an instance of oracle module
    let _oracle_module = IOracleModule::new(oracle_module_address, client);

    // get all prices from fetch_prices module
    let prices_and_timestamps = prices::fetch_prices().await?;

    // Check if current price is same as last price
    let current_price = prices_and_timestamps[0].0;
    let last_price = _oracle_module.read().await?.0.as_u64() as f32;

    if current_price == last_price {
        return Ok(());
    }

    // define vectors to be used to store variables to parse to the function onchain
    let mut prices: Vec<U256> = Vec::new();
    let mut timestamps: Vec<u64> = Vec::new();
    let mut signatures: Vec<Bytes> = Vec::new();

    // filter the price and timestamp info and push to their respective vectors
    for (p, t) in prices_and_timestamps.into_iter() {
        // turn into a 6 fixed-point unsigned integer
        let first = parse_units(p, 6).expect("failed to convert float to ethers unit");

        // match to only take U256 types
        let second = match first {
            ParseUnits::U256(a) => a,
            ParseUnits::I256(_) => panic!("Negative values not allowed"), // sanity check
        };

        // push to respective arrays
        prices.push(second);
        timestamps.push(t);
    }

    // wallet to sign tx. get private key from env, and revert if any unexpected stuff happens
    let wallet = LocalWallet::from_bytes(
        H256::from_str(
            &std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set in your .env file"),
        )
        .expect("invalid hex private key")
        .as_bytes(),
    )
    .unwrap();

    // loop through price and timestamp array, abi encode them concatenated together, hash and sign it as "\x19Ethereum Signed Message:\n" + len + encoded message
    for i in 0..prices.len() {
        // encode price + timestamp. where + is concatenation
        let message = encode(&[
            Token::Uint(prices[i]),
            Token::Uint(
                U256::try_from(timestamps[i]).expect("could not convert from u64 to timestamp"),
            ),
            Token::FixedBytes(FixedBytes::from(
                Vec::from_hex("555344432f784e474e0000000000000000000000000000000000000000000000")?, // hex of "USDC/xNGN"
            )),
        ]);

        // sign message with wallet
        let signature = wallet.sign_message(&message).await?;

        // push to vector
        let signature = Bytes::from(signature.to_vec());
        if signature.len() != 65 {
            panic!("invalid sig length");
        }
        signatures.push(signature);
    }

    let built_tx_object = _oracle_module
        .update(prices, timestamps, signatures)
        .from(wallet.address());
    let tx = built_tx_object.send().await?;

    println!("{:?}", built_tx_object);
    println!("{:?}", tx);

    Ok(())
}
