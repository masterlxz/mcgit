import { HashRouter, Link, Route, Routes } from "react-router-dom";
import { InstanceManagerScreen } from "./features/instance/InstanceManagerScreen";
import { JavaManagerScreen } from "./features/java/JavaManagerScreen";
import "./App.css";

function App() {
  return (
    <HashRouter>
      <main className="container">
        <nav>
          <Link to="/">Instances</Link> | <Link to="/java">Java</Link>
        </nav>
        <Routes>
          <Route path="/" element={<InstanceManagerScreen />} />
          <Route path="/java" element={<JavaManagerScreen />} />
        </Routes>
      </main>
    </HashRouter>
  );
}

export default App;
