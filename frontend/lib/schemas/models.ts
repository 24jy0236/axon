import { z } from "zod";
// さっき cargo test で生成した型をインポート！
import type { Room } from "@/types/generated/room";
import type { RoomId, UserId } from "@/types/generated/branded_types";

// --- Branded Types 用のカスタムバリデーション ---
// ただの文字列ではなく、UUID形式であることを確認した上で、ブランド（型）を付与する
export const RoomIdSchema = z.string().uuid().transform((val) => val as RoomId);
export const UserIdSchema = z.string().uuid().transform((val) => val as UserId);

// --- Entity スキーマ ---
// `satisfies z.ZodType<Room>` が超重要！
// もし Rust 側で Room のフィールドが増えたり型が変わったら、ここで即座にコンパイルエラーになる！
export const RoomSchema = z.object({
  id: RoomIdSchema,
  slug: z.string().min(4).max(16), // 仕様書通りのバリデーション
  name: z.string().min(1, "ルーム名を入力してください"),
  owner_id: UserIdSchema,
  created_at: z.string().datetime(), // Rustの DateTime<Utc> は ISO8601 文字列で来る
  updated_at: z.string().datetime(),
}) satisfies z.ZodType<Room>;

// --- DTO スキーマ ---
// フロントからバックエンドへ送るリクエストの検証用
import type { CreateRoomRequest } from "@/types/generated/create_room_dto";

export const CreateRoomRequestSchema = z.object({
  name: z.string().min(1),
  slug: z.string().min(4).max(16).nullable(), // Option<String> は nullable で受ける
}) satisfies z.ZodType<CreateRoomRequest>;