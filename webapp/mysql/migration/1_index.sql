-- users.username にユニーク制約
CALL DropIndexIfExists ('users', 'uk_username');
CREATE UNIQUE INDEX `uk_username` ON `users` (`username`);

-- dispachers.user_id にインデックス
CALL DropIndexIfExists ('dispatchers', 'idx_user_id');
CREATE INDEX `idx_user_id` ON `dispatchers` (`user_id`);

-- orders の (node_id, order_time) にインデックス
CALL DropIndexIfExists ( 'orders', 'idx_status_node_id_order_time' );
CREATE INDEX `idx_status_node_id_order_time` ON `orders` (
    `status`,
    `node_id`,
    `order_time`
);

-- nodes.area_id にインデックス
CALL DropIndexIfExists ('nodes', 'idx_area_id');
CREATE INDEX `idx_area_id` ON `nodes` (`area_id`);

-- locations.tow_truck_id にインデックス
CALL DropIndexIfExists ( 'locations', 'idx_tow_truck_id_timestamp' );
CREATE INDEX `idx_tow_truck_id_timestamp` ON `locations` (`tow_truck_id`, `timestamp` DESC);

-- tow_trucks.driver_id にインデックス
CALL DropIndexIfExists ('tow_trucks', 'idx_driver_id');
CREATE INDEX `idx_driver_id` ON `tow_trucks` (`driver_id`);

-- tow_trucks の (area_id, status, driver_id) にインデックス
CALL DropIndexIfExists (
    'tow_trucks',
    'idx_area_id_status_driver_id'
);
CREATE INDEX `idx_area_id_status_driver_id` ON `tow_trucks` (
    `area_id`,
    `status`,
    `driver_id`
);

-- sessions.session_token の prefix にインデックス
CALL DropIndexIfExists ( 'sessions', 'idx_session_token_prefix' );
CREATE INDEX `idx_session_token_prefix` ON `sessions` (`session_token` (16));