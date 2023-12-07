mod update_price;

pub async fn update_price() -> eyre::Result<()> {
    update_price::update_price().await?;
    Ok(())
}
