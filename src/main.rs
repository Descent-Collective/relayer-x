use warp::Filter;

pub mod interactions;
pub mod prices;

// Asynchronous handler function
async fn update_handler() -> Result<impl warp::Reply, warp::Rejection> {
    // Get latest price and timestamp
    let prices_and_timestamps = prices::fetch_prices().await.unwrap_or(vec![]);

    // Update oracle with latest price and timestamp
    interactions::update_price().await.unwrap_or(());

    Ok(warp::reply::json(&prices_and_timestamps))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define a warp filter for the "/async" path
    let async_route = warp::path!("async").and_then(|| update_handler());

    // Run the warp server
    warp::serve(async_route).run(([127, 0, 0, 1], 8080)).await;

    Ok(())
}
