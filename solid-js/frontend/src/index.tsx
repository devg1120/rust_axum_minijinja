import { render } from 'solid-js/web';
import { createSignal, createResource, Show } from 'solid-js';
import './index.css'

// Axum APIからデータを取得するフェッチャー関数
const fetchGreeting = async (name: string) => {
  if (!name) return null;
  const res = await fetch(`/api/greet/${encodeURIComponent(name)}`);
  if (!res.ok) throw new Error('APIの呼び出しに失敗しました');
  const data = await res.json();
  return data.message; // Rust側の `GreetResponse.message`
};

function App() {
  // ユーザーの入力値を管理するシグナル
  const [name, setName] = createSignal('World');

  // 入力値（name）の変化を検知して自動でAPIを叩くリソース
  const [greeting] = createResource(name, fetchGreeting);

  return (
    <div style={{ padding: '20px', 'font-family': 'sans-serif' }}>
      <h1 class="box">GS TEST CLASS</h1>

      <h1 style={{ color: '#2c4f7c' }}>SolidJS + Axum Frontend</h1>
      
      <div style={{ 'margin-bottom': '15px' }}>
        <label for="name-input" style={{ 'margin-right': '10px' }}>名前を入力してください:</label>
        <input
          id="name-input"
          type="text"
          value={name()}
          onInput={(e) => setName(e.currentTarget.value)}
          style={{ padding: '5px', 'font-size': '16px' }}
        />
      </div>

      <div style={{
        background: '#f4f6f9',
        padding: '15px',
        'border-left': '4px solid #2c4f7c',
        'margin-top': '20px'
      }}>
        {/* API通信の状態（ローディング、エラー、成功）に応じたUIの切り替え */}
        <Show when={!greeting.loading} fallback={<p>通信中...</p>}>
          <Show when={!greeting.error} fallback={<p style={{ color: 'red' }}>エラー: {greeting.error().message}</p>}>
            <p style={{ 'font-size': '18px', margin: 0 }}>
              <strong>APIからのレスポンス:</strong> {greeting()}
            </p>
          </Show>
        </Show>
      </div>
    </div>
  );
}

// HTMLの #root 要素にSolidJSアプリケーションをマウント
const root = document.getElementById('root');
if (root) {
  render(() => <App />, root);
} else {
  console.error("マウント先の #root 要素が見つかりませんでした。");
}
