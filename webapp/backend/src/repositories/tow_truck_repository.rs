use crate::domains::tow_truck_service::TowTruckRepository;
use crate::errors::AppError;
use crate::models::tow_truck::TowTruck;
use moka::future::Cache;
use sqlx::mysql::MySqlPool;

#[derive(Debug)]
pub struct TowTruckRepositoryImpl {
    pool: MySqlPool,
    latest_location_node_id_cache: Cache<i32, i32>,
    is_truck_busy_cache: Cache<i32, bool>,
}

impl TowTruckRepositoryImpl {
    pub fn new(
        pool: MySqlPool,
        latest_location_node_id_cache: Cache<i32, i32>,
        is_truck_busy_cache: Cache<i32, bool>,
    ) -> Self {
        TowTruckRepositoryImpl {
            pool,
            latest_location_node_id_cache,
            is_truck_busy_cache,
        }
    }
}

impl TowTruckRepository for TowTruckRepositoryImpl {
    fn get_is_busy_cache(&self) -> Cache<i32, bool> {
        self.is_truck_busy_cache.clone()
    }

    async fn get_paginated_tow_trucks(
        &self,
        page: i32,
        page_size: i32,
        status: Option<String>,
        area_id: Option<i32>,
    ) -> Result<Vec<TowTruck>, AppError> {
        let mut where_conditions = vec![];
        if let Some(status) = status {
            where_conditions.push(format!("tt.status = '{}'", status));
        }
        if let Some(area_id) = area_id {
            where_conditions.push(format!("tt.area_id = {}", area_id));
        }

        let where_clause = if where_conditions.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        let (limit_clause, offset_clause) = if page_size == -1 {
            ("".to_string(), "".to_string())
        } else {
            (
                format!("LIMIT {}", page_size),
                format!("OFFSET {}", page * page_size),
            )
        };

        // NOTE: node_id のキャッシュをここでも使うのは難しそう
        let query = format!(
            "SELECT
                tt.id,
                tt.driver_id,
                u.username AS driver_username,
                tt.status,
                tt.area_id,
                (SELECT node_id FROM locations WHERE tow_truck_id = tt.id ORDER BY timestamp DESC LIMIT 1) AS node_id
            FROM
                tow_trucks tt
            JOIN
                users u
            ON
                tt.driver_id = u.id
            {}
            ORDER BY
                tt.id ASC
            {}
            {}",
            where_clause, limit_clause, offset_clause
        );

        let tow_trucks = sqlx::query_as::<_, TowTruck>(&query)
            .fetch_all(&self.pool)
            .await?;

        for tt in tow_trucks.iter() {
            self.is_truck_busy_cache
                .insert(tt.id, tt.status == "busy")
                .await;
        }

        Ok(tow_trucks)
    }

    async fn update_location(&self, tow_truck_id: i32, node_id: i32) -> Result<(), AppError> {
        self.latest_location_node_id_cache
            .insert(tow_truck_id, node_id)
            .await;

        sqlx::query("INSERT INTO locations (tow_truck_id, node_id) VALUES (?, ?)")
            .bind(tow_truck_id)
            .bind(node_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_status(&self, tow_truck_id: i32, status: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE tow_trucks SET status = ? WHERE id = ?")
            .bind(status)
            .bind(tow_truck_id)
            .execute(&self.pool)
            .await?;

        self.is_truck_busy_cache
            .insert(tow_truck_id, status == "busy")
            .await;

        Ok(())
    }

    async fn find_tow_truck_by_id(&self, id: i32) -> Result<Option<TowTruck>, AppError> {
        let tow_truck = sqlx::query_as::<_, TowTruck>(
            "SELECT
                tt.id, tt.driver_id, u.username AS driver_username, tt.status, tt.area_id,
                1 AS node_id
            FROM
                tow_trucks tt
            JOIN
                users u
            ON
                tt.driver_id = u.id
            WHERE
                tt.id = ?
            ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(mut tow_truck) = tow_truck {
            self.is_truck_busy_cache
                .insert(tow_truck.id, tow_truck.status == "busy")
                .await;

            // NOTE: エラーハンドリングをちゃんとやる
            let node_id = self.latest_location_node_id_cache
            .try_get_with(tow_truck.id, async {
                let node_id = sqlx::query_scalar::<_, i32>(
                    "SELECT node_id FROM locations WHERE tow_truck_id = ? ORDER BY timestamp DESC LIMIT 1"
                )
                .bind(tow_truck.id)
                .fetch_one(&self.pool)
                .await?;
                Ok::<_, sqlx::Error>(node_id)
            })
            .await
            .map_err(|_| AppError::InternalServerError)?;

            tow_truck.node_id = node_id;
            Ok(Some(tow_truck))
        } else {
            Ok(None)
        }
    }
}
