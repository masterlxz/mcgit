import { HashRouter, Link, Route, Routes } from "react-router-dom";
import { AdvancedModeProvider, useAdvancedMode } from "./context/AdvancedModeContext";
import { InstanceDetailScreen } from "./features/instance/InstanceDetailScreen";
import { InstanceManagerScreen } from "./features/instance/InstanceManagerScreen";
import { SettingsScreen } from "./features/settings/SettingsScreen";
import "./App.css";

function AdvancedModeToggle() {
  const { advancedMode, setAdvancedMode } = useAdvancedMode();
  return (
    <label className="advanced-mode-toggle">
      <input
        type="checkbox"
        checked={advancedMode}
        onChange={(e) => setAdvancedMode(e.target.checked)}
      />
      Advanced mode
    </label>
  );
}

function App() {
  return (
    <AdvancedModeProvider>
      <HashRouter>
        <main className="container">
          <nav className="app-nav">
            <Link to="/" className="brand">
              mcgit
            </Link>
            <div className="nav-links">
              <Link to="/">Instances</Link>
              <Link to="/settings" className="settings-link" aria-label="Settings" title="Settings">
                ⚙
              </Link>
              <AdvancedModeToggle />
            </div>
          </nav>
          <Routes>
            <Route path="/" element={<InstanceManagerScreen />} />
            <Route path="/instances/:id" element={<InstanceDetailScreen />} />
            <Route path="/settings" element={<SettingsScreen />} />
          </Routes>
        </main>
      </HashRouter>
    </AdvancedModeProvider>
  );
}

export default App;
