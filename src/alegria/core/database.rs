// SPDX-License-Identifier: GPL-3.0-only

use dotenvy::dotenv;
use sqlx::PgPool;
use std::{env, sync::Arc};

pub async fn init_database() -> Arc<PgPool> {
    // Load .env file
    dotenv().ok();

    let pool = PgPool::connect(&env::var("DATABASE_URL").expect("No database URL set"))
        .await
        .expect("Error creating database");

    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => println!("Migrations run successfully"),
        Err(err) => {
            eprintln!("Error occurred running migrations: {}", err);
            std::process::exit(1);
        }
    };

    Arc::new(pool)
}
