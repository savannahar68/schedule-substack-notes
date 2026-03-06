/**
 * Backend API client for the Substack Notes Scheduler.
 * Reads backendUrl from chrome.storage.local (default: http://localhost:6894).
 */
class SchedulerAPI {
  constructor(baseUrl) {
    this.baseUrl = baseUrl;
  }

  static async create() {
    const result = await chrome.storage.local.get(['backendUrl']);
    const baseUrl = result.backendUrl || 'http://localhost:6894';
    return new SchedulerAPI(baseUrl);
  }

  async _getToken() {
    const result = await chrome.storage.local.get(['authToken']);
    return result.authToken || null;
  }

  async _request(method, path, body = null, requiresAuth = true) {
    const headers = { 'Content-Type': 'application/json' };

    if (requiresAuth) {
      const token = await this._getToken();
      if (!token) throw new Error('Not authenticated');
      headers['Authorization'] = `Bearer ${token}`;
    }

    const options = { method, headers };
    if (body) options.body = JSON.stringify(body);

    const response = await fetch(`${this.baseUrl}${path}`, options);
    const data = await response.json().catch(() => ({}));

    if (!response.ok) {
      throw new Error(data.error || `Request failed with status ${response.status}`);
    }

    return data;
  }

  /**
   * Register Substack cookies with the backend.
   * Returns { token, user: { handle } }
   */
  async registerCookies(cookies, handle = null) {
    return this._request('POST', '/api/auth/cookies', { cookies, handle }, false);
  }

  /**
   * Check if stored cookies are still valid.
   * Returns { valid, handle, last_checked }
   */
  async checkHealth() {
    return this._request('GET', '/api/auth/health');
  }

  /**
   * Schedule a note.
   * @param {string} text - Note content (double newline = paragraph break)
   * @param {string} scheduledAt - ISO 8601 datetime string (UTC)
   */
  async scheduleNote(text, scheduledAt) {
    return this._request('POST', '/api/notes/schedule', { text, scheduled_at: scheduledAt });
  }

  /**
   * Get the user's scheduled note queue.
   * Returns { notes: [...] }
   */
  async getQueue() {
    return this._request('GET', '/api/notes/queue');
  }

  /**
   * Delete a scheduled note (only pending notes can be deleted).
   */
  async deleteNote(noteId) {
    return this._request('DELETE', `/api/notes/${noteId}`);
  }

  /**
   * Update a scheduled note (only pending notes can be updated).
   * @param {string} noteId
   * @param {{ text?: string, scheduled_at?: string }} updates
   */
  async updateNote(noteId, updates) {
    return this._request('PUT', `/api/notes/${noteId}`, updates);
  }

  /**
   * Get notes that are due to be published now.
   * Returns { notes: [{ id, body_json }] }
   */
  async getDueNotes() {
    return this._request('GET', '/api/notes/due');
  }

  /**
   * Report the result of a publish attempt back to the backend.
   * @param {string} noteId
   * @param {{ success: boolean, substack_id?: string, substack_url?: string, error?: string }} result
   */
  async reportResult(noteId, result) {
    return this._request('POST', `/api/notes/${noteId}/result`, result);
  }
}
