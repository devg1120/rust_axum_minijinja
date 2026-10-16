import { customElement } from 'solid-element';
import { createSignal } from 'solid-js';

interface CounterProps {
  initial?: string;
  step?: string;
}

function CounterComponent2(
  props: CounterProps,
  // 💡 ここを { element: any } にします
  { element }: { element: any } 
) {
  const start = () => parseInt(props.initial || '0', 10);
  const step = () => parseInt(props.step || '1', 10);
  const [count, setCount] = createSignal(start());

  const handleIncrement = () => {
    const nextValue = count() + step();
    setCount(nextValue);

    // elementの実体はDOMなので問題なくdispatchEventを呼び出せます
    const event = new CustomEvent('countchange', {
      detail: { currentCount: nextValue },
      bubbles: true,
      composed: true
    });
    element.dispatchEvent(event);
  };

  return (
    <div>
    <style>{`
          p {
            margin: 0 0 10px 0;
            color: red;
          }
          button {
	     background-color: var(--counter-btn-color, #4f46e5);
            color: white;
            border: none;
            padding: 6px 12px;
            border-radius: 4px;
            cursor: pointer;
          }
          button:hover {
            background-color: yellow;
            color: blue;
          }
        `}</style>

      <p>Count: {count()}</p>
      <button onClick={handleIncrement}>+{step()}</button>
    </div>
  );
}

customElement('my-counter2', { initial: '0', step: '1' }, CounterComponent2);

