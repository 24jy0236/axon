"use client"; // Error components must be Client Components

import { useEffect } from "react";

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    // ここでエラーの正体がコンソールに出るはずだ！
    console.error("🔥 捕獲したエラー:", error);
  }, [error]);

  return (
    <div className="p-4 bg-red-100 text-red-900 border border-red-500 rounded">
      <h2 className="text-xl font-bold">Something went wrong!</h2>
      <p className="mt-2 text-sm font-mono bg-white p-2 rounded">{error.message}</p>
      <button
        className="mt-4 px-4 py-2 bg-red-500 text-white rounded"
        onClick={() => reset()}
      >
        Try again
      </button>
    </div>
  );
}