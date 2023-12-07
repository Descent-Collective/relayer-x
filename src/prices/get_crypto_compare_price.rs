use crate::prices::utils;
use dotenv::dotenv;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct Response {
    NGN: f32,
}

pub async fn get_crypto_compare_price() -> eyre::Result<(f32, u64)> {
    dotenv().ok();

    // Env variables
    let url: String = std::env::var("CRYPTO_COMPARE_URL").expect("CRYPTO_COMPARE_URL not set");

    let url = format!("{}/price?fsym=USDC&tsyms=NGN", url);

    let client = reqwest::Client::new();
    let body = client.get(&url).send().await?.json::<Response>().await?;

    // Calculate the Unix timestamp
    let unix_timestamp = utils::get_unix_timestamp();

    Ok((body.NGN, unix_timestamp))
}
