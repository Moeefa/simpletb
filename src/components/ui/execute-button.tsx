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
      className="cursor-default hover:bg-neutral-700/30 px-1.5 rounded-sm"
      {...props}
    >
      {children}
    </button>
  );
}
