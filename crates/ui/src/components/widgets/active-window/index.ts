import { icons } from "@/displays/data/data-icons";
import { names } from "@/displays/data/data-names";
import { Buffer } from "buffer";
import Label from "./label";
import Render from "./render";

export const replaceIcon = (window: { app: string; buffer: number[] }) => {
  return (
    icons[window.app] ||
    `data:image/png;base64,${Buffer.from(window?.buffer || []).toString(
      "base64",
    )}`
  );
};

export const replaceName = (name: string) => {
  return names[name as keyof typeof names] || name;
};

export default { Label, Render };
