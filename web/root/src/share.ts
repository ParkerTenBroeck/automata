import { bus } from "./bus.ts";

export class Share {
  private static readonly btn: HTMLButtonElement = document.getElementById(
    "shareBtn",
  )! as HTMLButtonElement;
  private static readonly toast: HTMLElement = document.getElementById(
    "shareToast",
  )!;

  private static docText: string;
  private static shareText: string;

  static {
    bus.on("editor/change", ({ text }) => Share.docText = text);

    Share.btn.onclick = async (_) => {
      const link = `${globalThis.window.location.href}?share=${
        encodeURIComponent(btoa(Share.docText))
      }`;
      await navigator.clipboard.writeText(link);

      Share.toast.classList.remove("show");
      void Share.toast.offsetWidth;
      Share.toast.classList.add("show");
    };

    try {
      const url = new URL(globalThis.window.location.href);
      let text: string | null = url.searchParams.get("share");
      if (text !== null) {
        text = atob(text);
        url.searchParams.delete("share");
        globalThis.window.history.replaceState(
          {},
          document.title,
          url.pathname + url.search + url.hash,
        );
        Share.shareText = text;
      }
    } catch (e) {
      console.log(e);
    }
  }

  public static sharedText(): string | null {
    return Share.shareText;
  }
}
