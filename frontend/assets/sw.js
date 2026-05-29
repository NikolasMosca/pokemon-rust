const CACHE_NAME = 'pokemon-rust-v1';

// Solo asset statici — mai intercettare PokéAPI o richieste di rete esterne
const STATIC_EXTENSIONS = [
  '.wasm', '.js', '.css', '.png', '.jpg', '.webp',
  '.ttf', '.ogg', '.m4a', '.ico'
];

function isStaticAsset(url) {
  const u = new URL(url);
  // Non intercettare mai richieste esterne (PokéAPI, CDN)
  if (u.hostname !== self.location.hostname) return false;
  return STATIC_EXTENSIONS.some(ext => u.pathname.endsWith(ext));
}

self.addEventListener('install', event => {
  self.skipWaiting();
});

self.addEventListener('activate', event => {
  event.waitUntil(
    caches.keys().then(keys =>
      Promise.all(
        keys
          .filter(key => key !== CACHE_NAME)
          .map(key => caches.delete(key))
      )
    ).then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', event => {
  if (!isStaticAsset(event.request.url)) return;

  event.respondWith(
    caches.match(event.request).then(cached => {
      if (cached) return cached;
      return fetch(event.request).then(response => {
        if (!response || response.status !== 200) return response;
        const clone = response.clone();
        caches.open(CACHE_NAME).then(cache => cache.put(event.request, clone));
        return response;
      });
    })
  );
});
