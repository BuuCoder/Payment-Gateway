export interface User {
  id: number;
  name: string;
  email: string;
  created_at: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface Room {
  id: string;
  name: string | null;
  room_type: 'direct' | 'group';
  created_by: number;
  created_at: string;
  members?: RoomMember[]; // Optional vì một số API không trả về members
}

export interface RoomMember {
  user_id: number;
  role: string;
  joined_at: string;
  user_name?: string;
  user_email?: string;
}

export interface Message {
  id: string;
  room_id: string;
  user_id: number;
  sender_id?: number; // Alias for user_id from API
  sender_name?: string; // Tên người gửi từ API
  content: string;
  created_at: string;
}

// WebSocket message types - Client gửi lên server
export interface WsClientMessage {
  type: 'message' | 'join_room' | 'leave_room' | 'typing' | 'ping';
  room_id?: string;
  content?: string;
  message_type?: string;
  metadata?: any;
  is_typing?: boolean;
}

// WebSocket response types - Server trả về
export interface WsServerMessage {
  type: 'message' | 'joined' | 'left' | 'typing' | 'error' | 'pong';
  // For message type
  id?: string;
  room_id?: string;
  sender_id?: number;
  sender_name?: string | null;
  content?: string;
  message_type?: string;
  metadata?: any;
  created_at?: string;
  // For typing type
  user_id?: number;
  user_name?: string | null;
  is_typing?: boolean;
  // For error type
  message?: string;
}
