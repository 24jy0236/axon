import { initializeApp, getApps } from "firebase/app";
import { getAuth } from "firebase/auth";

const firebaseConfig = {
  apiKey: process.env.FIREBASE_API_KEY,
  authDomain: process.env.FIREBASE_AUTH_DOMAIN,
  projectId: process.env.FIREBASE_PROJECT_ID,
  storageBucket: process.env.FIREBASE_STORAGE_BUCKET,
  messagingSenderId: process.env.FIREBASE_MESSAGING_SENDER_ID,
  appId: process.env.FIREBASE_APP_ID,
  measurementId: process.env.FIREBASE_MEASUREMENT_ID
};

// アプリが既に初期化されていなければ初期化する
// (Next.jsの開発モードでは再読み込みが走るから、このチェックが大事！)
const app = getApps().length ? getApps()[0] : initializeApp(firebaseConfig);

// 他のファイルで使えるように、authオブジェクトをエクスポート
export const auth = getAuth(app);