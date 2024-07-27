use crate::domains::tow_truck_service::TowTruckRepository;
use crate::errors::AppError;
use crate::models::tow_truck::TowTruck;
use sqlx::mysql::MySqlPool;

#[derive(Debug)]
pub struct TowTruckRepositoryImpl {
    pool: MySqlPool,
}

impl TowTruckRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        TowTruckRepositoryImpl { pool }
    }
}

impl TowTruckRepository for TowTruckRepositoryImpl {
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

        Ok(tow_trucks)
    }

    async fn update_location(&self, tow_truck_id: i32, node_id: i32) -> Result<(), AppError> {
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

        Ok(())
    }

    async fn find_tow_truck_by_id(&self, id: i32) -> Result<Option<TowTruck>, AppError> {
        let tow_truck = sqlx::query_as::<_, TowTruck>(
            "SELECT
                tt.id, tt.driver_id, u.username AS driver_username, tt.status, l.node_id, tt.area_id
            FROM
                tow_trucks tt
            JOIN
                users u
            ON
                tt.driver_id = u.id
            JOIN
                locations l
            ON
                tt.id = l.tow_truck_id
            WHERE
                tt.id = ?
            AND
                l.timestamp = (SELECT MAX(timestamp) FROM locations WHERE tow_truck_id = tt.id)",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(tow_truck)
    }
}
