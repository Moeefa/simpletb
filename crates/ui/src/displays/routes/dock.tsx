import { emit, listen } from "@tauri-apps/api/event";
import { Reorder, motion } from "framer-motion";
import { useEffect, useRef, useState } from "react";

import { invoke } from "@tauri-apps/api/core";
import { Buffer } from "buffer";

type App = {
  hwnd: number;
  buffer: number[];
};

let timeout: NodeJS.Timeout;
export function Dock() {
  const [active, setActive] = useState<number>(-1);
  const [apps, setApps] = useState<App[]>([]);
  const isJustReordered = useRef(false);

  useEffect(() => {
    listen<App[]>("set-apps", (event) => setApps(event.payload));

    listen<{ message: string; buffer: number[]; hwnd: number }>(
      "active-window",
      (event) => {
        console.log(event.payload);
        typeof event.payload === "number"
          ? setActive(event.payload)
          : setActive(event.payload.hwnd);
      },
    );

    listen("hover-hitbox", () => {
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
      emit("hover-bar");
    });

    emit("ready");
  }, []);

  const handleChangeWindow = async (app: App) => {
    active === app.hwnd ? setActive(-1) : setActive(app.hwnd);
    await invoke("show_window", {
      hwnd: app.hwnd,
    });
  };

  const onReorder = (newOrder: App[]) => {
    isJustReordered.current = true;

    setTimeout(() => {
      isJustReordered.current = false;
    }, 150);

    setApps(newOrder);
  };

  return (
    <Reorder.Group
      axis="x"
      values={apps}
      onReorder={onReorder}
      className="flex bg-white/80 dark:bg-transparent grow aspect-square gap-1 justify-center flex-nowrap items-center w-full px-1 h-full pb-[0.125rem]"
      as="ul"
    >
      {apps.map((app) => {
        return (
          <Reorder.Item
            data-active={active === app.hwnd}
            key={app.hwnd}
            value={app}
            id={app.hwnd.toString()}
            className="group dark:backdrop-blur select-none size-10 relative flex items-center justify-center aspect-square bg-white/60 dark:bg-white/5 rounded-md hover:bg-white/90 dark:hover:bg-white/15 border border-black/15 dark:border-white/[0.025]"
            onPointerUp={() =>
              !isJustReordered.current && handleChangeWindow(app)
            }
            // onContextMenu={async (e) => {
            //   e.preventDefault();
            //   await invoke("open_context", {
            //     x: e.clientX,
            //     y: e.clientY,
            //   });
            // }}
          >
            {app.buffer.length === 0 ? (
              <motion.h1 className="text-lg group-data-[active=true]:animate-[bounce-up_0.55s_ease-in-out_1] group-data-[active=false]:animate-[bounce-down_0.55s_ease-in-out_1]">
                ❔
              </motion.h1>
            ) : (
              <motion.img
                draggable="false"
                className="object-scale-down select-none aspect-square h-[1.45rem] group-data-[active=true]:animate-[bounce-up_0.55s_ease-in-out_1] group-data-[active=false]:animate-[bounce-down_0.55s_ease-in-out_1]"
                src={`data:image/png;base64,${Buffer.from(
                  app.buffer || [],
                ).toString("base64")}`}
              />
            )}
            <motion.div className="absolute duration-300 ease-in-out transition-all group-data-[active=true]:w-4 group-data-[active=false]:w-1.5 h-[0.18rem] group-data-[active=true]:bg-blue-400 group-data-[active=false]:bg-neutral-400 bottom-0 rounded-full" />
          </Reorder.Item>
        );
      })}
    </Reorder.Group>
  );
}
