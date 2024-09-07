import { HTMLAttributes, useEffect, useState } from "react";

import { blacklist } from "@/displays/data/data-blacklist";
import { listen } from "@tauri-apps/api/event";
import { replaceIcon } from ".";

export default function Label({ ...props }: HTMLAttributes<HTMLDivElement>) {
  const [activeWindow, setActiveWindow] = useState<{
    app: string;
    buffer: number[];
  }>({ app: "Windows Explorer", buffer: [0] });

  async function listenEvent() {
    await listen<{ message: string; buffer: number[]; hwnd: number }>(
      "active-window",
      (event) => {
        if (
          blacklist.includes(event.payload.message) ||
          event.payload.message === undefined
        )
          return;

        setActiveWindow({
          app: event.payload.message,
          buffer: event.payload.buffer,
        });
      },
    );
  }

  useEffect(() => {
    listenEvent();
  }, []);

  return (
    <div className="size-[1.15rem] aspect-square" {...props}>
      <img
        src={replaceIcon(activeWindow)}
        alt={activeWindow.app}
        className="object-scale-down size-[1.15rem]"
      />
    </div>
  );
}
