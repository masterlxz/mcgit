import { HashRouter, Link, Route, Routes } from "react-router-dom";
import { AdvancedModeProvider, useAdvancedMode } from "./context/AdvancedModeContext";
import { InstanceDetailScreen } from "./features/instance/InstanceDetailScreen";
import { InstanceManagerScreen } from "./features/instance/InstanceManagerScreen";
import { JavaManagerScreen } from "./features/java/JavaManagerScreen";
import "./App.css";

function AdvancedModeToggle() {
  const { advancedMode, setAdvancedMode } = useAdvancedMode();
  return (
    <label>
      <input
        type="checkbox"
        checked={advancedMode}
        onChange={(e) => setAdvancedMode(e.target.checked)}
      />{" "}
      Advanced mode
    </label>
  );
}

function App() {
  return (
    <AdvancedModeProvider>
      <HashRouter>
        <main className="container">
          <nav>
            <Link to="/">Instances</Link> | <Link to="/java">Java</Link> |{" "}
            <AdvancedModeToggle />
          </nav>
          <Routes>
            <Route path="/" element={<InstanceManagerScreen />} />
            <Route path="/instances/:id" element={<InstanceDetailScreen />} />
            <Route path="/java" element={<JavaManagerScreen />} />
          </Routes>
        </main>
      </HashRouter>
    </AdvancedModeProvider>
  );
}

export default App;
