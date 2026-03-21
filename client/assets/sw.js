var cacheName = 'egui-news-agg';
var filesToCache = [
    './',
    './index.html',
    "./assets/NotoSerifCJK-VF.otf.ttc"
];

self.addEventListener('install', (event) => {
    event.waitUntil(
        (async () => {
            const cache = await caches.open(cacheName);

            // 1. Fetch current index.html to find the new hashes
            const response = await fetch('./index.html');
            const html = await response.text();

            const hashRegex = /"(news-aggregator-client-[a-f0-9]{16}(?:_bg)?\.(?:js|wasm|css))"/g;

            let match;
            const assetsToCache = [...filesToCache];

            while ((match = hashRegex.exec(html)) !== null) {
                assetsToCache.push(`./${match[1]}`);
            }

            return cache.addAll(assetsToCache);
        })()
    );
});

/* Serve cached content when offline */
self.addEventListener('fetch', (event) => {
    if (event.request.mode === 'navigate') {
        event.respondWith(
            fetch(event.request)
                .catch(() => caches.match(event.request))
        );
        return;
    }
    event.respondWith(
        caches.match(event.request).then((response) => {
            // Return cached version, or fetch from network if missing
            return response || fetch(event.request);
        })
    );
});