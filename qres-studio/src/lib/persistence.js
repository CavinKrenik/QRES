/**
 * QRES Browser Persistence Module
 * Uses IndexedDB to store WorldState for offline/reload persistence.
 * Integrates with qres_wasm to hydrate state.
 */

const DB_NAME = 'qres_world_state';
const DB_VERSION = 1;
const STORE_NAME = 'states';

/**
 * Open or create the IndexedDB database.
 * @returns {Promise<IDBDatabase>}
 */
export function openDatabase() {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open(DB_NAME, DB_VERSION);

        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result);

        request.onupgradeneeded = (event) => {
            const target = /** @type {IDBOpenDBRequest} */ (event.target);
            const db = target.result;
            if (!db.objectStoreNames.contains(STORE_NAME)) {
                db.createObjectStore(STORE_NAME, { keyPath: 'version' });
            }
        };
    });
}

/**
 * Save a WorldState to IndexedDB.
 * @param {string} version - Version identifier.
 * @param {ArrayBuffer|Uint8Array} data - Serialized state data.
 * @returns {Promise<void>}
 */
export async function saveState(version, data) {
    const db = await openDatabase();
    return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, 'readwrite');
        const store = tx.objectStore(STORE_NAME);

        const record = {
            version,
            data: data instanceof ArrayBuffer ? data : data.buffer,
            timestamp: Date.now()
        };

        const request = store.put(record);
        request.onsuccess = () => resolve();
        request.onerror = () => reject(request.error);
    });
}

/**
 * Load a WorldState from IndexedDB.
 * @param {string} version - Version identifier, or 'latest' for most recent.
 * @returns {Promise<{version: string, data: ArrayBuffer}|null>}
 */
export async function loadState(version) {
    const db = await openDatabase();
    return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, 'readonly');
        const store = tx.objectStore(STORE_NAME);

        if (version === 'latest') {
            // Get all and find most recent
            const request = store.getAll();
            request.onsuccess = () => {
                const records = request.result;
                if (records.length === 0) {
                    resolve(null);
                } else {
                    const latest = records.reduce((a, b) =>
                        a.timestamp > b.timestamp ? a : b
                    );
                    resolve(latest);
                }
            };
            request.onerror = () => reject(request.error);
        } else {
            const request = store.get(version);
            request.onsuccess = () => resolve(request.result || null);
            request.onerror = () => reject(request.error);
        }
    });
}

/**
 * List all saved state versions.
 * @returns {Promise<string[]>}
 */
export async function listVersions() {
    const db = await openDatabase();
    return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, 'readonly');
        const store = tx.objectStore(STORE_NAME);
        const request = store.getAllKeys();

        request.onsuccess = () => resolve(/** @type {string[]} */(request.result));
        request.onerror = () => reject(request.error);
    });
}

/**
 * Delete a state by version.
 * @param {string} version 
 * @returns {Promise<void>}
 */
export async function deleteState(version) {
    const db = await openDatabase();
    return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, 'readwrite');
        const store = tx.objectStore(STORE_NAME);
        const request = store.delete(version);

        request.onsuccess = () => resolve();
        request.onerror = () => reject(request.error);
    });
}
