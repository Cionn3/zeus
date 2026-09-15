const NATIVE_HOST = 'io.github.zeus_wallet';
const DEFAULT_PORT = 65534;

const CONNECTION_REQUEST_TIMEOUT_MS = 30000;
const POLLING_INTERVAL_MS = 1000;
const SESSION_RETRY_MS = 5000;

let cachedSession = null;
let sessionLoadFailedAt = 0;

let lastKnownAccounts = null;
let lastKnownChainId = null;
let isFirstPoll = true;
let lastKnownConnectedOrigins = JSON.stringify([]);
let pollInFlight = false;

function serverUrls(port) {
    const p = port || DEFAULT_PORT;
    return {
        status: `http://127.0.0.1:${p}/status`,
        api: `http://127.0.0.1:${p}/api`,
    };
}

function loadSession() {
    return new Promise((resolve, reject) => {
        chrome.runtime.sendNativeMessage(NATIVE_HOST, { cmd: 'session' }, (response) => {
            if (chrome.runtime.lastError) {
                cachedSession = null;
                reject(new Error(chrome.runtime.lastError.message));
                return;
            }
            if (!response || !response.token) {
                cachedSession = null;
                reject(new Error('Zeus connector session missing token'));
                return;
            }
            cachedSession = {
                token: response.token,
                port: response.port || DEFAULT_PORT,
            };
            resolve(cachedSession);
        });
    });
}

async function getSession(force) {
    if (!force && cachedSession) return cachedSession;
    if (!force && sessionLoadFailedAt && (Date.now() - sessionLoadFailedAt) < SESSION_RETRY_MS) {
        throw new Error('Zeus connector session unavailable');
    }
    try {
        const session = await loadSession();
        sessionLoadFailedAt = 0;
        return session;
    } catch (e) {
        sessionLoadFailedAt = Date.now();
        throw e;
    }
}

function tabOrigin(sender) {
    const raw = sender && (sender.tab && sender.tab.url ? sender.tab.url : sender.url);
    if (!raw) return null;
    try {
        return new URL(raw).origin;
    } catch {
        return null;
    }
}

function stripBodyOrigin(options) {
    const next = Object.assign({}, options || {});
    if (typeof next.body !== 'string') return next;
    try {
        const parsed = JSON.parse(next.body);
        if (parsed && typeof parsed === 'object') {
            delete parsed.origin;
            next.body = JSON.stringify(parsed);
        }
    } catch {
        // leave body as-is
    }
    return next;
}

async function authorizedFetch(url, options, origin) {
    let session = await getSession(false);
    const build = (sess) => {
        const urls = serverUrls(sess.port);
        let targetUrl;
        if (url === '/status') targetUrl = urls.status;
        else if (url === '/api') targetUrl = urls.api;
        else throw new Error('Unknown connector path');

        const headers = Object.assign({}, (options && options.headers) || {}, {
            'X-Zeus-Token': sess.token,
        });
        if (origin) headers['X-Zeus-Origin'] = origin;

        return { targetUrl, fetchOpts: Object.assign({}, stripBodyOrigin(options), { headers }) };
    };

    let { targetUrl, fetchOpts } = build(session);
    let response;
    try {
        response = await fetch(targetUrl, fetchOpts);
    } catch (e) {
        // Zeus may have rebound on another port; reload connector.json.
        session = await getSession(true);
        ({ targetUrl, fetchOpts } = build(session));
        response = await fetch(targetUrl, fetchOpts);
    }
    if (response.status === 401) {
        session = await getSession(true);
        ({ targetUrl, fetchOpts } = build(session));
        response = await fetch(targetUrl, fetchOpts);
    }
    return response;
}

async function pollServerStatus() {
    if (pollInFlight) return;
    pollInFlight = true;
    try {
        const response = await authorizedFetch('/status', { method: 'GET' });
        if (!response.ok) return;
        const currentState = await response.json();

        const currentAccounts = currentState.accounts || [];
        const currentChainId = currentState.chainId || null;
        const currentOrigins = (currentState.connectedOrigins || []).slice().sort();
        const originsJson = JSON.stringify(currentOrigins);

        const accountsJson = JSON.stringify(currentAccounts.slice().sort());
        const previousOriginsJson = lastKnownConnectedOrigins;
        const previousAccounts = lastKnownAccounts;
        const previousChainId = lastKnownChainId;
        const chainIdChanged = previousChainId !== currentChainId;
        const accountsChanged = previousAccounts !== accountsJson;
        const originsChanged = previousOriginsJson !== originsJson;

        lastKnownChainId = currentChainId;
        lastKnownAccounts = accountsJson;
        lastKnownConnectedOrigins = originsJson;

        if (isFirstPoll) {
            isFirstPoll = false;
            return;
        }

        if (chainIdChanged || accountsChanged || originsChanged) {
            chrome.tabs.query({ url: ["http://*/*", "https://*/*"] }, (tabs) => {
                tabs.forEach(tab => {
                    const tabOrigin = new URL(tab.url).origin;
                    const wasConnected = JSON.parse(previousOriginsJson).includes(tabOrigin);
                    const isConnected = currentOrigins.includes(tabOrigin);

                    if (originsChanged) {
                        if (!isConnected && wasConnected) {
                            chrome.tabs.sendMessage(tab.id, { type: 'accountsChanged', payload: [] });
                        } else if (isConnected && !wasConnected) {
                            chrome.tabs.sendMessage(tab.id, { type: 'accountsChanged', payload: currentAccounts });
                        }
                    }

                    if (isConnected && (chainIdChanged || accountsChanged)) {
                        if (chainIdChanged) chrome.tabs.sendMessage(tab.id, { type: 'chainChanged', payload: currentChainId });
                        if (accountsChanged) chrome.tabs.sendMessage(tab.id, { type: 'accountsChanged', payload: currentAccounts });
                    }
                });
            });
        }
    } catch (error) {
        lastKnownAccounts = JSON.stringify([]);
        lastKnownChainId = null;
        chrome.tabs.query({ url: ["http://*/*", "https://*/*"] }, (tabs) => {
            tabs.forEach(tab => {
                chrome.tabs.sendMessage(tab.id, { type: 'accountsChanged', payload: [] });
            });
        });

        console.error("Background: Error during status poll:", error);
        isFirstPoll = true;
    } finally {
        pollInFlight = false;
    }
}


setInterval(pollServerStatus, POLLING_INTERVAL_MS);
setTimeout(pollServerStatus, 500);


chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    if (message.target !== 'background') {
        return false;
    }

    if (message.type === 'fetch') {
        const { url, options } = message.payload;
        const origin = tabOrigin(sender);
        if (url === '/api' && !origin) {
            sendResponse({ success: false, error: 'Missing tab origin' });
            return false;
        }

        authorizedFetch(url, options, url === '/api' ? origin : null)
            .then(response => response.ok ? response.json() : response.text().then(text => { throw new Error(text || `Server returned status ${response.status}`) }))
            .then(jsonData => {
                sendResponse({ success: true, data: jsonData });
            })
            .catch(error => {
                sendResponse({ success: false, error: error.message || 'Failed to fetch' });
            });

        return true;
    }

    return false;
});

console.log('Zeus background service worker started.');
