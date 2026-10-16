import { customElement } from 'solid-element';
import { createSignal } from 'solid-js';

// 1. 通常の SolidJS コンポーネントを作成
// 引数の props には、Web Component の属性（Attributes）が自動的にマウントされます
function CounterComponent(props: { initial?: string; step?: string }) {
  // 属性値は文字列で渡るため、数値にパース
  const start = () => parseInt(props.initial || '0', 10);
  const step = () => parseInt(props.step || '1', 10);

  const [count, setCount] = createSignal(start());

  return (
    <div style={{ padding: '10px', border: '1px solid #ccc', "border-radius": '4px' }}>
      <p>Count: <strong>{count()}</strong></p>
      <button onClick={() => setCount(count() + step())}>
        +{step()}
      </button>
      <button onClick={() => setCount(count() - step())} style={{ "margin-left": '5px' }}>
        -{step()}
      </button>
    </div>
  );
}

// 2. customElement 関数を使って Web Component として登録
// 第一引数：タグ名 (必ずハイフンを含む必要があります)
// 第二引数：リアクティブに監視・同期したい属性の初期値 / 定義
// 第三引数：SolidJS のコンポーネント本体
customElement(
  'my-counter', 
  { initial: '0', step: '1' }, 
  CounterComponent
);

