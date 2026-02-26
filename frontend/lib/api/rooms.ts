import { RoomSchema } from "@/lib/schemas/room";
import type { CreateRoomRequest } from "@/types/generated/create_room_dto";
import type { Room } from "@/types/generated/room";

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