'use client';

import { useEffect, useState, useRef } from 'react';
import { useRouter } from 'next/navigation';
import { 
  getUserRooms, createDirectRoom, createGroupRoom, getRoomMessages, getAllUsers,
  getInvitations, acceptInvitation, declineInvitation,
  leaveRoom, hideRoom, markRoomAsRead
} from '@/lib/api';
import { Room, Message, User, Invitation } from '@/types';
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
  const [showCreateGroup, setShowCreateGroup] = useState(false);
  const [showInvitations, setShowInvitations] = useState(false);
  const [allUsers, setAllUsers] = useState<User[]>([]);
  const [usersMap, setUsersMap] = useState<Map<number, User>>(new Map());
  const [loading, setLoading] = useState(true);
  const [invitations, setInvitations] = useState<Invitation[]>([]);
  const [connectionReplaced, setConnectionReplaced] = useState(false);
  
  // Group chat state
  const [groupName, setGroupName] = useState('');
  const [selectedMembers, setSelectedMembers] = useState<Set<number>>(new Set());
  
  // Presence & Typing state
  const [onlineUsers, setOnlineUsers] = useState<Set<number>>(new Set());
  const [roomPresence, setRoomPresence] = useState<Map<string, number[]>>(new Map());
  const [typingUsers, setTypingUsers] = useState<Map<string, Set<string>>>(new Map());
  const typingTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  
  // Refs để access latest values trong WebSocket callback
  const userRef = useRef<User | null>(null);
  const selectedRoomRef = useRef<Room | null>(null);
  const joinedRoomsRef = useRef<Set<string>>(new Set());

  // Sync refs với state
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
      await loadUsers(currentUser);
      const loadedRooms = await loadRooms();
      await loadInvitations();
      connectWebSocket(token, loadedRooms);
    };
    
    init();
    
    // Reload invitations when window gains focus (lightweight)
    const handleFocus = () => {
      loadInvitations();
    };
    window.addEventListener('focus', handleFocus);
    
    // Cleanup khi unmount
    return () => {
      window.removeEventListener('focus', handleFocus);
      if (ws) {
        ws.disconnect();
      }
    };
  }, [router]);

  const loadRooms = async () => {
    try {
      const data = await getUserRooms();
      // Sort by last_message_at or created_at
      const sorted = data.sort((a, b) => {
        const aTime = new Date(a.last_message_at || a.created_at).getTime();
        const bTime = new Date(b.last_message_at || b.created_at).getTime();
        return bTime - aTime;
      });
      setRooms(sorted);
      
      // Auto join all rooms via WebSocket nếu đã connected
      if (ws) {
        sorted.forEach(room => {
          if (!joinedRoomsRef.current.has(room.id)) {
            ws.send({
              type: 'join_room',
              room_id: room.id,
            });
            joinedRoomsRef.current.add(room.id);
          }
        });
      }
      
      return sorted;
    } catch (error) {
      console.error('Failed to load rooms:', error);
      return [];
    } finally {
      setLoading(false);
    }
  };

  const loadUsers = async (currentUser: User) => {
    try {
      const data = await getAllUsers();
      const map = new Map<number, User>();
      data.forEach(u => map.set(u.id, u));
      setUsersMap(map);
      // Filter out current user from the list
      setAllUsers(data.filter(u => u.id !== currentUser.id));
    } catch (error) {
      console.error('Failed to load users:', error);
    }
  };

  const loadInvitations = async () => {
    try {
      const data = await getInvitations();
      setInvitations(data);
    } catch (error) {
      console.error('Failed to load invitations:', error);
    }
  };

  const connectWebSocket = (token: string, roomsToJoin: Room[]) => {
    if (ws) {
      console.log('WebSocket already connected');
      return;
    }

    try {
      const newWs = new ChatWebSocket(token);
      newWs.connectGlobal();
      
      // Wait for connection then join all rooms
      setTimeout(() => {
        roomsToJoin.forEach(room => {
          if (!joinedRoomsRef.current.has(room.id)) {
            newWs.send({
              type: 'join_room',
              room_id: room.id,
            });
            joinedRoomsRef.current.add(room.id);
            console.log('Joined room:', room.id);
          }
        });
      }, 1000);
      
      // Handle incoming messages
      newWs.onMessage((message) => {
        console.log('Received WebSocket message:', message);
        
        if (message.type === 'message') {
          const newMsg: Message = {
            id: message.id!,
            room_id: message.room_id!,
            user_id: message.sender_id!,
            sender_id: message.sender_id!,
            sender_name: message.sender_name || undefined,
            content: message.content!,
            created_at: message.created_at!,
          };
          
          // Add to messages if viewing this room
          if (selectedRoomRef.current?.id === message.room_id) {
            setMessages(prev => {
              // Check if this is replacing a temporary message
              const hasTempMessage = prev.some(m => m.id.toString().startsWith('temp-'));
              
              if (hasTempMessage && message.sender_id === userRef.current?.id) {
                // Replace the last temporary message with real message
                const filtered = prev.filter(m => !m.id.toString().startsWith('temp-'));
                return [...filtered, newMsg];
              }
              
              // Otherwise just add the new message
              return [...prev, newMsg];
            });
          }
        } else if (message.type === 'room_created') {
          console.log('Room created notification received:', message);
          loadRooms();
          
          // Auto join the new room
          if (message.room_id && !joinedRoomsRef.current.has(message.room_id)) {
            newWs.send({
              type: 'join_room',
              room_id: message.room_id,
            });
            joinedRoomsRef.current.add(message.room_id);
            console.log('Auto-joined new room:', message.room_id);
          }
        } else if (message.type === 'invitation_received') {
          console.log('Invitation received:', message);
          loadInvitations();
        } else if (message.type === 'member_joined') {
          console.log('Member joined:', message);
          // Reload rooms to get updated member count
          loadRooms();
          
          // If viewing this room, show system message
          if (selectedRoomRef.current?.id === message.room_id) {
            const systemMsg: Message = {
              id: `system-${Date.now()}`,
              room_id: message.room_id!,
              user_id: 0, // System message
              sender_id: 0,
              sender_name: 'System',
              content: `${message.user_name} đã tham gia nhóm`,
              created_at: new Date().toISOString(),
            };
            setMessages(prev => [...prev, systemMsg]);
          }
        } else if (message.type === 'member_left') {
          console.log('Member left:', message);
          // Reload rooms to get updated member count
          loadRooms();
          
          // If viewing this room, show system message
          if (selectedRoomRef.current?.id === message.room_id) {
            const systemMsg: Message = {
              id: `system-${Date.now()}`,
              room_id: message.room_id!,
              user_id: 0,
              sender_id: 0,
              sender_name: 'System',
              content: `${message.user_name} đã rời khỏi nhóm`,
              created_at: new Date().toISOString(),
            };
            setMessages(prev => [...prev, systemMsg]);
          }
        } else if (message.type === 'user_online') {
          console.log('User online:', message);
          setOnlineUsers(prev => new Set(prev).add(message.user_id!));
        } else if (message.type === 'user_offline') {
          console.log('User offline:', message);
          setOnlineUsers(prev => {
            const next = new Set(prev);
            next.delete(message.user_id!);
            return next;
          });
        } else if (message.type === 'room_presence') {
          console.log('Room presence:', message);
          setRoomPresence(prev => {
            const next = new Map(prev);
            next.set(message.room_id!, message.online_users || []);
            return next;
          });
          
          // Also update global onlineUsers set
          if (message.online_users) {
            setOnlineUsers(prev => {
              const next = new Set(prev);
              message.online_users!.forEach(uid => next.add(uid));
              return next;
            });
          }
        } else if (message.type === 'typing') {
          console.log('Typing:', message);
          if (message.is_typing) {
            setTypingUsers(prev => {
              const next = new Map(prev);
              const roomTyping = next.get(message.room_id!) || new Set();
              roomTyping.add(message.user_name || `User ${message.user_id}`);
              next.set(message.room_id!, roomTyping);
              return next;
            });
          } else {
            setTypingUsers(prev => {
              const next = new Map(prev);
              const roomTyping = next.get(message.room_id!);
              if (roomTyping) {
                roomTyping.delete(message.user_name || `User ${message.user_id}`);
                if (roomTyping.size === 0) {
                  next.delete(message.room_id!);
                }
              }
              return next;
            });
          }
        } else if (message.type === 'room_updated') {
          console.log('Room updated:', message);
          // Update room's last_message_at and re-sort
          setRooms(prev => {
            const updated = prev.map(r => 
              r.id === message.room_id 
                ? { ...r, last_message_at: message.last_message_at }
                : r
            );
            return updated.sort((a, b) => {
              const aTime = new Date(a.last_message_at || a.created_at).getTime();
              const bTime = new Date(b.last_message_at || b.created_at).getTime();
              return bTime - aTime;
            });
          });
        } else if (message.type === 'unread_updated') {
          console.log('Unread updated:', message);
          // Update unread count for room
          setRooms(prev => prev.map(r => 
            r.id === message.room_id 
              ? { ...r, unread_count: message.unread_count }
              : r
          ));
        } else if (message.type === 'connection_replaced') {
          console.log('Connection replaced:', message);
          // Show modal and disable interactions
          setConnectionReplaced(true);
          // Disconnect WebSocket
          if (newWs) {
            newWs.disconnect();
          }
        } else if (message.type === 'rate_limit_exceeded') {
          console.warn('Rate limit exceeded:', message);
          // Show temporary notification
          alert(`${message.message || 'Bạn đang thực hiện hành động quá nhanh.'}\nVui lòng đợi ${Math.ceil(message.retry_after || 5)} giây.`);
        }
      });
      
      setWs(newWs);
    } catch (error) {
      console.error('Failed to connect WebSocket:', error);
    }
  };

  const selectRoom = async (room: Room) => {
    setSelectedRoom(room);

    // Mark as read if has unread messages
    if (room.unread_count && room.unread_count > 0) {
      try {
        await markRoomAsRead(room.id);
        // Update local state
        setRooms(prev => prev.map(r => 
          r.id === room.id ? { ...r, unread_count: 0 } : r
        ));
      } catch (error) {
        console.error('Failed to mark as read:', error);
      }
    }

    // Load messages
    try {
      const msgs = await getRoomMessages(room.id);
      setMessages(msgs.reverse());
    } catch (error) {
      console.error('Failed to load messages:', error);
    }
  };

  const sendMessage = async () => {
    if (!newMessage.trim() || !selectedRoom || !ws || connectionReplaced) return;

    const tempId = `temp-${Date.now()}`;
    const tempMessage: Message = {
      id: tempId,
      room_id: selectedRoom.id,
      user_id: user!.id,
      sender_id: user!.id,
      sender_name: user!.name,
      content: newMessage,
      created_at: new Date().toISOString(),
    };

    // Add temporary message to UI
    setMessages(prev => [...prev, tempMessage]);
    
    // Send via WebSocket
    ws.send({
      type: 'message',
      room_id: selectedRoom.id,
      content: newMessage,
      message_type: 'text',
    });

    // Stop typing indicator
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
      typingTimeoutRef.current = null;
    }
    ws.send({
      type: 'typing',
      room_id: selectedRoom.id,
      is_typing: false,
    });

    setNewMessage('');
  };
  
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setNewMessage(e.target.value);
    
    if (!selectedRoom || !ws) return;
    
    // Send typing indicator
    ws.send({
      type: 'typing',
      room_id: selectedRoom.id,
      is_typing: true,
    });
    
    // Clear previous timeout
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }
    
    // Auto-stop typing after 3 seconds
    typingTimeoutRef.current = setTimeout(() => {
      ws.send({
        type: 'typing',
        room_id: selectedRoom.id,
        is_typing: false,
      });
    }, 3000);
  };

  const createDirect = async (otherUserId: number) => {
    try {
      const room = await createDirectRoom(otherUserId);
      await loadRooms();
      setShowFindFriends(false);
      selectRoom(room);
      
      // Join room via WebSocket
      if (ws && !joinedRoomsRef.current.has(room.id)) {
        ws.send({
          type: 'join_room',
          room_id: room.id,
        });
        joinedRoomsRef.current.add(room.id);
      }
    } catch (error) {
      console.error('Failed to create direct room:', error);
      alert('Không thể tạo cuộc trò chuyện');
    }
  };

  const createGroup = async () => {
    if (!groupName.trim() || selectedMembers.size === 0) {
      alert('Vui lòng nhập tên nhóm và chọn ít nhất 1 thành viên');
      return;
    }

    try {
      const room = await createGroupRoom(groupName, Array.from(selectedMembers));
      await loadRooms();
      setShowCreateGroup(false);
      setGroupName('');
      setSelectedMembers(new Set());
      selectRoom(room);
      
      // Join room via WebSocket
      if (ws && !joinedRoomsRef.current.has(room.id)) {
        ws.send({
          type: 'join_room',
          room_id: room.id,
        });
        joinedRoomsRef.current.add(room.id);
      }
    } catch (error) {
      console.error('Failed to create group:', error);
      alert('Không thể tạo nhóm');
    }
  };

  const handleAcceptInvitation = async (invitationId: number) => {
    try {
      const result = await acceptInvitation(invitationId);
      await loadInvitations();
      await loadRooms();
      
      // Join the room via WebSocket
      if (ws && result.room && !joinedRoomsRef.current.has(result.room.id)) {
        ws.send({
          type: 'join_room',
          room_id: result.room.id,
        });
        joinedRoomsRef.current.add(result.room.id);
      }
      
      alert('Đã chấp nhận lời mời');
    } catch (error) {
      console.error('Failed to accept invitation:', error);
      alert('Không thể chấp nhận lời mời');
    }
  };

  const handleDeclineInvitation = async (invitationId: number) => {
    try {
      await declineInvitation(invitationId);
      await loadInvitations();
      alert('Đã từ chối lời mời');
    } catch (error) {
      console.error('Failed to decline invitation:', error);
      alert('Không thể từ chối lời mời');
    }
  };

  const handleLeaveRoom = async (roomId: string) => {
    if (!confirm('Bạn có chắc muốn rời khỏi nhóm này?')) return;
    
    try {
      await leaveRoom(roomId);
      await loadRooms();
      
      if (selectedRoom?.id === roomId) {
        setSelectedRoom(null);
        setMessages([]);
      }
      
      alert('Đã rời khỏi nhóm');
    } catch (error: any) {
      console.error('Failed to leave room:', error);
      alert(error.response?.data?.error || 'Không thể rời nhóm');
    }
  };

  const handleHideRoom = async (roomId: string) => {
    try {
      await hideRoom(roomId);
      await loadRooms();
      
      if (selectedRoom?.id === roomId) {
        setSelectedRoom(null);
        setMessages([]);
      }
    } catch (error) {
      console.error('Failed to hide room:', error);
      alert('Không thể ẩn cuộc trò chuyện');
    }
  };

  const getRoomName = (room: Room): string => {
    if (room.room_type === 'group') {
      return room.name || 'Nhóm không tên';
    }
    
    // For direct chat, find the other user
    const otherMember = room.members?.find(m => m.user_id !== user?.id);
    if (otherMember?.user_name) {
      return otherMember.user_name;
    }
    
    // Fallback to usersMap
    const otherUser = room.members?.find(m => m.user_id !== user?.id);
    if (otherUser) {
      const userData = usersMap.get(otherUser.user_id);
      return userData?.name || 'Unknown User';
    }
    
    return 'Trò chuyện';
  };
  
  const getOtherUserId = (room: Room): number => {
    if (room.room_type === 'direct') {
      const otherMember = room.members?.find(m => m.user_id !== user?.id);
      return otherMember?.user_id || 0;
    }
    return 0;
  };

  const toggleMemberSelection = (userId: number) => {
    setSelectedMembers(prev => {
      const newSet = new Set(prev);
      if (newSet.has(userId)) {
        newSet.delete(userId);
      } else {
        newSet.add(userId);
      }
      return newSet;
    });
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-lg">Đang tải...</div>
      </div>
    );
  }

  return (
    <div className="flex h-screen bg-gray-100">
      {/* Sidebar */}
      <div className="w-80 bg-white border-r border-gray-200 flex flex-col">
        {/* Header */}
        <div className="p-4 border-b border-gray-200">
          <div className="flex items-center justify-between mb-4">
            <h1 className="text-xl font-bold text-gray-900">Chat</h1>
            <button
              onClick={() => {
                // Disconnect WebSocket first so others see user as offline
                if (ws) {
                  ws.disconnect();
                }
                localStorage.removeItem('token');
                localStorage.removeItem('user');
                router.push('/login');
              }}
              className="text-sm text-red-600 hover:text-red-700"
            >
              Đăng xuất
            </button>
          </div>
          <div className="text-sm text-gray-600">
            Xin chào, {user?.name}
          </div>
        </div>

        {/* Actions */}
        <div className="p-4 border-b border-gray-200 space-y-2">
          <button
            onClick={() => setShowFindFriends(true)}
            className="w-full px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700"
          >
            Tìm bạn bè
          </button>
          <button
            onClick={() => setShowCreateGroup(true)}
            className="w-full px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700"
          >
            Tạo nhóm chat
          </button>
          <button
            onClick={() => setShowInvitations(true)}
            className="w-full px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 relative"
          >
            Lời mời ({invitations.length})
            {invitations.length > 0 && (
              <span className="absolute top-1 right-1 w-2 h-2 bg-red-500 rounded-full"></span>
            )}
          </button>
        </div>

        {/* Rooms List */}
        <div className="flex-1 overflow-y-auto">
          {rooms.length === 0 ? (
            <div className="p-4 text-center text-gray-500">
              Chưa có cuộc trò chuyện nào
            </div>
          ) : (
            rooms.map(room => (
              <button
                key={room.id}
                onClick={() => selectRoom(room)}
                className={`w-full text-left p-3 border-b border-gray-100 hover:bg-gray-50 relative ${
                  selectedRoom?.id === room.id ? 'bg-indigo-50' : ''
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 flex-1">
                    <div className={`${room.unread_count && room.unread_count > 0 ? 'font-bold' : 'font-medium'} text-gray-900`}>
                      {getRoomName(room)}
                    </div>
                    {room.room_type === 'direct' && (
                      <span className={`w-2 h-2 rounded-full ${
                        onlineUsers.has(getOtherUserId(room)) ? 'bg-green-500' : 'bg-gray-300'
                      }`} />
                    )}
                    {room.room_type === 'group' && roomPresence.get(room.id) && (
                      <span className="text-xs text-gray-500">
                        ({roomPresence.get(room.id)?.length || 0}/{room.members.length})
                      </span>
                    )}
                  </div>
                  <div className="flex items-center gap-2">
                    {room.unread_count && room.unread_count > 0 && (
                      <>
                        <span className="bg-blue-500 text-white text-xs px-2 py-1 rounded-full">
                          {room.unread_count}
                        </span>
                        <div className="w-2.5 h-2.5 bg-blue-500 rounded-full"></div>
                      </>
                    )}
                  </div>
                </div>
                <div className="text-xs text-gray-500 mt-1">
                  {room.room_type === 'direct' ? 'Trò chuyện riêng' : 'Nhóm'}
                </div>
              </button>
            ))
          )}
        </div>
      </div>

      {/* Main Chat Area */}
      <div className="flex-1 flex flex-col">
        {selectedRoom ? (
          <>
            {/* Chat Header */}
            <div className="bg-white border-b border-gray-200 p-4 flex items-center justify-between">
              <div>
                <h2 className="text-lg font-semibold text-gray-900">
                  {getRoomName(selectedRoom)}
                </h2>
                <p className="text-sm text-gray-500">
                  {selectedRoom.room_type === 'group' ? `${selectedRoom.members?.length || 0} thành viên` : 'Trò chuyện riêng'}
                </p>
              </div>
              <div className="flex gap-2">
                {selectedRoom.room_type === 'group' && (
                  <button
                    onClick={() => handleLeaveRoom(selectedRoom.id)}
                    className="px-3 py-1 text-sm bg-red-100 text-red-700 rounded hover:bg-red-200"
                  >
                    Rời nhóm
                  </button>
                )}
                <button
                  onClick={() => handleHideRoom(selectedRoom.id)}
                  className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200"
                >
                  Ẩn
                </button>
              </div>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-4 space-y-4">
              {messages.map((msg) => {
                const isOwn = msg.sender_id === user?.id || msg.user_id === user?.id;
                const senderName = msg.sender_name || usersMap.get(msg.sender_id || msg.user_id)?.name || 'Unknown';
                const isSystem = msg.sender_id === 0 || msg.sender_name === 'System';
                
                // System message (centered, gray)
                if (isSystem) {
                  return (
                    <div key={msg.id} className="flex justify-center">
                      <div className="bg-gray-100 text-gray-600 text-sm rounded-full px-4 py-1">
                        {msg.content}
                      </div>
                    </div>
                  );
                }
                
                // Regular user message
                return (
                  <div
                    key={msg.id}
                    className={`flex ${isOwn ? 'justify-end' : 'justify-start'}`}
                  >
                    <div className={`max-w-xs lg:max-w-md ${isOwn ? 'bg-indigo-600 text-white' : 'bg-white text-gray-900'} rounded-lg px-4 py-2 shadow`}>
                      {!isOwn && selectedRoom.room_type === 'group' && (
                        <div className="text-xs font-semibold mb-1 opacity-75">
                          {senderName}
                        </div>
                      )}
                      <div>{msg.content}</div>
                      <div className={`text-xs mt-1 ${isOwn ? 'text-indigo-200' : 'text-gray-500'}`}>
                        {new Date(msg.created_at).toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit' })}
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>

            {/* Typing Indicator */}
            {selectedRoom && typingUsers.get(selectedRoom.id) && typingUsers.get(selectedRoom.id)!.size > 0 && (
              <div className="px-4 py-2 text-sm text-gray-500 italic">
                {Array.from(typingUsers.get(selectedRoom.id)!).join(', ')} {
                  typingUsers.get(selectedRoom.id)!.size === 1 ? 'đang soạn tin' : 'đang soạn tin'
                }...
              </div>
            )}

            {/* Message Input */}
            <div className="bg-white border-t border-gray-200 p-4">
              <div className="flex gap-2">
                <input
                  type="text"
                  value={newMessage}
                  onChange={handleInputChange}
                  onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
                  placeholder={connectionReplaced ? "Phiên đã bị thay thế..." : "Nhập tin nhắn..."}
                  disabled={connectionReplaced}
                  className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:bg-gray-100 disabled:cursor-not-allowed"
                />
                <button
                  onClick={sendMessage}
                  disabled={connectionReplaced}
                  className="px-6 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
                >
                  Gửi
                </button>
              </div>
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center text-gray-500">
            Chọn một cuộc trò chuyện để bắt đầu
          </div>
        )}
      </div>

      {/* Find Friends Modal */}
      {showFindFriends && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-96 max-h-96 overflow-y-auto">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-semibold">Tìm bạn bè</h3>
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
                  onClick={() => createDirect(u.id)}
                  className="w-full text-left p-3 border border-gray-200 rounded-lg hover:bg-gray-50"
                >
                  <div className="font-medium">{u.name}</div>
                  <div className="text-sm text-gray-500">{u.email}</div>
                </button>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Create Group Modal */}
      {showCreateGroup && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-96 max-h-96 overflow-y-auto">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-semibold">Tạo nhóm chat</h3>
              <button
                onClick={() => {
                  setShowCreateGroup(false);
                  setGroupName('');
                  setSelectedMembers(new Set());
                }}
                className="text-gray-500 hover:text-gray-700"
              >
                ✕
              </button>
            </div>
            <input
              type="text"
              value={groupName}
              onChange={(e) => setGroupName(e.target.value)}
              placeholder="Tên nhóm"
              className="w-full px-4 py-2 border border-gray-300 rounded-lg mb-4 focus:outline-none focus:ring-2 focus:ring-green-500"
            />
            <div className="mb-4">
              <div className="text-sm font-medium text-gray-700 mb-2">
                Chọn thành viên ({selectedMembers.size})
              </div>
              <div className="space-y-2">
                {allUsers.map(u => (
                  <button
                    key={u.id}
                    onClick={() => toggleMemberSelection(u.id)}
                    className={`w-full text-left p-3 border-2 rounded-lg transition-colors ${
                      selectedMembers.has(u.id)
                        ? 'border-green-500 bg-green-50'
                        : 'border-gray-200 hover:bg-gray-50'
                    }`}
                  >
                    <div className="font-medium">{u.name}</div>
                    <div className="text-sm text-gray-500">{u.email}</div>
                  </button>
                ))}
              </div>
            </div>
            <button
              onClick={createGroup}
              className="w-full px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700"
            >
              Tạo nhóm
            </button>
          </div>
        </div>
      )}

      {/* Invitations Modal */}
      {showInvitations && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-96 max-h-96 overflow-y-auto">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-semibold">Lời mời tham gia nhóm</h3>
              <button
                onClick={() => setShowInvitations(false)}
                className="text-gray-500 hover:text-gray-700"
              >
                ✕
              </button>
            </div>
            {invitations.length === 0 ? (
              <div className="text-center text-gray-500 py-4">
                Không có lời mời nào
              </div>
            ) : (
              <div className="space-y-3">
                {invitations.map(inv => (
                  <div key={inv.id} className="border border-gray-200 rounded-lg p-3">
                    <div className="font-medium text-gray-900 mb-1">
                      {inv.room_name || 'Nhóm không tên'}
                    </div>
                    <div className="text-sm text-gray-600 mb-3">
                      Được mời bởi: {inv.invited_by_name}
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={() => handleAcceptInvitation(inv.id)}
                        className="flex-1 px-3 py-2 bg-green-600 text-white rounded hover:bg-green-700 text-sm"
                      >
                        Chấp nhận
                      </button>
                      <button
                        onClick={() => handleDeclineInvitation(inv.id)}
                        className="flex-1 px-3 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300 text-sm"
                      >
                        Từ chối
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
      
      {/* Connection Replaced Modal */}
      {connectionReplaced && (
        <div className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-8 max-w-md text-center">
            <div className="mb-4">
              <svg className="mx-auto h-12 w-12 text-yellow-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
            </div>
            <h3 className="text-xl font-semibold text-gray-900 mb-2">
              Phiên đã bị thay thế
            </h3>
            <p className="text-gray-600 mb-6">
              Phiên của bạn đã bị thay thế bởi một đăng nhập mới. Vui lòng tải lại trang để tiếp tục.
            </p>
            <button
              onClick={() => window.location.reload()}
              className="w-full px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 font-medium"
            >
              Tải lại trang
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
