'use client';

import { useAuth } from '@/hooks/useAuth';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { createRoom } from '@/lib/api/rooms';
import type { CreateRoomRequest } from '@/types/generated/create_room_dto';

export default function MenuPage() {
  const { user, token, logout } = useAuth(); // tokenを取得
  const router = useRouter();
  
  // 参加用ステート
  const [roomIdInput, setRoomIdInput] = useState('');
  
  // 作成用ステート
  const [isCustomId, setIsCustomId] = useState(false);
  const [customIdInput, setCustomIdInput] = useState('');
  const [createError, setCreateError] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  
  // 部屋に参加 (学生)
  const handleJoin = () => {
    if (!roomIdInput) return;
    router.push(`/room/${roomIdInput}`);
  };

  // 部屋を作成 (先生)
  const handleCreate = async () => {
    if (!token) {
      setCreateError('認証情報が見つかりません。再ログインしてください。');
      return;
    }

    // ID指定ありの場合のバリデーション
    let slug: string | null = null;
    if (isCustomId) {
      if (customIdInput.length < 4 || customIdInput.length > 16 || !/^[a-zA-Z0-9]+$/.test(customIdInput)) {
        setCreateError('ROOM IDは4〜16文字の英数字で入力してください。');
        return;
      }
      // バックエンドAPIの仕様に合わせて小文字に変換
      slug = customIdInput.toLowerCase();
    }

    try {
      setIsCreating(true);
      setCreateError(''); // エラーをリセット

      // DTOに基づくペイロードの作成
      const payload: CreateRoomRequest = {
        name: user?.displayName ? `${user.displayName}のルーム` : '新規ルーム',
        slug: slug,
      };

      // APIが完了するまでここで待機（await）します！
      const newRoom = await createRoom(token, payload);

      // バックエンドから返ってきた作成済みのSlug（またはID）を使って遷移
      // ※ URLは /room/[roomId] の形（仕様書通り）
      if (newRoom.slug) {
        router.push(`/room/${newRoom.slug}`);
      } else {
        // もしバックエンドで自動生成された場合、slugが無ければidを使うなどのフォールバック
        router.push(`/room/${newRoom.id}`);
      }

    } catch (error: unknown) {
      console.error('ルーム作成エラー:', error);
      // バックエンドからの一意制約違反（既出のID）などを想定
      setCreateError('ルームの作成に失敗しました。指定したIDが既に使用されている可能性があります。');
    } finally {
      setIsCreating(false);
    }
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
        <div className="bg-white p-8 rounded-xl shadow-lg hover:shadow-xl transition border-l-4 border-blue-500 flex flex-col justify-between">
          <div>
            <h2 className="text-2xl font-bold mb-4 text-gray-700">ROOMを作成</h2>
            <p className="text-gray-500 mb-6">新しい授業ルームを作成します（教員用）</p>
            
            <div className="mb-4">
              <label className="flex items-center gap-2 cursor-pointer">
                <input 
                  type="checkbox" 
                  checked={isCustomId}
                  onChange={(e) => setIsCustomId(e.target.checked)}
                  className="w-4 h-4 text-blue-600 rounded focus:ring-blue-500"
                />
                <span className="text-gray-700 font-medium">ROOM IDを指定して作成する</span>
              </label>
            </div>

            {isCustomId && (
              <div className="mb-6">
                <input 
                  type="text" 
                  placeholder="任意のルームIDを入力"
                  value={customIdInput}
                  onChange={(e) => setCustomIdInput(e.target.value)}
                  className={`w-full border p-2 rounded focus:outline-none focus:ring-2 ${createError ? 'border-red-500 focus:ring-red-400' : 'border-gray-300 focus:ring-blue-400'}`}
                />
                <p className="text-xs text-gray-500 mt-1">
                  ※4〜16文字の英数字
                </p>
              </div>
            )}

            {createError && (
              <p className="text-red-500 text-sm font-bold mb-4 bg-red-50 p-2 rounded">
                {createError}
              </p>
            )}
          </div>

          <button 
            onClick={handleCreate}
            disabled={isCreating}
            className={`w-full py-3 rounded-lg font-bold text-lg text-white transition mt-4 ${isCreating ? 'bg-gray-400 cursor-not-allowed' : 'bg-blue-500 hover:bg-blue-600'}`}
          >
            {isCreating ? '作成中...' : '+ 新規ルーム作成'}
          </button>
        </div>

      </main>
    </div>
  );
}