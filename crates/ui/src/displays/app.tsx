import { HashRouter } from "react-router-dom";
import Routes from "./router";

export function App() {
  return (
    <HashRouter>
      <Routes />
    </HashRouter>
  );
}
