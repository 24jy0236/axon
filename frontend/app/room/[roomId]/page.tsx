'use client';

import { useAuth } from '@/hooks/useAuth';
import { useParams, useRouter } from 'next/navigation';
import { useEffect, useState, useRef } from 'react';
import { joinRoom } from '@/lib/api/rooms';
import type { JoinRoomResponse } from '@/types/generated/join_room_response';
import type { WsMessagePayload } from '@/types/generated/ws_message';

export default function RoomPage() {
  const { user, token, loading: authLoading } = useAuth();
  const params = useParams();
  const router = useRouter();
  const slug = params.roomId as string;

  // 部屋情報とローディング状態
  const [roomData, setRoomData] = useState<JoinRoomResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  // 🌟 WebSocket用のステートと参照
  const [messages, setMessages] = useState<WsMessagePayload[]>([]);
  const [inputText, setInputText] = useState('');
  const wsRef = useRef<WebSocket | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null); // 自動スクロール用

  // 【1】部屋の参加検証と情報取得
  useEffect(() => {
    if (authLoading) return;
    if (!user || !token) {
      router.push('/');
      return;
    }

    const fetchRoom = async () => {
      try {
        const data = await joinRoom(token, slug);
        setRoomData(data);
      } catch (err: unknown) {
        setError('ルームの参加に失敗しました。');
      } finally {
        setLoading(false);
      }
    };

    fetchRoom();
  }, [slug, token, user, authLoading, router]);

  // 【2】WebSocketの接続管理
  useEffect(() => {
    // 部屋情報の取得が完了し、トークンがある場合のみ接続を開始
    if (!roomData || !token) return;

    // 現在のURLからWebSocketのURLを判定 (ローカル開発環境と本番環境の切り替え)
    // ※ API側で 13964 ポートを開いている場合はそれに合わせる
    const isLocal = window.location.hostname === 'localhost';
    const wsProtocol = isLocal ? 'ws:' : 'wss:';
    const wsHost = isLocal ? 'localhost:13964' : 'axon.asappy.xyz'; // サーバーのドメイン
    const wsUrl = `${wsProtocol}//${wsHost}/api/room/${slug}/ws?token=${token}`;

    console.log('Connecting to WebSocket:', wsUrl);
    const ws = new WebSocket(wsUrl);

    // 接続成功時
    ws.onopen = () => {
      console.log('✅ WebSocket Connected!');
    };

    // メッセージ受信時
    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data) as WsMessagePayload;
        setMessages((prev) => [...prev, msg]);
      } catch (e) {
        console.error('Failed to parse message:', e);
      }
    };

    // エラー発生時
    ws.onerror = (err) => {
      console.error('❌ WebSocket Error:', err);
    };

    // 接続切断時
    ws.onclose = () => {
      console.log('🔌 WebSocket Disconnected');
    };

    // コンポーネント外から send できるように useRef に保存
    wsRef.current = ws;

    // クリーンアップ関数: コンポーネントがアンマウントされたら切断する
    return () => {
      ws.close();
    };
  }, [roomData, slug, token]);

  // 新しいメッセージが来たら自動で一番下までスクロールする
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // 【3】メッセージ送信関数
  const handleSendMessage = () => {
    if (!inputText.trim() || !wsRef.current) return;

    // WebSocket経由でサーバーに送信！
    wsRef.current.send(inputText);
    setInputText(''); // 送信後は入力欄を空にする
  };

  // エンターキーで送信できるようにする
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    // 日本語変換中のエンター（e.nativeEvent.isComposing）は無視する
    if (e.key === 'Enter' && !e.nativeEvent.isComposing) {
      e.preventDefault();
      handleSendMessage();
    }
  };


  // --- 表示部分 ---

  if (authLoading || loading) {
    return <div className="min-h-screen flex items-center justify-center bg-gray-50">参加中...</div>;
  }

  if (error || !roomData) {
    return <div className="min-h-screen flex items-center justify-center bg-gray-50 text-red-500 font-bold">{error}</div>;
  }

  return (
    <div className="min-h-screen flex flex-col h-screen bg-gray-50">
      {/* ヘッダー領域 */}
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
          className="text-sm text-gray-500 hover:text-gray-800 border px-3 py-1 rounded transition"
        >
          退出
        </button>
      </header>

      {/* チャット表示領域 */}
      <main className="flex-1 overflow-y-auto p-4 flex flex-col gap-4">
        {messages.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <p className="mb-2">🎉 ルームへの参加が完了しました！</p>
            <p>メッセージを入力して会話を始めましょう。</p>
          </div>
        ) : (
          messages.map((msg) => {
            // 教師かどうかでスタイリングを分岐！
            const isTeacher = msg.sender_role === 'Teacher';

            return (
              <div key={msg.id} className="flex flex-col self-start max-w-[80%]">
                {/* 送信者情報 */}
                <div className="flex items-center gap-2 mb-1 pl-1">
                  {msg.sender_photo_url ? (
                    /* eslint-disable-next-line @next/next/no-img-element */
                    <img src={msg.sender_photo_url} alt="avatar" className="w-6 h-6 rounded-full border shadow-sm" />
                  ) : (
                    <div className="w-6 h-6 rounded-full bg-gray-300 flex items-center justify-center text-xs text-white">
                      ?
                    </div>
                  )}
                  <span className={`text-xs font-bold ${isTeacher ? 'text-red-600' : 'text-gray-500'}`}>
                    {msg.sender_name}
                  </span>
                  {isTeacher && (
                    <span className="bg-red-100 text-red-600 text-[10px] px-1.5 py-0.5 rounded border border-red-200">
                      教員
                    </span>
                  )}
                  <span className="text-[10px] text-gray-400">
                    {new Date(msg.sent_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                  </span>
                </div>

                {/* 吹き出し */}
                <div className={`p-3 rounded-2xl shadow-sm border ${isTeacher
                    ? 'bg-red-50 border-red-200 text-gray-800 rounded-tl-none'
                    : 'bg-white border-gray-200 text-gray-800 rounded-tl-none'
                  }`}>
                  <span className="whitespace-pre-wrap">{msg.content}</span>
                </div>
              </div>
            );
          })
        )}
        <div ref={messagesEndRef} />
      </main>

      {/* メッセージ入力領域 */}
      <footer className="bg-white p-4 border-t">
        <div className="max-w-4xl mx-auto flex gap-2">
          <input
            type="text"
            value={inputText}
            onChange={(e) => setInputText(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="[全体] メッセージを入力..."
            className="flex-1 border border-gray-300 rounded-lg p-3 focus:outline-none focus:ring-2 focus:ring-blue-400 bg-gray-50"
          />
          <button
            onClick={handleSendMessage}
            disabled={!inputText.trim()}
            className="bg-blue-500 text-white px-6 py-2 rounded-lg font-bold disabled:bg-gray-400 disabled:cursor-not-allowed hover:bg-blue-600 transition"
          >
            送信
          </button>
        </div>
      </footer>
    </div>
  );
}