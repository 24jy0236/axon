// frontend/app/page.tsx
'use client';

import { useAuth } from '@/hooks/useAuth';

export default function Home() {
  // カスタムフックから必要な機能を取り出すだけ！
  const { user, token, error, login, logout, loading } = useAuth();

  if (loading) return <main>読み込み中...</main>;

  return (
    <main style={{ padding: '2rem' }}>
      <h1>リアルタイムチャット "AXON"</h1>
      
      {!user ? (
        <button onClick={login}>Googleでログイン</button>
      ) : (
        <div>
          <h2>ようこそ、{user.displayName} さん！</h2>
          <button onClick={logout}>ログアウト</button>
          
          <div style={{ marginTop: '20px', wordBreak: 'break-all' }}>
            <h3>Token:</h3>
            <p>{token}</p>
          </div>
        </div>
      )}

      {error && <p style={{ color: 'red' }}>エラー: {error}</p>}
    </main>
  );
}