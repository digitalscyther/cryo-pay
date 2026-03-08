use sqlx::PgPool;

pub async fn get_block_number(db: &PgPool, network: &str) -> Result<Option<i64>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT block_number FROM network_monitor WHERE network = $1",
        network
    )
        .fetch_optional(db)
        .await?;

    Ok(result.map(|record| record.block_number))
}

pub async fn create_or_update_block_number(db: &PgPool, network: &str, block_number: i64) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        r#"
        INSERT INTO network_monitor (network, block_number)
        VALUES ($1, $2)
        ON CONFLICT (network) DO UPDATE
        SET block_number = EXCLUDED.block_number
        "#,
        network.to_lowercase(),
        block_number
    )
        .execute(db)
        .await?;

    Ok(())
}
