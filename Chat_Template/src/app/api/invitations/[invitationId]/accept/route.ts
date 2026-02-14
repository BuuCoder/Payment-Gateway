import { NextRequest, NextResponse } from 'next/server';

const CHAT_SERVICE_URL = process.env.CHAT_SERVICE_URL || 'http://localhost:8084';

export async function POST(
  request: NextRequest,
  context: { params: Promise<{ invitationId: string }> }
) {
  try {
    const token = request.headers.get('authorization');
    
    if (!token) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const { invitationId } = await context.params;

    const response = await fetch(
      `${CHAT_SERVICE_URL}/api/invitations/${invitationId}/accept`,
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
    console.error('Error accepting invitation:', error);
    return NextResponse.json(
      { error: 'Failed to accept invitation' },
      { status: 500 }
    );
  }
}
