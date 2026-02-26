import { z } from "zod";
import type { Room } from "@/types/generated/room";
import type { RoomId, UserId } from "@/types/generated/branded_types";

// カスタムバリデーション: 単なる文字列を Branded Type として扱う魔法
const roomIdSchema = z.uuid({version: "v7"}).transform((val) => val as RoomId);
const userIdSchema = z.uuid({version: "v7"}).transform((val) => val as UserId);

// Room の Zod スキーマ
export const RoomSchema = z.object({
  id: roomIdSchema,
  slug: z.string(),
  name: z.string(),
  owner_id: userIdSchema,
  created_at: z.string(), // 日付は一旦文字列として受ける
  updated_at: z.string(),
}) satisfies z.ZodType<Room>; 
// ↑ satisfies を使うことで、「生成されたTSの型」と「Zodの定義」がズレたらコンパイルエラーになる！(超重要)