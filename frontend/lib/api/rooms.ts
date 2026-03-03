import type { CreateRoomRequest } from "@/types/generated/create_room_dto";
import { JoinRoomResponse } from "@/types/generated/join_room_response";
import type { Room } from "@/types/generated/room";
import { RoomSchema } from "../schemas/models";

export async function createRoom(token: string, payload: CreateRoomRequest): Promise<Room> {
  const res = await fetch("https://axon.asappy.xyz/api/room/create", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Authorization": `Bearer ${token}`,
    },
    body: JSON.stringify(payload),
  });

  if (!res.ok) {
    throw new Error("Failed to create room");
  }

  const data = await res.json();
  
  // 🔥 ここが水際対策！パースに失敗したらエラーを投げ、UIには不正なデータを行かせない
  return RoomSchema.parse(data);
}

export async function joinRoom(token: string, slug: string): Promise<JoinRoomResponse> {
  // 環境に合わせてURLは調整してください (ローカルなら http://localhost:13964/api/room/${slug}/join)
  const res = await fetch(`https://axon.asappy.xyz/api/room/${slug}/join`, {
    method: "POST", // バックエンドで post(join_room_handler) と定義したためPOST
    headers: {
      "Content-Type": "application/json",
      "Authorization": `Bearer ${token}`,
    },
  });

  if (!res.ok) {
    if (res.status === 404) {
      throw new Error("RoomNotFound");
    }
    throw new Error("Failed to join room");
  }

  // Zodでパース（水際対策）するのがベストですが、まずは一旦そのまま返して疎通確認します
  const data = await res.json();
  return data as JoinRoomResponse; 
}