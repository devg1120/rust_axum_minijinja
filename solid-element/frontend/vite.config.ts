
import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'


export default defineConfig({
  plugins: [solid()],
  build: {
    rollupOptions: {
      output: {
        // 💡 ハッシュ値を付けずに「index.js」「index.css」として出力する設定
        entryFileNames: 'assets/[name].js',
        chunkFileNames: 'assets/[name].js',
        assetFileNames: 'assets/[name].[ext]',
      },
    },
  },
});


