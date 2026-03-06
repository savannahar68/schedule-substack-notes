async function getSubstackCookies() {
  const names = ['connect.sid', 'substack.sid', 'substack.lli'];
  const cookies = {};

  for (const name of names) {
    const matches = await chrome.cookies.getAll({ domain: 'substack.com', name });
    if (matches.length > 0) {
      cookies[name.replace(/\./g, '_')] = matches[matches.length - 1].value;
    }
  }

  return cookies;
}
