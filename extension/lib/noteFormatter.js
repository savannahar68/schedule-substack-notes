/**
 * Convert plain text into Substack's ProseMirror bodyJson format.
 * Double newlines = paragraph breaks.
 * Mirrors the Rust text_to_body_json() function in server/src/services/substack.rs
 */
function textToBodyJson(text) {
  const paragraphs = text
    .split('\n\n')
    .map((p) => p.trim())
    .filter((p) => p.length > 0)
    .map((p) => ({
      type: 'paragraph',
      content: [{ type: 'text', text: p }],
    }));

  const content = paragraphs.length > 0 ? paragraphs : [{ type: 'paragraph' }];

  return {
    bodyJson: {
      type: 'doc',
      attrs: { schemaVersion: 'v1' },
      content,
    },
    tabId: 'for-you',
    replyMinimumRole: 'everyone',
  };
}
