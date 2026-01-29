import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
	const isHttps =
		event.url.protocol === 'https:' ||
		event.request.headers.get('x-forwarded-proto') === 'https';
	const connectSrc = new Set([
		"'self'",
		'https://generativelanguage.googleapis.com',
		'ws:',
		'wss:'
	]);
	const envApiUrl = process.env.VITE_API_URL;
	if (envApiUrl) {
		try {
			const origin = new URL(envApiUrl).origin;
			connectSrc.add(origin);
		} catch {
			// ignore invalid env values
		}
	}
	const localHosts = new Set(['localhost', '127.0.0.1', '0.0.0.0', '::1']);
	if (localHosts.has(event.url.hostname)) {
		connectSrc.add('http://localhost:8080');
		connectSrc.add('http://127.0.0.1:8080');
		connectSrc.add('http://0.0.0.0:8080');
	}
	const frameSrc = [
		"'self'",
		'https://dartpad.dev',
		'https://play.golang.org',
		'https://go.dev',
		'https://pyodide.org',
		'https://jupyterlite.github.io'
	];
	const csp = [
		"default-src 'self'",
		"base-uri 'self'",
		"frame-ancestors 'none'",
		"object-src 'none'",
		"img-src 'self' data: blob:",
		"script-src 'self' 'unsafe-inline'",
		"style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
		"font-src 'self' https://fonts.gstatic.com",
		`frame-src ${frameSrc.join(' ')}`,
		`connect-src ${Array.from(connectSrc).join(' ')}`,
		"form-action 'self'"
	].join('; ');

	const applySecurityHeaders = (headers: Headers, isApi: boolean) => {
		if (!headers.has('x-content-type-options')) {
			headers.set('x-content-type-options', 'nosniff');
		}
		if (!headers.has('x-frame-options')) {
			headers.set('x-frame-options', 'DENY');
		}
		if (!headers.has('referrer-policy')) {
			headers.set('referrer-policy', 'strict-origin-when-cross-origin');
		}
		if (!headers.has('permissions-policy')) {
			headers.set('permissions-policy', 'camera=(), microphone=(), geolocation=()');
		}
		if (isHttps && !headers.has('strict-transport-security')) {
			headers.set('strict-transport-security', 'max-age=63072000; includeSubDomains; preload');
		}
		if (!headers.has('content-security-policy')) {
			headers.set('content-security-policy', isApi ? "default-src 'none'" : csp);
		}
	};

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
			applySecurityHeaders(responseHeaders, true);
			
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

	const response = await resolve(event);
	applySecurityHeaders(response.headers, false);
	return response;
};
