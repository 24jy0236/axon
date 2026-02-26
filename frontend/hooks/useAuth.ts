// frontend/hooks/useAuth.ts
import { useState, useEffect } from 'react';
import { auth } from '@/lib/firebase'; // パスエイリアス(@)を使うとスマート！
import { 
  GoogleAuthProvider, 
  signInWithPopup, 
  signOut as firebaseSignOut,
  onAuthStateChanged,
  User 
} from "firebase/auth";

// フックが返す値の型定義
type UseAuthReturn = {
  user: User | null;
  token: string | null;
  error: string | null;
  login: () => Promise<void>;
  logout: () => Promise<void>;
  loading: boolean; // ローディング状態もあると親切！
};

export const useAuth = (): UseAuthReturn => {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState<boolean>(true); // 初期ロード中

  // 監視リスナーを設定 (ページリロードしてもログイン状態を維持できる！)
  useEffect(() => {
    const unsubscribe = onAuthStateChanged(auth, async (currentUser) => {
      try {
        setUser(currentUser);
        if (currentUser) {
          const idToken = await currentUser.getIdToken();
          setToken(idToken);
        } else {
          setToken(null);
        }
      } catch (err: any) {
        console.error("Auth State Change Error:", err);
      } finally {
        setLoading(false);
      }
    });

    // コンポーネントが消える時にリスナーを解除
    return () => unsubscribe();
  }, []);

  const login = async () => {
    const provider = new GoogleAuthProvider();
    try {
      setError(null);
      await signInWithPopup(auth, provider);
      // onAuthStateChanged が勝手に検知してくれるので、ここで setUser しなくてOK！
    } catch (error: unknown) {
      if (error instanceof Error) {
        console.error(error.message);
        setError(error.message);
      } else {
        console.error("予期せぬエラーが発生しました", error);
        setError("予期せぬエラーが発生しました");
      }
    }
  };

  const logout = async () => {
    try {
      setError(null);
      await firebaseSignOut(auth);
    } catch (err: any) {
      console.error("Logout Error:", err);
      setError(err.message);
    }
  };

  return { user, token, error, login, logout, loading };
};