import { NextRequest } from 'next/server';

export async function GET(request: NextRequest) {
  const upgradeHeader = request.headers.get('upgrade');
  
  if (upgradeHeader !== 'websocket') {
    return new Response('Expected WebSocket', { status: 426 });
  }

  // Next.js Edge Runtime không hỗ trợ WebSocket upgrade
  // Cần dùng Node.js runtime hoặc external WebSocket server
  
  return new Response(
    'WebSocket proxy requires custom server. Use token in query string instead: ws://localhost:8085/api/ws?token=YOUR_TOKEN',
    { status: 501 }
  );
}

export const runtime = 'edge';
