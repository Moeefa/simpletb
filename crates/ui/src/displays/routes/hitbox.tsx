import { emit, listen } from "@tauri-apps/api/event";

import { useEffect, useState } from "react";

let timeout: NodeJS.Timeout;
export function Hitbox() {
  const [_fullscreen, setFullscreen] = useState(false);

  useEffect(() => {
    listen("app-fullscreen", () => setFullscreen(true));
    listen("app-not-fullscreen", () => setFullscreen(false));

    listen("hover-bar", () => {
      clearTimeout(timeout);
      emit("mouse-in");
    });

    document.body.addEventListener("mouseleave", () => {
      clearTimeout(timeout);
      timeout = setTimeout(() => {
        emit("mouse-out");
      }, 3000);
    });

    document.body.addEventListener("mouseenter", () => {
      clearTimeout(timeout);
      emit("hover-hitbox");
    });
  }, []);

  return <div className="h-full w-full bg-transparent" />;
}
