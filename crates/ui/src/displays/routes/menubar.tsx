import { Window } from "@tauri-apps/api/window";
import { useEffect } from "react";
import ExecuteButton from "../../components/ui/execute-button";
import ActiveWindow from "../../components/widgets/active-window";
import Clock from "../../components/widgets/clock";

export function Menubar() {
  const params = new URLSearchParams(window.location.search);

  async function fetchSize() {
    const window = Window.getCurrent();
    const size = await window.innerSize();
    document.body.style.height = `${size?.height}px`;
  }

  useEffect(() => {
    fetchSize();
  }, []);

  const widgets = {
    left: [
      {
        id: "window",
        label: <ActiveWindow.Label />,
        render: <ActiveWindow.Render />,
      },
      {
        id: "terminal",
        label: <></>,
        render: (
          <ExecuteButton commandline='pwsh.exe -NoExit -NoLogo -Command "cd $env:USERPROFILE"'>
            Terminal
          </ExecuteButton>
        ),
      },
      {
        id: "explorer",
        label: <></>,
        render: (
          <ExecuteButton applicationname="C:\Windows\explorer.exe">
            Explorer
          </ExecuteButton>
        ),
      },
      {
        id: "code",
        label: <></>,
        render: (
          <ExecuteButton applicationname="%USERPROFILE%\AppData\Local\Programs\Microsoft VS Code\Code.exe">
            Code
          </ExecuteButton>
        ),
      },
    ],

    center: [
      {
        id: "date",
        label: <></>,
        render: (
          <p className="first-letter:uppercase">
            {new Date().toLocaleDateString("pt-BR", {
              weekday: "short",
              day: "2-digit",
              month: "short",
            })}
          </p>
        ),
      },
    ],

    right: [
      // {
      //   id: "settings",
      //   label: (
      //     <button
      //       onClick={async () => await invoke("open_settings")}
      //       className="cursor-default hover:bg-neutral-700/30 rounded-sm mr-1 px-1"
      //     >
      //       <ToggleMultipleRegular fontSize={18} />
      //     </button>
      //   ),
      //   render: <></>,
      // },
      {
        id: "clock",
        label: <></>,
        render: <Clock.Render />,
      },
    ],
  };

  return (
    <div
      className="main-container"
      style={{
        backgroundColor:
          params.get("blur") === "true" ? "transparent" : "var(--background)",
      }}
    >
      <div className="widgets left">
        {widgets.left.map((widget) => (
          <div key={widget.id} className="widget">
            {widget.label}

            {widget.render}
          </div>
        ))}
      </div>

      <div className="widgets center">
        {widgets.center.map((widget) => (
          <div key={widget.id} className="widget">
            {widget.label}

            {widget.render}
          </div>
        ))}
      </div>

      <div className="widgets right">
        {widgets.right.map((widget) => (
          <div key={widget.id} className="widget">
            {widget.label}

            {widget.render}
          </div>
        ))}
      </div>
    </div>
  );
}
