import { HTMLAttributes, useEffect, useState } from "react";

import React from "react";
import { blacklist } from "../../../displays/data-blacklist";
import { listen } from "@tauri-apps/api/event";
import { replaceIcon } from ".";

export default function Label({ ...props }: HTMLAttributes<HTMLDivElement>) {
  const [activeWindow, setActiveWindow] = useState<{
    app: string;
    buffer: number[];
  }>({ app: "Windows Explorer", buffer: [0] });

  async function listenEvent() {
    await listen<{ message: string; buffer: number[] }>(
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
      }
    );
  }

  useEffect(() => {
    listenEvent();
  }, []);

  return (
    <div className="w-[15px]" {...props}>
      <img
        src={replaceIcon(activeWindow)}
        alt={activeWindow.app}
        width="100%"
        className="object-scale-down"
      />
    </div>
  );
}
