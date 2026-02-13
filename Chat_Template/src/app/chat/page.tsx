'use client';

import { useEffect, useState, useRef } from 'react';
import { useRouter } from 'next/navigation';
import { getUserRooms, createDirectRoom, getRoomMessages, getAllUsers } from '@/lib/api';
import { Room, Message, User } from '@/types';
import { ChatWebSocket } from '@/lib/websocket';

export default function ChatPage() {
  const router = useRouter();
  const [user, setUser] = useState<User | null>(null);
  const [rooms, setRooms] = useState<Room[]>([]);
  const [selectedRoom, setSelectedRoom] = useState<Room | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [newMessage, setNewMessage] = useState('');
  const [ws, setWs] = useState<ChatWebSocket | null>(null);
  const [showFindFriends, setShowFindFriends] = useState(false);
  const [allUsers, setAllUsers] = useState<User[]>([]);
  const [usersMap, setUsersMap] = useState<Map<number, User>>(new Map());
  const [loading, setLoading] = useState(true);
  const [unreadRooms, setUnreadRooms] = useState<Set<string>>(new Set());
  const [isWindowActive, setIsWindowActive] = useState(true);
  
  // Refs để access latest values trong WebSocket callback
  const isWindowActiveRef = useRef(true);
  const userRef = useRef<User | null>(null);
  const selectedRoomRef = useRef<Room | null>(null);

  // Sync refs với state
  useEffect(() => {
    isWindowActiveRef.current = isWindowActive;
  }, [isWindowActive]);

  useEffect(() => {
    userRef.current = user;
  }, [user]);

  useEffect(() => {
    selectedRoomRef.current = selectedRoom;
  }, [selectedRoom]);

  useEffect(() => {
    const token = localStorage.getItem('token');
    const userStr = localStorage.getItem('user');
    
    if (!token || !userStr) {
      router.push('/login');
      return;
    }

    const currentUser = JSON.parse(userStr);
    setUser(currentUser);
    
    // Load users và rooms, sau đó connect WebSocket
    const init = async () => {
      // Load users trước để có usersMap
      await loadUsers();
      const loadedRooms = await loadRooms();
      
      // Connect WebSocket một lần duy nhất
      connectWebSocket(token, loadedRooms);
    };
    
    init();
    
    // Detect window focus/blur
    const handleFocus = () => setIsWindowActive(true);
    const handleBlur = () => setIsWindowActive(false);
    const handleVisibilityChange = () => {
      setIsWindowActive(!document.hidden);
    };
    
    window.addEventListener('focus', handleFocus);
    window.addEventListener('blur', handleBlur);
    document.addEventListener('visibilitychange', handleVisibilityChange);
    
    // Cleanup khi unmount
    return () => {
      if (ws) {
        ws.disconnect();
      }
      window.removeEventListener('focus', handleFocus);
      window.removeEventListener('blur', handleBlur);
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, [router]);

  const loadRooms = async () => {
    try {
      const data = await getUserRooms();
      setRooms(data);
      return data;
    } catch (error) {
      console.error('Failed to load rooms:', error);
      return [];
    } finally {
      setLoading(false);
    }
  };

  const loadUsers = async () => {
    try {
      const data = await getAllUsers();
      
      // Tạo map để lookup user nhanh
      const map = new Map<number, User>();
      data.forEach(u => map.set(u.id, u));
      setUsersMap(map);
      
      // Filter sau khi set map
      setAllUsers(data.filter(u => u.id !== user?.id));
    } catch (error) {
      console.error('Failed to load users:', error);
    }
  };

  const connectWebSocket = (token: string, roomsToJoin: Room[]) => {
    if (ws) {
      console.log('WebSocket already connected');
      return;
    }

    try {
      const newWs = new ChatWebSocket(token);
      
      // Connect WebSocket
      newWs.connectGlobal();
      
      // Wait for connection then join all rooms
      setTimeout(() => {
        roomsToJoin.forEach(room => {
          newWs.send({
            type: 'join_room',
            room_id: room.id,
          });
        });
      }, 500);
      
      newWs.onMessage((message) => {
        console.log('📬 Received WebSocket message:', message);
        
        try {
          if (message.type === 'message') {
            // Tin nhắn mới từ bất kỳ room nào
            const newMsg: Message = {
              id: message.id!,
              room_id: message.room_id!,
              user_id: message.sender_id!,
              sender_id: message.sender_id!,
              sender_name: message.sender_name || undefined,
              content: message.content!,
              created_at: message.created_at!,
            };
            
            // Cập nhật messages hoặc unread indicator
            const currentRoom = selectedRoomRef.current;
            const currentUser = userRef.current;
            const windowActive = isWindowActiveRef.current;
            
            if (currentRoom && currentRoom.id === message.room_id) {
              // Đang xem room này
              setMessages(prev => {
                // Kiểm tra xem tin nhắn đã tồn tại chưa (tránh duplicate)
                const exists = prev.some(m => m.id === newMsg.id);
                if (exists) return prev;
                
                // Loại bỏ tin nhắn tạm thời nếu có
                const filtered = prev.filter(m => !m.id.toString().startsWith('temp-'));
                return [...filtered, newMsg];
              });
              
              // Nếu window không active và không phải tin nhắn của mình, đánh dấu unread
              if (!windowActive && message.sender_id !== currentUser?.id) {
                setUnreadRooms(prev => new Set(prev).add(message.room_id!));
              }
            } else {
              // Không xem room này, đánh dấu unread nếu không phải tin nhắn của mình
              if (message.sender_id !== currentUser?.id) {
                setUnreadRooms(prev => new Set(prev).add(message.room_id!));
              }
            }
            
          } else if (message.type === 'joined') {
            console.log(`✅ Successfully joined room ${message.room_id}`);
          } else if (message.type === 'left') {
            console.log(`Left room ${message.room_id}`);
          } else if (message.type === 'typing') {
            console.log(`User ${message.user_id} is typing: ${message.is_typing}`);
          } else if (message.type === 'error') {
            console.error('❌ WebSocket error from server:', message.message);
          } else if (message.type === 'pong') {
            console.log('Received pong');
          }
        } catch (error) {
          console.error('Error handling WebSocket message:', error, message);
        }
      });
      
      setWs(newWs);
    } catch (error) {
      console.error('Failed to connect WebSocket:', error);
    }
  };

  const selectRoom = async (room: Room) => {
    setSelectedRoom(room);

    // Xóa unread indicator cho room này
    setUnreadRooms(prev => {
      const newSet = new Set(prev);
      newSet.delete(room.id);
      return newSet;
    });

    // Load messages cho room này
    try {
      const msgs = await getRoomMessages(room.id);
      setMessages(msgs.reverse());
    } catch (error) {
      console.error('Failed to load messages:', error);
    }
  };

  // Xóa unread khi window active trở lại và đang xem room
  useEffect(() => {
    if (isWindowActive && selectedRoom) {
      setUnreadRooms(prev => {
        const newSet = new Set(prev);
        newSet.delete(selectedRoom.id);
        return newSet;
      });
    }
  }, [isWindowActive, selectedRoom]);

  const sendMessage = () => {
    if (!newMessage.trim() || !ws || !selectedRoom || !user) return;

    // Tạo tin nhắn tạm thời để hiển thị ngay
    const tempMessage: Message = {
      id: `temp-${Date.now()}`,
      room_id: selectedRoom.id,
      user_id: user.id,
      sender_id: user.id,
      sender_name: user.name,
      content: newMessage,
      created_at: new Date().toISOString(),
    };

    // Thêm tin nhắn vào UI ngay lập tức
    setMessages(prev => [...prev, tempMessage]);

    // Gửi qua WebSocket
    ws.send({
      type: 'message',
      room_id: selectedRoom.id,
      content: newMessage,
      message_type: 'text',
    });

    setNewMessage('');
  };

  const startChat = async (otherUser: User) => {
    try {
      const room = await createDirectRoom(otherUser.id);
      setRooms(prev => {
        const exists = prev.find(r => r.id === room.id);
        return exists ? prev : [...prev, room];
      });
      
      // Join room mới qua WebSocket
      if (ws) {
        ws.send({
          type: 'join_room',
          room_id: room.id,
        });
      }
      
      setShowFindFriends(false);
      selectRoom(room);
    } catch (error) {
      console.error('Failed to create room:', error);
    }
  };

  const logout = () => {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    if (ws) ws.disconnect();
    router.push('/login');
  };

  const getRoomName = (room: Room) => {
    // Nếu có tên room (group chat), hiển thị tên đó
    if (room.name) return room.name;
    
    // Nếu là direct chat, hiển thị tên người kia
    if (room.room_type === 'direct' && user && room.members) {
      const otherMember = room.members.find(m => m.user_id !== user.id);
      if (otherMember) {
        // Ưu tiên dùng user_name từ member (đã join từ backend)
        if (otherMember.user_name) {
          return otherMember.user_name;
        }
        // Fallback: tìm trong usersMap
        const otherUser = usersMap.get(otherMember.user_id);
        if (otherUser) {
          return otherUser.name;
        }
        return `User ${otherMember.user_id}`;
      }
    }
    
    // Fallback
    return room.room_type === 'group' ? 'Nhóm chat' : 'Trò chuyện';
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-gray-900 mx-auto"></div>
          <p className="mt-4 text-gray-600">Đang tải...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar */}
      <div className="w-80 bg-white border-r border-gray-200 flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <div className="flex items-center justify-between mb-4">
            <h1 className="text-xl font-bold text-gray-900">Chat App</h1>
            <button
              onClick={logout}
              className="text-sm text-red-600 hover:text-red-700"
            >
              Đăng xuất
            </button>
          </div>
          <div className="text-sm text-gray-600">
            Xin chào, {user?.name}
          </div>
        </div>

        <div className="p-4 border-b border-gray-200">
          <button
            onClick={() => {
              setShowFindFriends(true);
              loadUsers();
            }}
            className="w-full bg-indigo-600 text-white py-2 px-4 rounded-md hover:bg-indigo-700"
          >
            Tìm bạn bè
          </button>
        </div>

        <div className="flex-1 overflow-y-auto">
          <div className="p-2">
            <h2 className="text-sm font-semibold text-gray-600 px-2 mb-2">
              Cuộc trò chuyện
            </h2>
            {rooms.length === 0 ? (
              <p className="text-sm text-gray-500 px-2">Chưa có cuộc trò chuyện nào</p>
            ) : (
              rooms.map(room => {
                const hasUnread = unreadRooms.has(room.id);
                return (
                  <button
                    key={room.id}
                    onClick={() => selectRoom(room)}
                    className={`w-full text-left p-3 rounded-lg mb-1 hover:bg-gray-100 relative ${
                      selectedRoom?.id === room.id ? 'bg-indigo-50' : ''
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <div className={`flex-1 ${hasUnread ? 'font-bold' : 'font-medium'} text-gray-900`}>
                        {getRoomName(room)}
                      </div>
                      {hasUnread && (
                        <div className="w-2.5 h-2.5 bg-blue-500 rounded-full ml-2"></div>
                      )}
                    </div>
                    <div className="text-xs text-gray-500">
                      {room.room_type === 'direct' ? 'Trò chuyện riêng' : 'Nhóm'}
                    </div>
                  </button>
                );
              })
            )}
          </div>
        </div>
      </div>

      {/* Main Chat Area */}
      <div className="flex-1 flex flex-col">
        {selectedRoom ? (
          <>
            <div className="bg-white border-b border-gray-200 p-4">
              <h2 className="text-lg font-semibold text-gray-900">{getRoomName(selectedRoom)}</h2>
              <p className="text-sm text-gray-500">
                {selectedRoom.members?.length || 0} thành viên
              </p>
            </div>

            <div className="flex-1 overflow-y-auto p-4 space-y-4 bg-gray-50">
              {messages.map(msg => {
                const isOwnMessage = msg.user_id === user?.id;
                const displayName = msg.sender_name || usersMap.get(msg.user_id)?.name || `User ${msg.user_id}`;
                
                return (
                  <div
                    key={msg.id}
                    className={`flex ${isOwnMessage ? 'justify-end' : 'justify-start'}`}
                  >
                    <div
                      className={`max-w-xs lg:max-w-md px-4 py-2 rounded-lg ${
                        isOwnMessage
                          ? 'bg-indigo-600 text-white'
                          : 'bg-white text-gray-900 border border-gray-200'
                      }`}
                    >
                      <div className={`text-xs mb-1 ${
                        isOwnMessage ? 'text-indigo-100' : 'text-gray-500'
                      }`}>
                        {displayName}
                      </div>
                      <div>{msg.content}</div>
                    </div>
                  </div>
                );
              })}
            </div>

            <div className="bg-white border-t border-gray-200 p-4">
              <div className="flex space-x-2">
                <input
                  type="text"
                  value={newMessage}
                  onChange={(e) => setNewMessage(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && sendMessage()}
                  placeholder="Nhập tin nhắn..."
                  className="flex-1 border border-gray-300 rounded-lg px-4 py-2 focus:outline-none focus:ring-2 focus:ring-indigo-500 text-gray-900"
                />
                <button
                  onClick={sendMessage}
                  className="bg-indigo-600 text-white px-6 py-2 rounded-lg hover:bg-indigo-700"
                >
                  Gửi
                </button>
              </div>
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center text-gray-500 bg-gray-50">
            Chọn một cuộc trò chuyện để bắt đầu
          </div>
        )}
      </div>

      {/* Find Friends Modal */}
      {showFindFriends && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-96 max-h-96 overflow-y-auto">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-xl font-bold text-gray-900">Tìm bạn bè</h2>
              <button
                onClick={() => setShowFindFriends(false)}
                className="text-gray-500 hover:text-gray-700"
              >
                ✕
              </button>
            </div>
            <div className="space-y-2">
              {allUsers.map(u => (
                <button
                  key={u.id}
                  onClick={() => startChat(u)}
                  className="w-full text-left p-3 border border-gray-200 rounded-lg hover:bg-gray-50"
                >
                  <div className="font-medium text-gray-900">{u.name}</div>
                  <div className="text-sm text-gray-500">{u.email}</div>
                </button>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
