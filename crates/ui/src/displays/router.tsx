import { Route, Routes } from "react-router-dom";
import { Rounded } from "../components/ui/round-menubar";
import { Dock } from "./routes/dock";
import { Hitbox } from "./routes/hitbox";
import { Menubar } from "./routes/menubar";
import { Settings } from "./routes/settings";

export default function RoutesElement() {
  return (
    <Routes>
      <Route path="/menubar" element={<Menubar />} />
      <Route path="/settings" element={<Settings />} />
      <Route path="/dock" element={<Dock />} />
      <Route path="/hitbox" element={<Hitbox />} />
      <Route path="/rounded" element={<Rounded />} />
    </Routes>
  );
}
