# Direct Chat Demo - Chat 1-1 riêng tư

## Scenario: Alice và Bob chat riêng, Charlie không thể xem

### Step 1: Alice tạo direct chat với Bob

```bash
# Alice (user_id: 1) login và lấy JWT token
ALICE_TOKEN="eyJhbGc..."

# Alice tạo direct room với Bob (user_id: 2)
curl -X POST http://localhost:8085/api/rooms \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "room_type": "direct",
    "member_ids": [2]
  }'

# Response:
{
  "id": "abc-123-def",
  "name": null,
  "room_type": "direct",
  "created_by": 1,
  "members": [
    {"user_id": 1, "role": "admin", "joined_at": "2024-01-01T00:00:00Z"},
    {"user_id": 2, "role": "member", "joined_at": "2024-01-01T00:00:00Z"}
  ],
  "created_at": "2024-01-01T00:00:00Z"
}
```

### Step 2: Alice và Bob chat qua WebSocket

```javascript
// Alice's WebSocket
const aliceWs = new WebSocket('ws://localhost:8085/api/ws');

aliceWs.onopen = () => {
  // Join room
  aliceWs.send(JSON.stringify({
    type: 'join_room',
    room_id: 'abc-123-def'
  }));
  
  // Send message
  aliceWs.send(JSON.stringify({
    type: 'message',
    room_id: 'abc-123-def',
    content: 'Hi Bob! This is private.',
    message_type: 'text'
  }));
};

// Bob's WebSocket
const bobWs = new WebSocket('ws://localhost:8085/api/ws');

bobWs.onopen = () => {
  bobWs.send(JSON.stringify({
    type: 'join_room',
    room_id: 'abc-123-def'
  }));
};

bobWs.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  console.log('Bob received:', msg);
  // Output: {type: "message", content: "Hi Bob! This is private.", sender_id: 1, ...}
};
```

### Step 3: Charlie cố đọc tin nhắn (FAIL ❌)

```bash
# Charlie (user_id: 3) login
CHARLIE_TOKEN="eyJhbGc..."

# Charlie cố đọc messages của Alice và Bob
curl http://localhost:8085/api/rooms/abc-123-def/messages \
  -H "Authorization: Bearer $CHARLIE_TOKEN"

# Response: 403 Forbidden
{
  "error": "Not a member of this room"
}
```

### Step 4: Charlie cố join WebSocket room (FAIL ❌)

```javascript
// Charlie's WebSocket
const charlieWs = new WebSocket('ws://localhost:8085/api/ws');

charlieWs.onopen = () => {
  charlieWs.send(JSON.stringify({
    type: 'join_room',
    room_id: 'abc-123-def'  // Room của Alice và Bob
  }));
};

charlieWs.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  console.log('Charlie received:', msg);
  // Output: {type: "error", message: "Not a member of this room"}
};
```

### Step 5: Charlie cố gửi message (FAIL ❌)

```javascript
charlieWs.send(JSON.stringify({
  type: 'message',
  room_id: 'abc-123-def',
  content: 'I am hacking!',
  message_type: 'text'
}));

// Response:
// {type: "error", message: "Not a member of this room"}
// Message KHÔNG được save vào database
// Alice và Bob KHÔNG nhận được message này
```

## Kết quả:

✅ **Alice và Bob:**
- Tạo được direct room
- Chat được với nhau
- Chỉ 2 người thấy messages

❌ **Charlie:**
- KHÔNG đọc được messages
- KHÔNG join được room
- KHÔNG gửi được messages

## Database State:

```sql
-- chat_rooms table
| id          | name | room_type | created_by |
|-------------|------|-----------|------------|
| abc-123-def | NULL | direct    | 1          |

-- chat_room_members table
| room_id     | user_id | role   |
|-------------|---------|--------|
| abc-123-def | 1       | admin  |
| abc-123-def | 2       | member |
-- Charlie (user_id: 3) KHÔNG có trong table này!

-- chat_messages table
| id  | room_id     | sender_id | content                  |
|-----|-------------|-----------|--------------------------|
| m1  | abc-123-def | 1         | Hi Bob! This is private. |
| m2  | abc-123-def | 2         | Hi Alice!                |
-- Charlie KHÔNG thể query được messages này vì không phải member
```

