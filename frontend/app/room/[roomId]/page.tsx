'use client';

import { useAuth } from '@/hooks/useAuth';
// URLパラメータを受け取るためのフック
import { useParams } from 'next/navigation'; 

export default function RoomPage() {
  const { roomId } = useParams(); // URLの [roomId] 部分が取れる！
  const { user } = useAuth();

  return (
    <div className="min-h-screen flex flex-col">
      <header className="bg-white border-b p-4 flex justify-between items-center shadow-sm">
        <h1 className="font-bold text-lg">Room: {roomId}</h1>
        <div className="text-sm text-gray-600">
          参加者: {user?.displayName}
        </div>
      </header>

      <main className="flex-1 p-4 overflow-y-auto bg-slate-100">
        <div className="text-center text-gray-400 mt-10">
          ここがチャットエリアになります！<br/>
          WebSocketをつなぐぞー！🚀
        </div>
      </main>

      <footer className="bg-white p-4 border-t">
        <div className="flex gap-2">
          <input 
            type="text" 
            className="flex-1 border rounded p-2" 
            placeholder="メッセージを入力..." 
          />
          <button className="bg-blue-500 text-white px-4 py-2 rounded">
            送信
          </button>
        </div>
      </footer>
    </div>
  );
}