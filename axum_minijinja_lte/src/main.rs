use axum::{
    extract::{Path, State},
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use minijinja::{context, Environment, Value};
use std::sync::Arc;
use tower_http::services::ServeDir;

// アプリケーション状態の共有構造体
struct AppState {
    jinja: Environment<'static>,
}

#[tokio::main]
async fn main() {

// 1. MiniJinja の初期化と設定
let mut jinja_env = Environment::new();
jinja_env.set_loader(minijinja::path_loader("templates"));

// メソッド名を `add_function` に修正
jinja_env.add_function("url_for", |endpoint: String, args: minijinja::value::Rest<Value>| {
    if endpoint == "static" {
        if let Some(kwargs) = args.0.last() {
            if let Some(filename) = kwargs.get_attr("filename").ok() {
                return Ok(format!("/static/{}", filename));
            }
        }
    }
    Ok(format!("/{}", endpoint))
});

    let shared_state = Arc::new(AppState { jinja: jinja_env });

    // 2. ルーターの構築
    let app = Router::new()
        // トップページ (FastAPIの @app.get('/') に相当)
        .route("/", get(index_root))
        // すべてのネストパスをキャッチ (FastAPIの @app.get('/{path:path}') に相当)
        .route("/*path", get(index_path))
        // 静的ファイルフォルダのマウント (FastAPIの app.mount("/static", ...) に相当)
        .nest_service("/static", ServeDir::new("static"))
        .with_state(shared_state);

    // 3. サーバー起動
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("サーバーが起動しました: http://localhost:8000");
    axum::serve(listener, app).await.unwrap();
}

// 共通のレンダリング・ヘルパー関数 (FastAPIの def render(...) に相当)
// テンプレートが見つからない（TemplateNotFound）場合は 404 画面とステータスコードを返します
fn render_template(state: &AppState, template_name: &str, segment: &str) -> Response {
    match state.jinja.get_template(template_name) {
        Ok(tmpl) => {
            // コンテキスト（Jinjaに渡す変数）を作成
            let ctx = context! {
                segment => segment,
                // 注: Axum では Request オブジェクトそのものをテンプレートに渡すのは難しいため、
                // 必要な情報（URIやセグメント）だけを分解してコンテキストに含めます。
            };
            match tmpl.render(ctx) {
                Ok(html_content) => Html(html_content).into_response(),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Template Render Error").into_response(),
            }
        }
        Err(err) if err.kind() == minijinja::ErrorKind::TemplateNotFound => {
            // ファイルが見つからなければ 404 ページを表示
            eprintln!("Template not found: {}, falling back to 404", template_name);
            fallback_404(state, segment)
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response(),
    }
}

// 404 画面をレンダリングするヘルパー
fn fallback_404(state: &AppState, segment: &str) -> Response {
    if let Ok(tmpl) = state.jinja.get_template("home/page-404.html") {
        if let Ok(html_content) = tmpl.render(context! { segment => segment }) {
            return (StatusCode::NOT_FOUND, Html(html_content)).into_response();
        }
    }
    (StatusCode::NOT_FOUND, "Page Not Found").into_response()
}

// パスから末尾のセグメント（識別子）を取得するヘルパー (FastAPIの get_segment に相当)
fn get_segment(path: &str) -> &str {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        "index"
    } else {
        trimmed.split('/').last().unwrap_or("index")
    }
}

// 1. トップページ（/）のハンドラ
async fn index_root(State(state): State<Arc<AppState>>, uri: Uri) -> impl IntoResponse {
    let segment = get_segment(uri.path());
    render_template(&state, "home/index.html", segment)
}

// 2. 動的なパス（/*path）のハンドラ
async fn index_path(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let segment = get_segment(&path);
    // ユーザーが入力したパスに合うHTMLを templates/home/ 内から探して表示
    let template_name = format!("home/{}", path);
    render_template(&state, &template_name, segment)
}
