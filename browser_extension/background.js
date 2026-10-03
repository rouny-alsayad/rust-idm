chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "download-with-rust-idm",
    title: "Download with Rust IDM",
    contexts: ["link", "video", "audio", "image"],
  });
});

chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId !== "download-with-rust-idm") return;

  const url = info.linkUrl || info.srcUrl;
  if (url) {
    sendToIDM(url, null, tab?.url || "");
  }
});

chrome.downloads.onCreated.addListener((downloadItem) => {
  if (downloadItem.state !== "in_progress") return;

  const url = downloadItem.finalUrl || downloadItem.url;
  sendToIDM(url, downloadItem.id, downloadItem.referrer || "");
});

function getCookieHeader(url) {
  return new Promise((resolve) => {
    chrome.cookies.getAll({ url }, (cookies) => {
      if (chrome.runtime.lastError || !Array.isArray(cookies)) {
        resolve("");
        return;
      }

      resolve(
        cookies
          .map((cookie) => `${cookie.name}=${cookie.value}`)
          .join("; ")
      );
    });
  });
}

async function sendToIDM(url, downloadIdToCancel, referer) {
  try {
    const cookies = await getCookieHeader(url);

    const response = await fetch("http://127.0.0.1:8765/download", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Rust-IDM-Auth": "secure-local-token",
      },
      body: JSON.stringify({
        url,
        user_agent: navigator.userAgent || "",
        cookies,
        referer: referer || "",
      }),
    });

    if (!response.ok) {
      console.warn("Rust IDM rejected the download:", response.status);
      return;
    }

    if (downloadIdToCancel !== null && downloadIdToCancel !== undefined) {
      try {
        await chrome.downloads.cancel(downloadIdToCancel);
        await chrome.downloads.erase({ id: downloadIdToCancel });
      } catch (error) {
        console.warn("Could not cancel browser download:", error);
      }
    }
  } catch (error) {
    console.warn("Rust IDM is not running or unreachable.", error);
  }
}
