use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

// データベースのレコードに対応する構造体
#[derive(Serialize, sqlx::FromRow)]
struct Todo {
    id: i64,
    title: String,
}

// フォームから送信されるデータ用
#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

// 複数のハンドラで共有するアプリケーション状態
struct AppState {
    db: SqlitePool,
    jinja: Environment<'static>,
}

#[tokio::main]
async fn main() {
    // 1. インメモリの SQLite データベースプールを作成
    let db_pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // サンプル用のテーブル作成と初期データ挿入
    sqlx::query(
        "CREATE TABLE todos (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL);"
    )
    .execute(&db_pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO todos (title) VALUES ('Rustの勉強'), ('Axumアプリの開発');")
        .execute(&db_pool)
        .await
        .unwrap();

    // 2. MiniJinja 環境の設定（templatesディレクトリから自動ロード）
    //let mut jinja_env = Environment::new();
    //jinja_env.set_source(minijinja::Source::from_path("templates"));
    let mut jinja_env = Environment::new();
    jinja_env.set_loader(minijinja::path_loader("templates"));


    // 3. アプリケーション状態を Arc で包む
    let shared_state = Arc::new(AppState {
        db: db_pool,
        jinja: jinja_env,
    });

    // 4. ルーターの構築
    let app = Router::new()
        .route("/", get(show_todos))
        .route("/add", post(add_todo))
        .with_state(shared_state);

    // 5. サーバーの起動
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("サーバー起動中: http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

// タスク一覧を表示するハンドラ
async fn show_todos(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // DB からタスク一覧を取得
    let todos = sqlx::query_as::<_, Todo>("SELECT id, title FROM todos")
        .fetch_all(&state.db)
        .await
        .unwrap();

    // テンプレートをレンダリング
    let tmpl = state.jinja.get_template("index.html").unwrap();
    let html_content = tmpl.render(context!(todos => todos)).unwrap();

    Html(html_content)
}

// 新しいタスクを追加するハンドラ
async fn add_todo(
    State(state): State<Arc<AppState>>,
    Form(form): Form<CreateTodo>,
) -> impl IntoResponse {
    // DB にタスクを挿入
    sqlx::query("INSERT INTO todos (title) VALUES (?)")
        .bind(form.title)
        .execute(&state.db)
        .await
        .unwrap();

    // 完了後はトップページにリダイレクト
    Redirect::to("/")
}
