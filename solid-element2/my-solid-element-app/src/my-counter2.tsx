import { customElement } from 'solid-element';

// カスタムエレメントの定義
customElement(
  'my-counter2', // HTMLタグ名として使用
  { count: 0 }, // デフォルトのProps (属性)
  (props, { element }) => {
    // SolidJSのシンタックスでコンポーネントを記述
    return (
      <div>
        <p>my-counter2 カウント: {props.count}</p>
        <button onClick={() => element.setAttribute('count', String(Number(props.count) + 1))}>
          カウントアップ
        </button>
      </div>
    );
  }
);

