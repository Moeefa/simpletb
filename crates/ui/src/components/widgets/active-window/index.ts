import { Buffer } from "buffer";
import Label from "./label";
import Render from "./render";
import { icons } from "../../../displays/data-icons";
import { names } from "../../../displays/data-names";

export const replaceIcon = (window: { app: string; buffer: number[] }) => {
  return (
    icons[window.app] ||
    `data:image/png;base64,${Buffer.from(window?.buffer || []).toString(
      "base64"
    )}`
  );
};

export const replaceName = (name: string) => {
  return names[name] || name;
};

export default { Label, Render };
