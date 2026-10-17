use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::{get, get_service},
    Json, Router,
};
use minijinja::{context, Environment};
use serde::Serialize;
use std::sync::Arc;
use tower_http::services::ServeDir;

// MiniJinjaの環境をスレッド間で共有するための共通状態
struct AppState {
    jinja_env: Environment<'static>,
}

#[derive(Serialize)]
struct APIResponse {
    message: String,
}

#[tokio::main]
async fn main() {
    // 1. MiniJinjaの設定（templatesフォルダから読み込み）
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader("templates"));

    let shared_state = Arc::new(AppState { jinja_env: env });

    // 2. ルーターの定義
    let app = Router::new()
        // ルートパス：MiniJinjaで初期HTMLを返す
        .route("/", get(render_index))
        // API：SolidJSから非同期で呼ばれるエンドポイント
//        .route("/api/greet/:name", get(api_greet))



// ⭕ 修正後 (Axum v0.8〜 新構文)
.route("/api/greet/{name}", get(api_greet))



        // スタティックファイル：ViteがビルドしたJS/CSSを配信 (frontend/dist/assets)
        .nest_service(
            "/assets",
            //get_service(ServeDir::new("frontend/dist/assets")),
            get_service(ServeDir::new("my-solid-element-app/dist")),
        )
        .with_state(shared_state);

    // 3. サーバーの起動
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("🚀 Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

// 初期HTMLをレンダリングするハンドラー
async fn render_index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let tmpl = state.jinja_env.get_template("index.html").unwrap();
    
    // サーバーサイドから初期データをテンプレートに渡すことも可能
    let html_content = tmpl
        .render(context!(
            title => "Axum + MiniJinja + SolidJS"
        ))
        .unwrap();

    Html(html_content)
}

// フロントエンド（SolidJS）から呼び出されるAPIハンドラー
async fn api_greet(Path(name): Path<String>) -> impl IntoResponse {
    let response = APIResponse {
        message: format!("Hello, {}! from Axum API", name),
    };
    Json(response)
}
