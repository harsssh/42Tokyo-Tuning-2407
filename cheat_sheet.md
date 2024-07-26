サンプルコードは動作確認していません
## Rust
### コマンドを呼び出さない
別プロセスで処理するオーバーヘッドが大きいのでだめ。
アプリケーションで処理する。

ダメな例
`sh`, `echo`, `shasum` のコマンドが実行される
```rust
use std::process::Command;

let text = "Hello, world!";
let output = Command::new("sh")
    .arg("-c")
    .arg(format!("echo -n '{}' | shasum -a 256", text))
    .output()
    .expect("failed to execute shasum");

let sha_sum = String::from_utf8_lossy(&output.stdout)
    .split_whitespace()
    .next()
    .unwrap();
println!("SHA-256: {}", sha_sum);
```

いい例
```rust
use sha2::{Sha256, Digest};

let text = "Hello, world!";
let hash = Sha256::digest(text.as_bytes());
println!("SHA-256: {:x}", hash);
```

### 直列より並列
ダメな例
`result1` が終わってから、`result2` を実行
```rust
let query1 = sqlx::query!("SELECT * FROM table1").fetch_all(pool.get_ref());
let query2 = sqlx::query!("SELECT * FROM table2").fetch_all(pool.get_ref());

let result1 = query1.await;
let result2 = query2.await;
```

いい例
同時に実行して、`tokio::join!` で合流
```rust
let query1 = sqlx::query!("SELECT * FROM table1").fetch_all(pool.get_ref());
let query2 = sqlx::query!("SELECT * FROM table2").fetch_all(pool.get_ref());

let (result1, result2) = tokio::join!(query1, query2);
```

### Vec への N 回 push
ダメな例
```rust
let mut vec = Vec::new();
for i in 0..1000 {
    vec.push(i);
}
```

先にメモリを確保
```rust
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i);
}
```

これに関してはもっと短くかける
```rust
let vec: Vec<_> = (0..1000).collect();
```

### for よりイテレータ
ダメな例
```rust
let vec = vec![1, 2, 3, 4, 5];
let mut sum = 0;
for i in &vec {
    sum += i;
}
```

いい例
イテレータは効率的に反復処理をする
```rust
let vec = vec![1, 2, 3, 4, 5];
let sum: i32 = vec.iter().sum();
```

### 所有権を奪える場合は into_iter を使う
`iter()` は参照を生成する。データアクセスのオーバーヘッドが生じる。
イテレータとして消費していい場合は、`into_iter()` を使う。

```rust
let vec = vec![1, 2, 3, 4, 5];
for val in vec.into_iter() {
    println!("{}", val);
}
```

### 反復処理の並列化
```toml
[dependencies]
rayon = "1.10"
```

```rust
use rayon::prelude::*;
fn sum_of_squares(input: &[i32]) -> i32 {
    input.par_iter() // <-- just change that!
         .map(|&i| i * i)
         .sum()
}
```

### iter のメソッド
便利なものがいっぱい
ドキュメントを読んで

```cardlink
url: https://doc.rust-lang.org/std/iter/trait.Iterator.html
title: "Iterator in std::iter - Rust"
description: "A trait for dealing with iterators."
host: doc.rust-lang.org
favicon: ../../static.files/favicon-2c020d218678b618.svg
```

