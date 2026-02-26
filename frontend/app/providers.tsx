"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";

export default function Providers({ children }: { children: React.ReactNode }) {
  // コンポーネントが再レンダリングされても QueryClient が再生成されないように useState で保持する
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 1000 * 60 * 5, // 5分間はキャッシュを新鮮とみなす
            refetchOnWindowFocus: false, // ウィンドウフォーカス時の自動フェッチをオフ（お好みで）
          },
        },
      })
  );

  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
}