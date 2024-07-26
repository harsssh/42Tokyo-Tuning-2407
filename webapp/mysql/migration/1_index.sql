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