## Full HTML Demo:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Direct Chat Demo</title>
</head>
<body>
    <h1>Direct Chat Security Demo</h1>
    
    <div id="alice">
        <h2>Alice (User 1)</h2>
        <button onclick="aliceCreateRoom()">Create Direct Chat with Bob</button>
        <button onclick="aliceConnect()">Connect WebSocket</button>
        <input id="aliceMsg" placeholder="Message to Bob">
        <button onclick="aliceSend()">Send</button>
        <div id="aliceMessages"></div>
    </div>
    
    <div id="bob">
        <h2>Bob (User 2)</h2>
        <button onclick="bobConnect()">Connect WebSocket</button>
        <div id="bobMessages"></div>
    </div>
    
    <div id="charlie">
        <h2>Charlie (User 3) - Hacker</h2>
        <button onclick="charlieConnect()">Try to Connect</button>
        <button onclick="charlieHack()">Try to Read Messages</button>
        <div id="charlieMessages" style="color: red;"></div>
    </div>

    <script>
        let aliceWs, bobWs, charlieWs;
        let roomId = null;
        
        const ALICE_TOKEN = 'alice-jwt-token';
        const BOB_TOKEN = 'bob-jwt-token';
        const CHARLIE_TOKEN = 'charlie-jwt-token';
        
        async function aliceCreateRoom() {
            const response = await fetch('http://localhost:8085/api/rooms', {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${ALICE_TOKEN}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    room_type: 'direct',
                    member_ids: [2]
                })
            });
            
            const room = await response.json();
            roomId = room.id;
            document.getElementById('aliceMessages').innerHTML += 
                `<p>✅ Created room: ${roomId}</p>`;
        }
        
        function aliceConnect() {
            aliceWs = new WebSocket('ws://localhost:8085/api/ws');
            
            aliceWs.onopen = () => {
                aliceWs.send(JSON.stringify({
                    type: 'join_room',
                    room_id: roomId
                }));
            };
            
            aliceWs.onmessage = (event) => {
                const msg = JSON.parse(event.data);
                document.getElementById('aliceMessages').innerHTML += 
                    `<p>${JSON.stringify(msg)}</p>`;
            };
        }
        
        function aliceSend() {
            const content = document.getElementById('aliceMsg').value;
            aliceWs.send(JSON.stringify({
                type: 'message',
                room_id: roomId,
                content: content,
                message_type: 'text'
            }));
        }
        
        function bobConnect() {
            bobWs = new WebSocket('ws://localhost:8085/api/ws');
            
            bobWs.onopen = () => {
                bobWs.send(JSON.stringify({
                    type: 'join_room',
                    room_id: roomId
                }));
            };
            
            bobWs.onmessage = (event) => {
                const msg = JSON.parse(event.data);
                document.getElementById('bobMessages').innerHTML += 
                    `<p>✅ ${JSON.stringify(msg)}</p>`;
            };
        }
        
        function charlieConnect() {
            charlieWs = new WebSocket('ws://localhost:8085/api/ws');
            
            charlieWs.onopen = () => {
                charlieWs.send(JSON.stringify({
                    type: 'join_room',
                    room_id: roomId
                }));
            };
            
            charlieWs.onmessage = (event) => {
                const msg = JSON.parse(event.data);
                document.getElementById('charlieMessages').innerHTML += 
                    `<p>❌ ${JSON.stringify(msg)}</p>`;
            };
        }
        
        async function charlieHack() {
            try {
                const response = await fetch(
                    `http://localhost:8085/api/rooms/${roomId}/messages`,
                    {
                        headers: {
                            'Authorization': `Bearer ${CHARLIE_TOKEN}`
                        }
                    }
                );
                
                const data = await response.json();
                document.getElementById('charlieMessages').innerHTML += 
                    `<p>❌ ${response.status}: ${JSON.stringify(data)}</p>`;
            } catch (error) {
                document.getElementById('charlieMessages').innerHTML += 
                    `<p>❌ Error: ${error.message}</p>`;
            }
        }
    </script>
</body>
</html>
```

## Kết luận:

🔒 **Bảo mật hoàn toàn!**
- Direct chat chỉ 2 người
- Người thứ 3 KHÔNG thể xem/gửi messages
- Bảo vệ ở cả REST API và WebSocket
- Database-level security
