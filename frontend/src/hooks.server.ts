import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
	// API requests proxy logic for production/public access
	if (event.url.pathname.startsWith('/api')) {
		// Determine the backend target
		// In Docker, the backend service is reachable at http://backend:8080
		const backendUrl = 'http://backend:8080';
		const targetUrl = new URL(event.url.pathname + event.url.search, backendUrl);

		console.log(`[Proxy] ${event.request.method} ${event.url.pathname} -> ${targetUrl.toString()}`);

		try {
			const requestInit: RequestInit = {
				method: event.request.method,
				headers: event.request.headers,
				// @ts-ignore - duplex is needed for streaming bodies in Bun/Node
				duplex: 'half'
			};

			// Only add body for non-GET/HEAD requests
			if (event.request.method !== 'GET' && event.request.method !== 'HEAD') {
				requestInit.body = event.request.body;
			}

			const response = await fetch(targetUrl.toString(), requestInit);
			
			// We need to create a new Response object to return, 
			// because the original response headers might be immutable or need adjustment
			const responseHeaders = new Headers(response.headers);
			// CORS headers are usually handled by the backend, but if needed we can add them here
			
			return new Response(response.body, {
				status: response.status,
				statusText: response.statusText,
				headers: responseHeaders
			});
		} catch (e) {
			console.error('[Proxy Error]', e);
			return new Response('Backend Connection Error', { status: 502 });
		}
	}

	return resolve(event);
};