Qiita
```cardlink
url: https://qiita.com/lo48576/items/34887794c146042aebf1
title: "Rustのイテレータの網羅的かつ大雑把な紹介 - Qiita"
description: "はじめに正直なところ、公式ドキュメント std::iter::Iterator - Rust を読めば終了、この記事は不要、という感じはある。しかしこのページ、例とかは豊富だし良いのだが、関数の…"
host: qiita.com
favicon: https://cdn.qiita.com/assets/favicons/public/production-c620d3e403342b1022967ba5e3db1aaa.ico
image: https://qiita-user-contents.imgix.net/https%3A%2F%2Fcdn.qiita.com%2Fassets%2Fpublic%2Farticle-ogp-background-412672c5f0600ab9a64263b751f1bc81.png?ixlib=rb-4.0.0&w=1200&mark64=aHR0cHM6Ly9xaWl0YS11c2VyLWNvbnRlbnRzLmltZ2l4Lm5ldC9-dGV4dD9peGxpYj1yYi00LjAuMCZ3PTk3MiZoPTM3OCZ0eHQ9UnVzdCVFMyU4MSVBRSVFMyU4MiVBNCVFMyU4MyU4NiVFMyU4MyVBQyVFMyU4MyVCQyVFMyU4MiVCRiVFMyU4MSVBRSVFNyVCNiVCMiVFNyVCRSU4NSVFNyU5QSU4NCVFMyU4MSU4QiVFMyU4MSVBNCVFNSVBNCVBNyVFOSU5QiU5MSVFNiU4QSU4QSVFMyU4MSVBQSVFNyVCNCVCOSVFNCVCQiU4QiZ0eHQtYWxpZ249bGVmdCUyQ3RvcCZ0eHQtY29sb3I9JTIzMjEyMTIxJnR4dC1mb250PUhpcmFnaW5vJTIwU2FucyUyMFc2JnR4dC1zaXplPTU2JnM9NjcxMzVkZGI0YzVkMmFmNWUwNjk2MTkzNDUwZmY4ZmY&mark-x=142&mark-y=57&blend64=aHR0cHM6Ly9xaWl0YS11c2VyLWNvbnRlbnRzLmltZ2l4Lm5ldC9-dGV4dD9peGxpYj1yYi00LjAuMCZoPTc2Jnc9NzcwJnR4dD0lNDBsbzQ4NTc2JnR4dC1jb2xvcj0lMjMyMTIxMjEmdHh0LWZvbnQ9SGlyYWdpbm8lMjBTYW5zJTIwVzYmdHh0LXNpemU9MzYmdHh0LWFsaWduPWxlZnQlMkN0b3Amcz1iNDFiYzU3ZjJkYWY2NTVkNjNkNjY3YzM1MjZjMTA1Zg&blend-x=142&blend-y=486&blend-mode=normal&s=418cfec4184b5e20ef40e940bc140cf6
```

### Moka でキャッシュ
高速なインメモリキャッシュライブラリ
```cardlink
url: https://github.com/moka-rs/moka
title: "GitHub - moka-rs/moka: A high performance concurrent caching library for Rust"
description: "A high performance concurrent caching library for Rust - moka-rs/moka"
host: github.com
favicon: https://github.githubassets.com/favicons/favicon.svg
image: https://opengraph.githubassets.com/e62b21b6b7b5b5e626db18f88be4089e45496a1cc3c340ff56caa788cfdb7d5c/moka-rs/moka
```

```rust
use actix_web::{web, App, HttpServer, HttpResponse};
use moka::future::Cache;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            // Cache を登録
            .data(AppState { cache: Cache::new(100) })
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn index(state: web::Data<AppState>) -> HttpResponse {
    // state から Cache を取り出して利用
    // あればキャッシュを利用し、なければクロージャーを実行して保存
    let value = state.cache.get_with("example_key", || {
        "Hello, Cache!".to_string()
    }).await;

    HttpResponse::Ok().body(value)
}
```

## N+1 問題
1 回のクエリ + N 回ループ内でクエリ

クエリの実行にはパースや実行計画作成などのオーバーヘッドがあるため、遅くなる。
また、ロジックによっては無駄な行の取得をしていることもあり、非効率。
### JOIN で 1 回で取得
ブログサービスで「あるユーザーの特定のタグが付いた投稿を全て取得する」場合、naive に実装すると以下のようになる。
```rust
#[derive(Debug)]
struct Post {
    id: i32,
    title: String,
    user_id: i32
}

async fn get_tagged_posts_by_user(pool: &MySqlPool, user_id: i32) -> Result<Vec<Post>, sqlx::Error> {
    // ユーザーの全投稿の取得
    let posts = fetch_posts_by_user_id(user_id).await?;

    let mut tagged_posts = Vec::new();

    for post in posts {
        // 各投稿のタグを個別に取得
        let tags = fetch_tag_by_post_id(post.tag_id).await?;

        // 投稿にタグが存在する場合のみ追加
        if !tags.is_empty() {
            tagged_posts.push(post);
        }
    }

    Ok(tagged_posts)
}
```

