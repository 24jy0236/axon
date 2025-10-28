'use client'; // App Routerを使う場合はこの行が必要

import { useState, useEffect } from 'react';

export default function Home() {
  const [message, setMessage] = useState('Loading...');

  useEffect(() => {
    // バックエンドAPIからデータを取得する
    fetch('http://localhost:3001/api/hello')
      .then(res => res.json())
      .then(data => {
        setMessage(data.message);
      })
      .catch(err => {
        console.error("Error fetching data:", err);
        setMessage("Failed to load message from server.");
      });
  }, []); // 空の依存配列で、コンポーネントのマウント時に一度だけ実行

  return (
    <main style={{ padding: '2rem' }}>
      <h1>リアルタイムチャットへようこそ！</h1>
      <p>
        バックエンドからのメッセージ: <strong>{message}</strong>
      </p>
    </main>
  );
}