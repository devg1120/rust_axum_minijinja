use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse, sse::{Event, Sse}},
    routing::get,
    Router,
};
use futures_util::{stream::{self, Stream}, SinkExt, StreamExt};
use minijinja::{context, Environment};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::broadcast;

// アプリケーションの共有状態
#[derive(Clone)]
struct AppState {
    jinja_env: Arc<Environment<'static>>,
    tx: broadcast::Sender<String>, // WebSocket用のブロードキャストチャンネル
}

#[tokio::main]
async fn main() {
    // 1. MiniJinjaの設定
    let mut env = Environment::new();
    // テンプレートファイルを登録
    env.add_template("index", include_str!("templates/index.html")).unwrap();
    // SSE用：単に時間文字列を返すだけ
    env.add_template("time_fragment", r#"{{ time }}"#).unwrap();
    // WebSocket用：受信したメッセージをそのまま表示するpタグ
    env.add_template("chat_message", r#"<p><strong>ユーザー:</strong> {{ msg }}</p>"#).unwrap();

    // 2. WebSocket用のブロードキャストチャンネル作成（100件までバッファ）
    let (tx, _) = broadcast::channel(100);

    let state = AppState {
        jinja_env: Arc::new(env),
        tx,
    };

    // 3. ルーティング定義（外部JSファイルを配信するルートは不要になりました）
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/sse", get(sse_handler))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("サーバー起動中: http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

// メインページのHTMLを返すハンドラ
async fn index_handler(State(state): State<AppState>) -> Html<String> {
    let tmpl = state.jinja_env.get_template("index").unwrap();
    let html = tmpl.render(context!()).unwrap();
    Html(html)
}

// 【SSEエンドポイント】 サーバー時刻を2秒ごとにプッシュ配信
async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
        Duration::from_secs(2),
    ))
    .map(move |_| {
        let tmpl = state.jinja_env.get_template("time_fragment").unwrap();
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        let html_content = tmpl.render(context!(time => now)).unwrap();

        // 標準の message イベントとして配信
        let event = Event::default().data(html_content);
        Ok(event)
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}

// 【WebSocketエンドポイント】 ハンドシェイクとアップグレード
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

// WebSocketの実際の通信処理
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // タスク1: 他のクライアントからのブロードキャストメッセージを受信してブラウザに送る
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // タスク2: このブラウザから送信されたメッセージを受信して全員にブロードキャストする
    let tx = state.tx.clone();
    let env = state.jinja_env.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // クライアント側からプレーンテキストで送られてくるメッセージを抽出
            if let Some(msg) = text.strip_prefix("chat_message=") {
                let decoded_msg = urlencoding::decode(msg).unwrap_or_default().into_owned();
                
                // MiniJinjaでチャットメッセージのHTML断片を生成
                let tmpl = env.get_template("chat_message").unwrap();
                let html_content = tmpl.render(context!(msg => decoded_msg)).unwrap();

                // 全員にHTMLを一斉送信
                let _ = tx.send(html_content);
            }
        }
    });

    // どちらかのタスクが終了（切断など）したら、もう片方も終了する
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
