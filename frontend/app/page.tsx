'use client';

import { useAuth } from '@/hooks/useAuth';
import { useRouter } from 'next/navigation';
import { useState } from 'react';

export default function MenuPage() {
  const { user, logout } = useAuth();
  const router = useRouter();
  const [roomIdInput, setRoomIdInput] = useState('');
  
  // 部屋に参加 (学生)
  const handleJoin = () => {
    if (!roomIdInput) return;
    router.push(`/room/${roomIdInput}`);
  };

  // 部屋を作成 (先生)
  const handleCreate = () => {
    // 本来はAPIを叩いてIDを発行するけど、一旦ランダムIDでモック
    const newRoomId = crypto.randomUUID(); 
    router.push(`/room/${newRoomId}`);
  };

  if (!user) return null; // AuthGuardが処理するけど念のため

  return (
    <div className="min-h-screen bg-gray-50 p-8">
      {/* ヘッダー部分：ユーザー情報 */}
      <header className="flex justify-between items-center mb-12">
        <h1 className="text-3xl font-bold text-gray-800">AXON</h1>
        <div className="flex items-center gap-4">
          <div className="text-right hidden sm:block">
            <p className="font-semibold">{user.displayName}</p>
            <p className="text-xs text-gray-500">{user.email}</p>
          </div>
          {/* アバター表示！ */}
          {user.photoURL && (
            <img 
              src={user.photoURL} 
              alt="Avatar" 
              className="w-10 h-10 rounded-full border border-gray-300"
            />
          )}
          <button onClick={logout} className="text-sm text-red-500 hover:underline">
            ログアウト
          </button>
        </div>
      </header>

      {/* メインメニュー */}
      <main className="max-w-4xl mx-auto grid grid-cols-1 md:grid-cols-2 gap-8">
        
        {/* ROOMに参加カード */}
        <div className="bg-white p-8 rounded-xl shadow-lg hover:shadow-xl transition border-l-4 border-green-500">
          <h2 className="text-2xl font-bold mb-4 text-gray-700">ROOMに参加</h2>
          <p className="text-gray-500 mb-6">先生から共有されたIDを入力してください</p>
          <div className="flex gap-2">
            <input 
              type="text" 
              placeholder="ルームIDを入力"
              value={roomIdInput}
              onChange={(e) => setRoomIdInput(e.target.value)}
              className="flex-1 border p-2 rounded focus:outline-none focus:ring-2 focus:ring-green-400"
            />
            <button 
              onClick={handleJoin}
              className="bg-green-500 text-white px-6 py-2 rounded hover:bg-green-600 font-bold"
            >
              参加
            </button>
          </div>
        </div>

        {/* ROOMを作成カード */}
        <div className="bg-white p-8 rounded-xl shadow-lg hover:shadow-xl transition border-l-4 border-blue-500">
          <h2 className="text-2xl font-bold mb-4 text-gray-700">ROOMを作成</h2>
          <p className="text-gray-500 mb-6">新しい授業ルームを作成します（教員用）</p>
          <button 
            onClick={handleCreate}
            className="w-full bg-blue-500 text-white py-3 rounded-lg hover:bg-blue-600 font-bold text-lg"
          >
            + 新規ルーム作成
          </button>
        </div>

      </main>
    </div>
  );
}