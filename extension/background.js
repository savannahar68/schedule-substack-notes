importScripts('lib/cookies.js', 'lib/api.js');

// Alarms are registered here (not just onInstalled) because the service worker
// can be killed and restarted by Chrome at any time.
async function ensureAlarms() {
  const existing = new Set((await chrome.alarms.getAll()).map((a) => a.name));
  if (!existing.has('publish-due-notes')) {
    chrome.alarms.create('publish-due-notes', { delayInMinutes: 1, periodInMinutes: 1 });
  }
  if (!existing.has('cookie-health-check')) {
    chrome.alarms.create('cookie-health-check', { delayInMinutes: 60, periodInMinutes: 360 });
  }
}
ensureAlarms();

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === 'publish-due-notes') publishDueNotes();
  if (alarm.name === 'cookie-health-check') updateHealthBadge();
});

async function updateHealthBadge() {
  const { authToken } = await chrome.storage.local.get(['authToken']);
  if (!authToken) return;

  const sid = await chrome.cookies.get({ url: 'https://substack.com', name: 'substack.sid' })
    || await chrome.cookies.get({ url: 'https://substack.com', name: 'connect.sid' });

  if (sid) {
    chrome.action.setBadgeText({ text: '' });
  } else {
    chrome.action.setBadgeText({ text: '!' });
    chrome.action.setBadgeBackgroundColor({ color: '#ef4444' });
  }
}

async function getSubstackUser() {
  const lli = await chrome.cookies.get({ url: 'https://substack.com', name: 'substack.lli' });
  if (!lli) throw new Error('Not logged in to Substack');

  const { userId } = JSON.parse(atob(lli.value.split('.')[1]));
  const r = await fetch(`https://substack.com/api/v1/user/${userId}/public_profile`, {
    credentials: 'include',
  });
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  const data = await r.json();
  return data.user_profile || data;
}

async function publishDueNotes() {
  const { authToken } = await chrome.storage.local.get(['authToken']);
  if (!authToken) return;

  const api = await SchedulerAPI.create();
  let notes;
  try {
    ({ notes } = await api.getDueNotes());
  } catch (err) {
    console.error('Failed to fetch due notes:', err.message);
    return;
  }

  for (const note of notes) {
    try {
      const response = await fetch('https://substack.com/api/v1/comment/feed', {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: note.body_json,
      });

      if (!response.ok) {
        const text = await response.text();
        throw new Error(`Substack ${response.status}: ${text.slice(0, 200)}`);
      }

      const result = await response.json();
      api.reportResult(note.id, {
        success: true,
        substack_id: result.id != null ? String(result.id) : undefined,
        substack_url: result.url || undefined,
      }).catch(() => {});
    } catch (err) {
      console.error(`Failed to publish note ${note.id}:`, err.message);
      api.reportResult(note.id, { success: false, error: err.message }).catch(() => {});
    }
  }
}

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message.type === 'GET_SUBSTACK_COOKIES') {
    getSubstackCookies()
      .then((cookies) => sendResponse({ success: true, cookies }))
      .catch((err) => sendResponse({ success: false, error: err.message }));
    return true;
  }

  if (message.type === 'GET_SUBSTACK_USER') {
    getSubstackUser()
      .then((user) => sendResponse({ success: true, user }))
      .catch((err) => sendResponse({ success: false, error: err.message }));
    return true;
  }
});
