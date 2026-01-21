import { createApiClient } from './api-client/client.ts';
import { z } from 'zod';

async function test() {
  const client = createApiClient({ baseUrl: 'http://localhost:9867' });

  // Start recording
  await client.post('/api/macros/start-recording');
  console.log('First call succeeded');

  // Try to start again (should get 400)
  try {
    const response = await client.customRequest(
      'POST',
      '/api/macros/start-recording',
      z.any()
    );
    console.log('Second call succeeded:', response);
    console.log('Status:', response.status);
    console.log('Data:', response.data);
  } catch (error: any) {
    console.log('Second call failed');
    console.log('Error type:', error.constructor.name);
    console.log('Has statusCode?', 'statusCode' in error);
    console.log('statusCode:', error.statusCode);
    console.log('response:', error.response);
  }
}

test().catch(console.error);
