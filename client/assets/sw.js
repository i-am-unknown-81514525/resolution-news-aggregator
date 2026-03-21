var cacheName = 'egui-news-agg';
var filesToCache = [
    './',
    './index.html',
    './new-aggregator-client_bg.wasm',
    './new-aggregator-client.js'
];

self.addEventListener('install', (event) => {
    event.waitUntil(
        (async () => {
            const cache = await caches.open(cacheName);

            // 1. Fetch current index.html to find the new hashes
            const response = await fetch('./index.html');
            const html = await response.text();

            // 2. Regex to find the hashed .js and .wasm files Trunk created
            // It searches for strings like "news-aggregator-client-f1e2d3..._bg.wasm"
            const hashRegex = /"([^"]+\.[a-f0-9]{16}(?:_bg)?\.(?:js|wasm|css))"/g;
            let match;
            const assetsToCache = ['./', './index.html'];

            while ((match = hashRegex.exec(html)) !== null) {
                assetsToCache.push(match[1]); // match[1] is the captured filename
            }

            // 3. Cache the newly discovered assets
            return cache.addAll(assetsToCache);
        })()
    );
});

/* Serve cached content when offline */
self.addEventListener('fetch', (event) => {
    event.respondWith(
        caches.match(event.request).then((response) => {
            return response || fetch(event.request);
        })
    );
});