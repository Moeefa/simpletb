import { HTMLAttributes, useEffect, useState } from "react";

export default function Render({ ...props }: HTMLAttributes<HTMLDivElement>) {
  const [currentTime, setCurrentTime] = useState<string>(
    new Date().toLocaleTimeString("pt-BR", {
      hour: "2-digit",
      minute: "2-digit",
    })
  );

  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentTime(
        new Date().toLocaleTimeString("pt-BR", {
          hour: "2-digit",
          minute: "2-digit",
        })
      );
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  return <div {...props}>{currentTime}</div>;
}
