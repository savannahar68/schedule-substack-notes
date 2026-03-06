'use strict';

const HOSTED_URL = 'https://api.substacknotes.app';
const SELF_HOSTED_URL = 'http://localhost:6894';

async function init() {
  const { backendUrl, authToken, substackHandle } = await chrome.storage.local.get([
    'backendUrl', 'authToken', 'substackHandle',
  ]);

  const url = backendUrl || SELF_HOSTED_URL;
  document.getElementById('backend-url').value = url;
  document.getElementById('status-handle').textContent = substackHandle || '—';
  document.getElementById('status-server').textContent = url;
  document.getElementById('status-cookies').textContent = authToken ? 'Synced ✓' : 'Not connected';

  setMode(url === HOSTED_URL ? 'cloud' : 'self');

  document.getElementById('btn-save').addEventListener('click', handleSave);
  document.getElementById('btn-resync').addEventListener('click', handleResync);
  document.getElementById('btn-check-health').addEventListener('click', handleHealthCheck);
  document.getElementById('btn-mode-self').addEventListener('click', () => setMode('self'));
  document.getElementById('btn-mode-cloud').addEventListener('click', () => setMode('cloud'));
}

function setMode(mode) {
  document.getElementById('btn-mode-self').classList.toggle('active', mode === 'self');
  document.getElementById('btn-mode-cloud').classList.toggle('active', mode === 'cloud');
  document.getElementById('self-hosted-info').classList.toggle('hidden', mode !== 'self');
  document.getElementById('cloud-info').classList.toggle('hidden', mode !== 'cloud');
  document.getElementById('backend-url').value = mode === 'cloud' ? HOSTED_URL : SELF_HOSTED_URL;
}

async function handleSave() {
  const url = document.getElementById('backend-url').value.trim();
  if (!url) return showStatus('URL cannot be empty.', 'error');
  await chrome.storage.local.set({ backendUrl: url });
  document.getElementById('status-server').textContent = url;
  showStatus('Saved!', 'success');
}

async function handleResync() {
  const btn = document.getElementById('btn-resync');
  btn.disabled = true;
  btn.textContent = 'Re-syncing…';

  try {
    const { success, cookies, error } = await chrome.runtime.sendMessage({ type: 'GET_SUBSTACK_COOKIES' });

    if (!success) throw new Error(error || 'Failed to get cookies');
    if (!cookies.connect_sid && !cookies.substack_sid) {
      throw new Error('No Substack session found. Please log in to substack.com first.');
    }

    const userResult = await chrome.runtime.sendMessage({ type: 'GET_SUBSTACK_USER' });
    const handle = userResult.success
      ? (userResult.user.handle || userResult.user.name || null)
      : null;

    const api = await SchedulerAPI.create();
    const response = await api.registerCookies(cookies, handle);
    const finalHandle = handle || response.user.handle;

    await chrome.storage.local.set({ authToken: response.token, substackHandle: finalHandle });

    document.getElementById('status-handle').textContent = finalHandle || '—';
    document.getElementById('status-cookies').textContent = 'Synced ✓';
    showStatus(finalHandle ? `Connected as ${finalHandle}` : 'Connected', 'success');
  } catch (err) {
    showStatus(err.message, 'error');
  } finally {
    btn.disabled = false;
    btn.textContent = 'Re-sync Cookies';
  }
}

async function handleHealthCheck() {
  const { authToken } = await chrome.storage.local.get(['authToken']);
  const cookiesEl = document.getElementById('status-cookies');

  if (!authToken) {
    cookiesEl.textContent = 'Not connected';
    return;
  }

  const btn = document.getElementById('btn-check-health');
  btn.disabled = true;
  btn.textContent = 'Checking…';

  const sid = await chrome.cookies.get({ url: 'https://substack.com', name: 'substack.sid' })
    || await chrome.cookies.get({ url: 'https://substack.com', name: 'connect.sid' });

  cookiesEl.textContent = sid
    ? `Valid (checked ${new Date().toLocaleTimeString()})`
    : 'Logged out of Substack — please Re-sync';
  cookiesEl.style.color = sid ? 'var(--success)' : 'var(--danger)';

  btn.disabled = false;
  btn.textContent = 'Check Now';
}

function showStatus(msg, type) {
  const el = document.getElementById('save-status');
  el.textContent = msg;
  el.className = 'status-text ' + type;
  el.classList.remove('hidden');
  setTimeout(() => el.classList.add('hidden'), 4000);
}

document.addEventListener('DOMContentLoaded', init);
