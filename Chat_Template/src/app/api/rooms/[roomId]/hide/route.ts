import { NextRequest, NextResponse } from 'next/server';

const CHAT_SERVICE_URL = process.env.CHAT_SERVICE_URL || 'http://localhost:8084';

export async function POST(
  request: NextRequest,
  context: { params: Promise<{ roomId: string }> }
) {
  try {
    const token = request.headers.get('authorization');
    
    if (!token) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { roomId } = await context.params;

    const response = await fetch(
      `${CHAT_SERVICE_URL}/api/rooms/${roomId}/hide`,
      {
        method: 'POST',
        headers: {
          'Authorization': token,
          'Content-Type': 'application/json',
        },
      }
    );

    // Check if response is JSON
    const contentType = response.headers.get('content-type');
    if (!contentType || !contentType.includes('application/json')) {
      const text = await response.text();
      console.error('Non-JSON response from chat service:', text);
      return NextResponse.json(
        { error: 'Invalid response from chat service', details: text },
        { status: 500 }
      );
    }

    const data = await response.json();

    if (!response.ok) {
      return NextResponse.json(data, { status: response.status });
    }

    return NextResponse.json(data);
  } catch (error) {
    console.error('Error hiding room:', error);
    return NextResponse.json(
      { error: 'Failed to hide room' },
      { status: 500 }
    );
  }
}
