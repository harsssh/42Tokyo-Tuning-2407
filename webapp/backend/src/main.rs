use std::fs;
use std::sync::Arc;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use api::{
    auth_handler, health_check_handler, map_handler, order_handler, result_handler,
    tow_truck_handler,
};
use domains::map_service::MapService;
use domains::{
    auth_service::AuthService, order_service::OrderService, tow_truck_service::TowTruckService,
};
use middlewares::auth_middleware::AuthMiddleware;
use moka::future::Cache;
use repositories::auth_repository::AuthRepositoryImpl;
use repositories::map_repository::MapRepositoryImpl;
use repositories::order_repository::OrderRepositoryImpl;
use repositories::tow_truck_repository::TowTruckRepositoryImpl;

mod api;
mod domains;
mod errors;
mod infrastructure;
mod middlewares;
mod models;
mod repositories;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = infrastructure::db::create_pool().await;

    // NOTE: 本来は infrastructure に書く
    let latest_location_node_id_cache = Cache::builder()
        .max_capacity(2000)
        .time_to_live(Duration::from_secs(300))
        .build();
    let is_truck_busy_cache = Cache::builder()
        .max_capacity(2000)
        .time_to_live(Duration::from_secs(300))
        .build();

    let sock_path = "/tmp/da.sock";

    let auth_service = web::Data::new(AuthService::new(AuthRepositoryImpl::new(pool.clone())));
    let auth_service_for_middleware =
        Arc::new(AuthService::new(AuthRepositoryImpl::new(pool.clone())));
    let tow_truck_service = web::Data::new(TowTruckService::new(
        TowTruckRepositoryImpl::new(
            pool.clone(),
            latest_location_node_id_cache.clone(),
            is_truck_busy_cache.clone(),
        ),
        OrderRepositoryImpl::new(pool.clone()),
        MapRepositoryImpl::new(pool.clone()),
    ));
    let order_service = web::Data::new(OrderService::new(
        OrderRepositoryImpl::new(pool.clone()),
        TowTruckRepositoryImpl::new(
            pool.clone(),
            latest_location_node_id_cache.clone(),
            is_truck_busy_cache.clone(),
        ),
        AuthRepositoryImpl::new(pool.clone()),
        MapRepositoryImpl::new(pool.clone()),
    ));
    let map_service = web::Data::new(MapService::new(MapRepositoryImpl::new(pool.clone())));

    let server =
        HttpServer::new(move || {
            let mut cors = Cors::default();

            cors = cors
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec![
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::ACCEPT,
                ])
                .allowed_header(actix_web::http::header::CONTENT_TYPE)
                .supports_credentials()
                .max_age(3600);

            App::new()
                .app_data(tow_truck_service.clone())
                .app_data(auth_service.clone())
                .app_data(order_service.clone())
                .app_data(map_service.clone())
                .wrap(cors)
                .service(
                    web::scope("/api")
                        .service(
                            web::resource("/health_check")
                                .route(web::get().to(health_check_handler::health_check_handler)),
                        )
                        .service(
                            web::resource("/result")
                                .route(web::get().to(result_handler::result_handler)),
                        )
                        .service(
                            web::resource("/register")
                                .route(web::post().to(auth_handler::register_handler)),
                        )
                        .service(
                            web::resource("/login")
                                .route(web::post().to(auth_handler::login_handler)),
                        )
                        .service(
                            web::resource("/logout")
                                .route(web::post().to(auth_handler::logout_handler)),
                        )
                        .service(
                            web::resource("/user_image/{user_id}")
                                .route(web::get().to(auth_handler::user_profile_image_handler)),
                        )
                        .service(
                            web::scope("/tow_truck")
                                .wrap(AuthMiddleware::new(auth_service_for_middleware.clone()))
                                .service(
                                    web::resource("/list").route(
                                        web::get().to(
                                            tow_truck_handler::get_paginated_tow_trucks_handler,
                                        ),
                                    ),
                                )
                                .service(web::resource("/location").route(
                                    web::post().to(tow_truck_handler::update_location_handler),
                                ))
                                .service(web::resource("/nearest").route(web::get().to(
                                    tow_truck_handler::get_nearest_available_tow_trucks_handler,
                                )))
                                .service(web::resource("/{id}").route(
                                    web::get().to(tow_truck_handler::get_tow_truck_handler),
                                )),
                        )
                        .service(
                            web::scope("/order")
                                .wrap(AuthMiddleware::new(auth_service_for_middleware.clone()))
                                .service(web::resource("/list").route(
                                    web::get().to(order_handler::get_paginated_orders_handler),
                                ))
                                .service(web::resource("/status").route(
                                    web::post().to(order_handler::update_order_status_handler),
                                ))
                                .service(web::resource("/client").route(
                                    web::post().to(order_handler::create_client_order_handler),
                                ))
                                .service(web::resource("/dispatcher").route(
                                    web::post().to(order_handler::create_dispatcher_order_handler),
                                ))
                                .service(
                                    web::resource("/{id}")
                                        .route(web::get().to(order_handler::get_order_handler)),
                                ),
                        )
                        .service(
                            web::scope("/map")
                                .wrap(AuthMiddleware::new(auth_service_for_middleware.clone()))
                                .service(
                                    web::resource("/update_edge")
                                        .route(web::put().to(map_handler::update_edge_handler)),
                                ),
                        ),
                )
        })
        .bind_uds(sock_path)
        .expect("Failed to bind uds");

    // 755 -> 777 にする
    while fs::metadata(sock_path).is_err() {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    use std::os::unix::fs::PermissionsExt;
    let mut perm = fs::metadata(sock_path)?.permissions();
    perm.set_mode(0o777);
    fs::set_permissions(sock_path, perm)?;

    // TODO: 最適値を考える
    server.workers(8).backlog(256).run().await
}
