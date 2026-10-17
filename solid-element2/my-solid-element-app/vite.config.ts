/*

import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'

export default defineConfig({
  plugins: [solid()],
})

*/

import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solidPlugin()],
  build: {
    lib: {
      // エントリーポイント（カスタムエレメントを登録しているファイル）
      entry: 'src/index.tsx', 
      // 出力されるグローバル変数名（IIFE/UMD形式用）
      name: 'MyCounter',
      // 出力されるファイル名
      fileName: (format) => `my-counter.${format}.js`,
      // 出力形式（複数の環境で使えるように es と umd、または iife を指定）
      formats: ['es', 'umd'], 
    },
    // 必要に応じて、Reactのように外部化したいライブラリがあれば指定（Solidは基本同梱でOKです）
    rollupOptions: {
      external: [], 
    },
  },
});

