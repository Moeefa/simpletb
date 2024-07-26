import { HTMLAttributes, useEffect, useState } from "react";

import React from "react";
import { blacklist } from "../../../displays/data-blacklist";
import { listen } from "@tauri-apps/api/event";
import { replaceName } from ".";

export default function Render({ ...props }: HTMLAttributes<HTMLDivElement>) {
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
    <div className="truncate flex-1 font-semibold mr-2" {...props}>
      {replaceName(activeWindow.app)}
    </div>
  );
}
