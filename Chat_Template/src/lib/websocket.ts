import { WsClientMessage, WsServerMessage } from '@/types';

export class ChatWebSocket {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 3000;
  private messageHandlers: ((message: WsServerMessage) => void)[] = [];
  private shouldReconnect = true; // Flag để kiểm soát reconnect

  constructor(private token: string) {}

  connectGlobal() {
    const wsUrl = `${process.env.NEXT_PUBLIC_CHAT_WS_URL}/api/ws?token=${encodeURIComponent(this.token)}`;
    
    console.log('🔌 Connecting to WebSocket globally...');
    console.log('WS URL:', wsUrl.substring(0, 50) + '...');
    
    this.shouldReconnect = true;
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      console.log('✅ WebSocket connected successfully');
      this.reconnectAttempts = 0;
      // Không join room ở đây, sẽ join từ bên ngoài
    };

    this.ws.onmessage = (event) => {
      try {
        console.log('📨 Raw WebSocket data:', event.data);
        const message: WsServerMessage = JSON.parse(event.data);
        console.log('📬 Parsed WebSocket message:', message);
        this.messageHandlers.forEach(handler => {
          try {
            handler(message);
          } catch (error) {
            console.error('❌ Error in message handler:', error);
          }
        });
      } catch (error) {
        console.error('❌ Failed to parse WebSocket message:', error);
        console.error('Raw data was:', event.data);
      }
    };

    this.ws.onerror = (error) => {
      console.error('❌ WebSocket error:', error);
    };

    this.ws.onclose = (event) => {
      console.log('🔌 WebSocket closed:', {
        code: event.code,
        reason: event.reason,
        wasClean: event.wasClean
      });
      
      if (event.code === 1005) {
        console.log('WebSocket closed normally (no status code)');
      } else if (event.code === 1006) {
        console.error('⚠️ WebSocket closed abnormally (1006) - possibly authentication failed or server not reachable');
      } else if (event.code === 1000) {
        console.log('WebSocket closed normally');
      } else {
        console.error(`WebSocket closed with code ${event.code}: ${event.reason}`);
      }
      
      if (this.shouldReconnect) {
        this.attemptReconnectGlobal();
      } else {
        console.log('Reconnect disabled, not attempting to reconnect');
      }
    };
  }

  private attemptReconnectGlobal() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
      setTimeout(() => this.connectGlobal(), this.reconnectDelay);
    } else {
      console.error('Max reconnection attempts reached');
    }
  }

  connect(roomId: string) {
    const wsUrl = `${process.env.NEXT_PUBLIC_CHAT_WS_URL}/api/ws?token=${encodeURIComponent(this.token)}`;
    
    console.log('🔌 Connecting to WebSocket...');
    console.log('WS URL:', wsUrl.substring(0, 50) + '...');
    
    this.shouldReconnect = true; // Enable reconnect
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      console.log('✅ WebSocket connected successfully');
      this.reconnectAttempts = 0;
      
      // Join room after connection
      console.log('📨 Sending join_room message for room:', roomId);
      this.send({
        type: 'join_room',
        room_id: roomId,
      });
    };

    this.ws.onmessage = (event) => {
      try {
        console.log('📨 Raw WebSocket data:', event.data);
        const message: WsServerMessage = JSON.parse(event.data);
        console.log('📬 Parsed WebSocket message:', message);
        this.messageHandlers.forEach(handler => {
          try {
            handler(message);
          } catch (error) {
            console.error('❌ Error in message handler:', error);
          }
        });
      } catch (error) {
        console.error('❌ Failed to parse WebSocket message:', error);
        console.error('Raw data was:', event.data);
      }
    };

    this.ws.onerror = (error) => {
      console.error('❌ WebSocket error:', error);
    };

    this.ws.onclose = (event) => {
      console.log('🔌 WebSocket closed:', {
        code: event.code,
        reason: event.reason,
        wasClean: event.wasClean
      });
      
      if (event.code === 1005) {
        console.log('WebSocket closed normally (no status code)');
      } else if (event.code === 1006) {
        console.error('⚠️ WebSocket closed abnormally (1006) - possibly authentication failed or server not reachable');
      } else if (event.code === 1000) {
        console.log('WebSocket closed normally');
      } else {
        console.error(`WebSocket closed with code ${event.code}: ${event.reason}`);
      }
      
      // Only reconnect if shouldReconnect flag is true
      if (this.shouldReconnect) {
        this.attemptReconnect(roomId);
      } else {
        console.log('Reconnect disabled, not attempting to reconnect');
      }
    };
  }

  private attemptReconnect(roomId: string) {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
      setTimeout(() => this.connect(roomId), this.reconnectDelay);
    } else {
      console.error('Max reconnection attempts reached');
    }
  }

  send(message: WsClientMessage) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      console.log('📤 Sending WebSocket message:', message);
      this.ws.send(JSON.stringify(message));
    } else {
      console.error('❌ WebSocket is not connected. ReadyState:', this.ws?.readyState);
      console.error('ReadyState meanings: 0=CONNECTING, 1=OPEN, 2=CLOSING, 3=CLOSED');
    }
  }

  onMessage(handler: (message: WsServerMessage) => void) {
    this.messageHandlers.push(handler);
  }

  disconnect() {
    console.log('🔌 Manually disconnecting WebSocket');
    this.shouldReconnect = false; // Disable reconnect
    if (this.ws) {
      this.ws.close(1000, 'Client disconnecting'); // Normal closure
      this.ws = null;
    }
  }
}
