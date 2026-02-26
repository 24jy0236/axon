import { CreateRoomRequest } from "@/types/generated/create_room_dto";
import { Room } from "@/types/generated/room";

/**
 * 
 * @param name ROOM名
 * @param token FirebaseのIDトークン
 * @returns 
 */
export async function createRoom(name: string, token: string): Promise<Room> {
  const payload: CreateRoomRequest = { 
    name, 
    slug: null // これを足す！
  };
  
  const res = await fetch("https://axon.asappy.xyz/api/rooms", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Authorization": `Bearer ${token}`,
    },
    body: JSON.stringify(payload),
  });

  if (!res.ok) throw new Error("Failed to create room");
  return res.json();
}