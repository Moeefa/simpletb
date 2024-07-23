import { emit, listen } from "@tauri-apps/api/event";

import { useEffect } from "react";

let timeout: NodeJS.Timeout;
function App() {
  useEffect(() => {
    listen("hover-bar", () => {
      console.log("hover-bar");
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

export default App;