クエリをうまく書けば、1 回の実行で済み、取得する行数も少ない。
```rust
#[derive(Debug)]
struct Post {
    id: i32,
    title: String,
    user_id: i32
}

async fn get_tagged_posts_by_user(pool: &MySqlPool, user_id: i32, tag: &str) -> Result<Vec<Post>, sqlx::Error> {
    let tagged_posts = sqlx::query_as!(
        Post,
        r#"
        SELECT p.id, p.title, p.user_id
        FROM posts p
        JOIN post_tags pt ON p.id = pt.post_id
        JOIN tags t ON pt.tag_id = t.id
        WHERE p.user_id = ? AND t.name = ?
        "#,
        user_id,
        tag
    )
    .fetch_all(pool)
    .await?;

    Ok(tagged_posts)
}
```

### レコード数が多い JOIN は避けて、IN を使う
全ユーザーの全投稿を取得する例

ダメな例
```rust
use sqlx::mysql::MySqlPool;

async fn fetch_users_with_posts(pool: &MySqlPool) -> Result<Vec<(User, Vec<Post>)>, sqlx::Error> {
    let users = fetch_users().await?;

    let mut user_posts = Vec::new();
    for user in users {
        let posts = fetch_user_posts(user.id).await?;
        user_posts.push((user, posts));
    }

    Ok(user_posts)
}
```

JOIN すると N+1 を解消できるが、ユーザー数×投稿数の行が返ってくるので、逆に遅くなることがある。

この場合、結合処理はアプリケーションで行う。
```rust
use sqlx::{mysql::MySqlPool, query_builder::QueryBuilder};
use std::collections::HashMap;

async fn fetch_users_with_posts(pool: &MySqlPool) -> Result<Vec<(User, Vec<Post>)>, sqlx::Error> {
    let users: Vec<User> = fetch_users().await?;

    // IN 句を生成
    let mut query_builder = QueryBuilder::new("SELECT id, title, user_id FROM posts WHERE user_id IN (");
    let mut separated = query_builder.separated(", ");
    let user_ids: Vec<i32> = users.iter().map(|u| u.id).collect();
    for id in user_ids.iter() {
      separated.push_bind(id);
    }
    separated.push_unseparated(") ");

    let posts: Vec<Post> = query_builder.build().fetch_all(pool).await?;

    // user_id -> Post の HashMap
    let mut user_posts_map: HashMap<i32, Vec<Post>> = HashMap::new();
    for post in posts {
        user_posts_map.entry(post.user_id).or_default().push(post);
    }

    let mut user_posts = Vec::new();
    for user in users {
        let related_posts = user_posts_map.remove(&user.id).unwrap_or_default();
        user_posts.push((user, related_posts));
    }

    Ok(user_posts)
}
```

### キャッシュを使う
キャッシュに対する N+1 は十分早い場合が多い。
クエリ自体の改善が難しい場合におすすめ。

```rust
use moka::sync::Cache;

fn fetch_users_with_posts(cache: &Cache<i32, Vec<Post>>) -> Vec<(User, Vec<Post>)> {
    let users = fetch_users();
    let mut user_posts = Vec::new();

    for user in users {
        let posts = cache.get_with(user.id, || fetch_posts_by_user_id(user.id));
        user_posts.push((user, posts));
    }

    user_posts
}
```

### まとめて INSERT
for ループで 1 件ずつ INSERT するのは非効率

```rust
let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
    "INSERT INTO users(id, username, email, password) "
);

// 以下の形式のクエリになる
// INSERT INTO users(id, username, email, password) VALUES (?, ?, ?, ?), (?, ?, ?, ?)
query_builder.push_values(users.into_iter(), |mut b, user| {
    b.push_bind(user.id)
        .push_bind(user.username)
        .push_bind(user.email)
        .push_bind(user.password);
});
```

