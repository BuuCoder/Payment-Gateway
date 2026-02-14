import { NextRequest, NextResponse } from 'next/server';

export async function GET(
  request: NextRequest,
  context: { params: Promise<{ roomId: string }> }
) {
  try {
    const token = request.headers.get('authorization');
    const { searchParams } = new URL(request.url);
    const limit = searchParams.get('limit') || '50';
    
    if (!token) {
      console.error('No authorization token provided');
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }
    
    const { roomId } = await context.params;
    
    console.log(`Fetching messages for room ${roomId} with token: ${token.substring(0, 20)}...`);
    
    const url = `${process.env.CHAT_SERVICE_URL}/api/rooms/${roomId}/messages?limit=${limit}`;
    console.log(`Calling: ${url}`);
    
    const response = await fetch(url, {
      headers: {
        'Authorization': token,
      },
    });

    const data = await response.json();

    if (!response.ok) {
      console.error(`Chat service returned ${response.status}:`, data);
      return NextResponse.json(data, { status: response.status });
    }

    return NextResponse.json(data);
  } catch (error) {
    console.error('Get messages error:', error);
    return NextResponse.json(
      { error: 'Failed to get messages' },
      { status: 500 }
    );
  }
}
