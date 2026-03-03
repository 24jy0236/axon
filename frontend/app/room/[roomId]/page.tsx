'use client';

import { useAuth } from '@/hooks/useAuth';
import { useEffect, useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { joinRoom } from '@/lib/api/rooms';
import type { JoinRoomResponse } from '@/types/generated/join_room_response';

export default function RoomPage() {
  const { user, token, loading: authLoading } = useAuth();
  const params = useParams();
  const router = useRouter();

  // URLの /room/[roomId] の部分。今回はこれがSlugになります
  const slug = params.roomId as string;

  const [roomData, setRoomData] = useState<JoinRoomResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // 認証情報のロード中は何もしない
    if (authLoading) return;

    // ログインしていなければトップページへ弾く
    if (!user || !token) {
      router.push('/');
      return;
    }

    const fetchRoom = async () => {
      try {
        const data = await joinRoom(token, slug);
        setRoomData(data);
      } catch (err: unknown) {
        if (err instanceof Error && err.message === 'RoomNotFound') {
          setError('指定されたルームが見つかりません。URLが正しいか確認してください。');
        } else {
          setError('ルームの参加に失敗しました。');
        }
      } finally {
        setLoading(false);
      }
    };

    fetchRoom();
  }, [slug, token, user, authLoading, router]);

  // ローディング中の表示
  if (authLoading || loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-gray-500 font-bold animate-pulse">ルームに参加中...</div>
      </div>
    );
  }

  // エラー時の表示 (404など)
  if (error) {
    return (
      <div className="min-h-screen flex flex-col items-center justify-center bg-gray-50">
        <h1 className="text-2xl font-bold text-red-500 mb-4">{error}</h1>
        <button
          onClick={() => router.push('/')}
          className="bg-blue-500 text-white px-6 py-2 rounded-lg hover:bg-blue-600 font-bold"
        >
          メニューに戻る
        </button>
      </div>
    );
  }

  if (!roomData) return null;

  return (
    <div className="min-h-screen flex flex-col h-screen bg-gray-50">
      {/* 🔴 ヘッダー領域 */}
      <header className="bg-white shadow-sm p-4 flex justify-between items-center border-b">
        <div>
          <h1 className="text-xl font-bold text-gray-800">{roomData.room.name}</h1>
          <p className="text-sm text-gray-500 mt-1">
            ID: <span className="font-mono bg-gray-100 px-1 rounded">{roomData.room.slug}</span>
            <span className="mx-2">|</span>
            権限:
            <span className={`ml-1 font-bold ${roomData.role === 'Teacher' ? 'text-red-500' : 'text-blue-500'}`}>
              {roomData.role === 'Teacher' ? '教員' : '学生'}
            </span>
          </p>
        </div>
        <button
          onClick={() => router.push('/')}
          className="text-sm text-gray-500 hover:text-gray-800 border px-3 py-1 rounded"
        >
          退出
        </button>
      </header>

      {/* 🔴 チャット表示領域（仮） */}
      <main className="flex-1 overflow-y-auto p-4">
        <div className="flex flex-col items-center justify-center h-full text-gray-400">
          <p className="mb-2">🎉 ルームへの参加が完了しました！</p>
          <p>（ここにWebSocketによるリアルタイムチャットが実装されます）</p>
        </div>
      </main>

      {/* 🔴 メッセージ入力領域（仮） */}
      <footer className="bg-white p-4 border-t">
        <div className="max-w-4xl mx-auto flex gap-2">
          <input
            type="text"
            placeholder="メッセージを入力... (現在はまだ送信できません)"
            className="flex-1 border border-gray-300 rounded-lg p-3 focus:outline-none focus:ring-2 focus:ring-blue-400 bg-gray-50"
            disabled
          />
          <button
            className="bg-blue-500 text-white px-6 py-2 rounded-lg font-bold disabled:bg-gray-400 cursor-not-allowed"
            disabled
          >
            送信
          </button>
        </div>
      </footer>
    </div>
  );
}