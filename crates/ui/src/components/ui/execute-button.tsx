import { HTMLAttributes } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function ExecuteButton({
  applicationname = "",
  commandline = "",
  children,
  ...props
}: HTMLAttributes<HTMLButtonElement> & {
  applicationname?: string;
  commandline?: string;
}) {
  return (
    <button
      onClick={async () =>
        await invoke("execute", {
          applicationname,
          commandline,
        })
      }
      className="cursor-default hover:bg-white/90 dark:hover:bg-black-700/60 px-1.5 py-0.5 rounded-md"
      {...props}
    >
      {children}
    </button>
  );
}
