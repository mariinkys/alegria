// SPDX-License-Identifier: GPL-3.0-only

use dotenvy::dotenv;
use sqlx::PgPool;
use std::{env, sync::Arc};
use tokio::time::{Duration, timeout};

pub async fn init_database() -> Result<Arc<PgPool>, String> {
    // Load .env file
    dotenv().ok();

    let db_url = &env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not found.")?;

    let pool = timeout(Duration::from_secs(3), PgPool::connect(db_url))
        .await
        .map_err(|_| "Timed out connecting to the database".to_string())?
        .map_err(|e| format!("Could not connect to database: {}", e))?;

    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => println!("Migrations run successfully"),
        Err(err) => {
            return Err(format!("Error occurred running migrations: {}", err));
        }
    };

    Ok(Arc::new(pool))
}