### まとめて INSERT (INSERT ... SELECT)
`SELECT`, `INSERT` の 2 回に分けてクエリが発行されている
```rust
// users テーブルから条件に合うユーザーを取得
let condition = "some_condition";
let selected_users = sqlx::query_as!(
    User,
    "SELECT id, username, email, password FROM users WHERE some_column = ?",
    condition
)
.fetch_all(&pool)
.await?;

// 取得したユーザーを別のテーブルに INSERT
for user in selected_users {
    sqlx::query!(
        "INSERT INTO other_table (id, username, email, password) VALUES (?, ?, ?, ?)",
        user.id,
        user.username,
        user.email,
        user.password
    )
    .execute(&pool)
    .await?;
}
```

これは 1 つのクエリに直せる
```sql
INSERT INTO other_table (id, username, email, password)
SELECT id, username, email, password
FROM users
WHERE some_column = 'some_condition'
```

### まとめて UPDATE
`UPDATE` 自体にまとめて更新する機能はない。
`INSERT` を使って、既に存在すれば更新、と書く
```sql
INSERT INTO
	example (`id`, `name`)
VALUES
	(1, "Isac"),
	(2, "James")
ON DUPLICATE KEY UPDATE `name` = VALUES(`name`);
```

完全に更新目的で、カラムが欠けてるとだめそう。
## SQL
### 存在確認
余計な行を取得しない方が早い
```sql
SELECT 1 FROM users WHERE id = '1';
```

`WHERE` で使う場合は、`EXISTS` がある
```sql
SELECT * FROM users u
WHERE EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id AND o.status = 'completed');
```

### 条件が違う複数のクエリを CASE でまとめる
ダメな例
```sql
SELECT user_id, SUM(amount) AS total_sales
FROM orders
WHERE status = 'completed'
GROUP BY user_id;

SELECT user_id, SUM(amount) AS pending_sales
FROM orders
WHERE status = 'pending'
GROUP BY user_id;
```

いい例
```sql
SELECT
    user_id,
    SUM(CASE WHEN status = 'completed' THEN amount ELSE 0 END) AS total_sales,
    SUM(CASE WHEN status = 'pending' THEN amount ELSE 0 END) AS pending_sales
FROM orders
GROUP BY user_id;
```

他の例
```sql
SELECT
    student_id,
    score,
    CASE
        WHEN score >= 80 THEN 'A'
        WHEN score >= 60 THEN 'B'
        ELSE 'C'
    END AS grade
FROM exams;
```
### M 番目から N 件取得
ダメな例
無駄にレコードを取得するので遅い
```rust
use sqlx::{MySql, Pool};

async fn fetch_with_limit_offset(pool: &Pool<MySql>, limit: i64, offset: i64) -> Result<Vec<Item>, sqlx::Error> {
    let items: Vec<Item> = sqlx::query_as::<_, Item>("SELECT id, value FROM items")
        .fetch_all(pool)
        .await?;

    Ok(items.into_iter().skip(offset).take(limit).collect())
}
```

クエリで先に絞り込む
```rust
use sqlx::{MySql, Pool};

async fn fetch_with_limit_offset(pool: &Pool<MySql>, limit: i64, offset: i64) -> Result<Vec<Item>, sqlx::Error> {
    let items: Vec<Item> = sqlx::query_as!(
        Item,
        "SELECT id, value FROM items LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(items)
}
```

### ランキング
数字を飛ばしたくない場合は、`DENSE_RANK()` を使う。
```sql
SELECT
    id,
    value,
    RANK() OVER (ORDER BY score DESC) AS rank
FROM
    items;
```

実行例
```
id | value   | rank
---|---------|-----
 1 | Item A |  1
 3 | Item C |  1
 2 | Item B |  3
 4 | Item D |  4
```
### カテゴリごとのランキング
`PARTITION BY` でカテゴリごとに切り分けてから、ランキングを付ける。
```sql
SELECT
    category,
    id,
    value,
    RANK() OVER (PARTITION BY category ORDER BY score DESC) AS rank
FROM
    items;
```

実行例
```
category | id | value   | rank
---------|----|---------|-----
A        |  1 | Item A |  1
A        |  3 | Item C |  1
A        |  2 | Item B |  3
B        |  4 | Item D |  1
B        |  5 | Item E |  2
```

