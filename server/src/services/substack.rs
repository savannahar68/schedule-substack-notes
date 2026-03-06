use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstackCookies {
    pub connect_sid: Option<String>,
    pub substack_sid: Option<String>,
    pub substack_lli: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubstackUser {
    pub id: Option<i64>,
    pub handle: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PublishedNote {
    pub id: Option<i64>,
    pub url: Option<String>,
}

impl SubstackCookies {
    pub fn to_cookie_header(&self) -> String {
        let mut parts = vec![];
        if let Some(ref sid) = self.connect_sid {
            parts.push(format!("connect.sid={}", sid));
        }
        if let Some(ref sid) = self.substack_sid {
            parts.push(format!("substack.sid={}", sid));
        }
        if let Some(ref lli) = self.substack_lli {
            parts.push(format!("substack.lli={}", lli));
        }
        parts.join("; ")
    }
}

/// Convert plain text (double newline = paragraph break) into Substack's ProseMirror bodyJson format.
pub fn text_to_body_json(text: &str) -> Value {
    let paragraphs: Vec<Value> = text
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| {
            json!({
                "type": "paragraph",
                "content": [{ "type": "text", "text": p }]
            })
        })
        .collect();

    // If no content, emit a single empty paragraph
    let content = if paragraphs.is_empty() {
        vec![json!({ "type": "paragraph" })]
    } else {
        paragraphs
    };

    json!({
        "bodyJson": {
            "type": "doc",
            "attrs": { "schemaVersion": "v1" },
            "content": content
        },
        "tabId": "for-you",
        "replyMinimumRole": "everyone"
    })
}

/// Verify that cookies are valid by hitting Substack's /api/v1/user/self endpoint.
pub async fn verify_cookies(cookies: &SubstackCookies, client: &Client) -> Result<SubstackUser> {
    let response = client
        .get("https://substack.com/api/v1/user/self")
        .header("Cookie", cookies.to_cookie_header())
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .context("Failed to reach Substack")?;

    if !response.status().is_success() {
        let status = response.status();
        anyhow::bail!("Substack returned {status} — cookies may be invalid");
    }

    let user: SubstackUser = response
        .json()
        .await
        .context("Failed to parse Substack user response")?;

    Ok(user)
}

/// Publish a note to Substack using the provided cookies and pre-built bodyJson.
pub async fn publish_note(
    cookies: &SubstackCookies,
    body_json: &Value,
    client: &Client,
) -> Result<PublishedNote> {
    let response = client
        .post("https://substack.com/api/v1/comment/feed")
        .header("Content-Type", "application/json")
        .header("Cookie", cookies.to_cookie_header())
        .header("User-Agent", USER_AGENT)
        .json(body_json)
        .send()
        .await
        .context("Failed to reach Substack")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Substack returned {status}: {body}");
    }

    let note: PublishedNote = response
        .json()
        .await
        .context("Failed to parse Substack note response")?;

    Ok(note)
}
