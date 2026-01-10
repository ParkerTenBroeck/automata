import { getText } from "./editor.ts";

const btn = document.getElementById("shareBtn")!;
const toast = document.getElementById("shareToast")!;

function generateShareLink() {
  return `${globalThis.window.location.href}?share=${encodeURIComponent(btoa(getText()))}`;
}

async function copy(text: string) {
  await navigator.clipboard.writeText(text);
}

btn.addEventListener("click", async () => {
  await copy(generateShareLink());

  toast.classList.remove("show");
  void toast.offsetWidth; 
  toast.classList.add("show");
});


export function sharedText(): string|null {
  const url = new URL(globalThis.window.location.href);
  let text: string | null = url.searchParams.get("share");
  if (text !== null) {
    text = atob(text);
    url.searchParams.delete("share");
    globalThis.window.history.replaceState(
      {},
      document.title,
      url.pathname + url.search + url.hash
    );
  }
  return text;
}