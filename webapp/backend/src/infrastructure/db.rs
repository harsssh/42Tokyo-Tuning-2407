use sqlx::mysql::MySqlPool;

// TODO: パラメータの最適値を考える
pub async fn create_pool() -> MySqlPool {
    let options = sqlx::mysql::MySqlConnectOptions::new()
        .socket("/var/run/mysqld/mysqld.sock")
        .username("user")
        .password("password")
        .database("42Tokyo-db")
        .statement_cache_capacity(1000);

    sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(40)
        .connect_with(options)
        .await
        .expect("failed to connect db")
}
