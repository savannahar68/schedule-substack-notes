'use strict';

let api = null;

function showView(id) {
  document.querySelectorAll('.view').forEach((v) => v.classList.add('hidden'));
  document.getElementById(id).classList.remove('hidden');
}

function showError(id, msg) {
  const el = document.getElementById(id);
  el.textContent = msg;
  el.classList.remove('hidden');
}

function hideError(id) {
  document.getElementById(id).classList.add('hidden');
}

function showSuccess(id, msg) {
  const el = document.getElementById(id);
  el.textContent = msg;
  el.classList.remove('hidden');
  setTimeout(() => el.classList.add('hidden'), 3000);
}

async function init() {
  showView('view-loading');
  api = await SchedulerAPI.create();

  const { authToken, substackHandle } = await chrome.storage.local.get(['authToken', 'substackHandle']);

  if (!authToken) {
    showView('view-disconnected');
    document.getElementById('btn-connect').addEventListener('click', handleConnect);
    return;
  }

  showView('view-connected');
  setupConnectedView(substackHandle);
  await loadQueue();
}

async function handleConnect() {
  const btn = document.getElementById('btn-connect');
  btn.disabled = true;
  btn.textContent = 'Connecting…';
  hideError('connect-error');

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

    const response = await api.registerCookies(cookies, handle);

    await chrome.storage.local.set({
      authToken: response.token,
      substackHandle: handle || response.user.handle,
    });

    await init();
  } catch (err) {
    btn.disabled = false;
    btn.textContent = 'Connect to Substack';
    showError('connect-error', err.message);
  }
}

function setupConnectedView(handle) {
  document.getElementById('user-handle').textContent = handle || 'Connected';

  const next = new Date();
  next.setHours(next.getHours() + 1, 0, 0, 0);
  document.getElementById('schedule-time').value = toDatetimeLocal(next);

  const textarea = document.getElementById('note-text');
  const counter = document.getElementById('char-count');
  textarea.addEventListener('input', () => {
    const len = textarea.value.length;
    counter.textContent = len;
    counter.className = 'char-count' + (len > 500 ? ' warn' : '') + (len > 2000 ? ' over' : '');
  });

  document.getElementById('btn-schedule').addEventListener('click', handleSchedule);
  document.getElementById('btn-disconnect').addEventListener('click', handleDisconnect);
  document.getElementById('btn-refresh').addEventListener('click', loadQueue);
}

async function handleSchedule() {
  const text = document.getElementById('note-text').value.trim();
  const scheduledAt = document.getElementById('schedule-time').value;

  hideError('schedule-error');

  if (!text) return showError('schedule-error', 'Note text is required.');
  if (!scheduledAt) return showError('schedule-error', 'Please pick a date and time.');

  const btn = document.getElementById('btn-schedule');
  btn.disabled = true;
  btn.textContent = 'Scheduling…';

  try {
    await api.scheduleNote(text, new Date(scheduledAt).toISOString());
    document.getElementById('note-text').value = '';
    document.getElementById('char-count').textContent = '0';

    const next = new Date();
    next.setHours(next.getHours() + 1, 0, 0, 0);
    document.getElementById('schedule-time').value = toDatetimeLocal(next);

    showSuccess('schedule-success', 'Note scheduled!');
    await loadQueue();
  } catch (err) {
    showError('schedule-error', err.message);
  } finally {
    btn.disabled = false;
    btn.textContent = 'Schedule';
  }
}

async function handleDisconnect() {
  await chrome.storage.local.remove(['authToken', 'substackHandle']);
  chrome.action.setBadgeText({ text: '' });
  await init();
}

async function loadQueue() {
  const btn = document.getElementById('btn-refresh');
  btn.textContent = '…';

  try {
    const { notes } = await api.getQueue();
    renderQueue(notes);
  } catch (err) {
    if (err.message === 'Invalid token') {
      await chrome.storage.local.remove(['authToken', 'substackHandle']);
      await init();
    }
  } finally {
    btn.textContent = '↻';
  }
}

function renderQueue(notes) {
  const list = document.getElementById('queue-list');

  if (!notes || notes.length === 0) {
    list.innerHTML = '<div class="queue-empty">No scheduled notes</div>';
    return;
  }

  list.innerHTML = '';
  notes.forEach((note) => list.appendChild(createQueueItem(note)));
}

function createQueueItem(note) {
  const item = document.createElement('div');
  item.className = 'queue-item' + (note.status === 'failed' ? ' failed' : '');

  const preview = note.text.replace(/\n+/g, ' ').slice(0, 60) + (note.text.length > 60 ? '…' : '');

  item.innerHTML = `
    <div class="queue-item-body">
      <div class="queue-item-time">${formatDateTime(note.scheduled_at)}</div>
      <div class="queue-item-text">${escapeHtml(preview)}</div>
      ${note.error ? `<div class="queue-item-error">Error: ${escapeHtml(note.error)}</div>` : ''}
      ${note.substack_url
        ? `<div class="queue-item-status published"><a href="${note.substack_url}" target="_blank">View note</a></div>`
        : `<div class="queue-item-status ${note.status}">${capitalize(note.status)}</div>`}
    </div>
    ${note.status === 'pending' ? `<button class="queue-item-delete" data-id="${note.id}" title="Cancel">✕</button>` : ''}
  `;

  item.querySelector('.queue-item-delete')?.addEventListener('click', async () => {
    try {
      await api.deleteNote(note.id);
      await loadQueue();
    } catch (err) {
      console.error('Delete failed:', err);
    }
  });

  return item;
}

function toDatetimeLocal(date) {
  const pad = (n) => String(n).padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

function formatDateTime(iso) {
  return new Date(iso).toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}

function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function capitalize(str) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

document.addEventListener('DOMContentLoaded', init);
