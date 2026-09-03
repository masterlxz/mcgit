import { GitIdentitySettings } from "./GitIdentitySettings";
import { JavaManagerScreen } from "../java/JavaManagerScreen";

export function SettingsScreen() {
  return (
    <section>
      <h1>Settings</h1>
      <GitIdentitySettings />
      <hr className="section-divider" />
      <JavaManagerScreen />
    </section>
  );
}
