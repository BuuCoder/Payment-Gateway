import axios from 'axios';
import { AuthResponse, Room, Message, User } from '@/types';

// Sử dụng Next.js API routes làm proxy để tránh CORS
const api = axios.create({
  baseURL: '/api',
});

// Add token to requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Auth APIs
export const register = async (name: string, email: string, password: string): Promise<AuthResponse> => {
  const { data } = await api.post<AuthResponse>('/auth/register', { name, email, password });
  return data;
};

export const login = async (email: string, password: string): Promise<AuthResponse> => {
  const { data } = await api.post<AuthResponse>('/auth/login', { email, password });
  return data;
};

// Chat APIs
export const getUserRooms = async (): Promise<Room[]> => {
  const { data } = await api.get<Room[]>('/rooms');
  return data;
};

export const createDirectRoom = async (otherUserId: number): Promise<Room> => {
  const { data } = await api.post<Room>('/rooms/direct', { other_user_id: otherUserId });
  return data;
};

export const getRoomMessages = async (roomId: string, limit = 50): Promise<Message[]> => {
  const { data } = await api.get<any[]>(`/rooms/${roomId}/messages`, { params: { limit } });
  // Map sender_id to user_id for consistency
  return data.map(msg => ({
    id: msg.id,
    room_id: msg.room_id,
    user_id: msg.sender_id,
    sender_id: msg.sender_id,
    sender_name: msg.sender_name,
    content: msg.content,
    created_at: msg.created_at,
  }));
};

// Core APIs - Get all users for finding friends
export const getAllUsers = async (): Promise<User[]> => {
  const { data } = await api.get<User[]>('/users');
  return data;
};
