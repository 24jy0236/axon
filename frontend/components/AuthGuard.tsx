'use client';

import { useAuth } from '@/hooks/useAuth'; // さっき作ったフック
import { useRouter, usePathname } from 'next/navigation';
import { useEffect } from 'react';

export default function AuthGuard({ children }: { children: React.ReactNode }) {
  const { user, loading } = useAuth();
  const router = useRouter();
  const pathname = usePathname();

  useEffect(() => {
    // ロードが終わって、ユーザーがいなくて、今いるのがログインページじゃないなら
    if (!loading && !user && pathname !== '/login') {
      router.push('/login'); // 強制送還！
    }
  }, [user, loading, pathname, router]);

  // ロード中は画面を隠すか、ローディング表示
  if (loading) return <div className="min-h-screen flex items-center justify-center">Loading AXON...</div>;

  // ログインページにいる時や、ユーザーがいる時は中身を表示
  return <>{children}</>;
}