### A に対する最新の B
愚直にサブクエリで
```sql
SELECT a.*, b.*
FROM A AS a
JOIN (
    SELECT b1.*
    FROM B AS b1
    JOIN (
        SELECT a_id, MAX(updated_at) AS max_updated
        FROM B
        GROUP BY a_id
    ) AS b2 ON b1.a_id = b2.a_id AND b1.updated_at = b2.max_updated
) AS b ON a.id = b.a_id;
```

window 関数を使うと早い場合がある
```sql
SELECT a.*, b.*
FROM A AS a
LEFT JOIN (
    SELECT *, ROW_NUMBER() OVER (PARTITION BY a_id ORDER BY updated_at DESC) AS rn
    FROM B
) AS b ON a.id = b.a_id AND b.rn = 1;
```

LATERAL を使うと多分最速
サブクエリを修飾して、サブクエリの外の値を参照できる
`WHERE B.a_id = a.id` の部分がそう
```sql
SELECT a.*, b.*
FROM A AS a
LEFT JOIN LATERAL (
    SELECT *
    FROM B
    WHERE B.a_id = a.id
    ORDER BY updated_at DESC
    LIMIT 1
) AS b ON true;
```
## DB その他
### 降順インデックス
MySQL では、インデックスのソート順を逆にできる
```sql
CREATE INDEX idx_name ON table_name (column_name DESC);
```
### 範囲検索をするカラムは、複合インデックスの後ろに配置
```sql
SELECT id, value
FROM items
WHERE category = 'A' AND score BETWEEN 10 AND 1000000;
```

`(score, category)` のインデックスだと、`score` で絞り込んだ結果はカテゴリでソートされていない。(インデックスの構造を考えよ)
約 1000000 件のデータを走査することになる。

逆順の `(category, score)` でインデックスを作ると効率がいい。

### EXPLAIN で実行計画を確認
`\G` を付けると、横に長い出力が見やすい
```sql
EXPLAIN SELECT id, value FROM items WHERE score > 70\G
```

```
id: 1
select_type: SIMPLE
table: items
type: range
possible_keys: idx_score
key: idx_score
key_len: 4
ref: NULL
rows: 20
Extra: Using where
```

読み方
- type: 検索の種類
    - ALL: テーブル全体を走査
    - index: index 全体を走査。ALL よりはマシだが、非効率
    - range: index を使った範囲検索
    - ref: index を使った検索
    - eq_ref: PK, UK で結合
    - const: PK, UK で検索
- key: 使われる index
- Extra: 追加情報
    - Using where: WHERE でフィルタリングする
    - Using index: テーブルアクセスなしで、index からデータを取得。すごく早い
    - Using temporary: 一時テーブルを作成。遅い。`ORDER BY`, `GROUP BY` で現れがち
    - Using filesort: ディスク上でソート。すごく遅い

### EXPLAIN ANALYZE で実行計画の詳細を確認
```sql
EXPLAIN ANALYZE SELECT * FROM users u JOIN scores s ON u.id = s.user_id WHERE u.id < 100;
```

`scores` を全件走査で絞り込みしている
`scores.user_id` にインデックス (or 外部キー制約) が必要
```
Nested loop inner join  (cost=2399258.64 rows=1994337) (actual time=9.622..598.250 rows=198 loops=1)
    -> Filter: (s.user_id < 100)  (cost=205487.94 rows=1994337) (actual time=9.598..597.860 rows=198 loops=1)
        -> Table scan on s  (cost=205487.94 rows=1994337) (actual time=9.595..530.336 rows=2000000 loops=1)
    -> Single-row index lookup on u using PRIMARY (id=s.user_id)  (cost=1.00 rows=1) (actual time=0.002..0.002 rows=1 loops=198)
```

cost は秒ではない。相対値に意味がある。

### 不要なトランザクションを張らない
トランザクションはオーバーヘッドが大きい
整合性に影響を与えない部分では張らない

ダメな例
```rust
// 不要なトランザクション
let mut tx = pool.begin().await?;

let row: (String,) = sqlx::query("SELECT name FROM users WHERE id = ?")
    .bind(1)
    .fetch_one(&mut tx)
    .await?;

transaction.commit().await?;
}
```