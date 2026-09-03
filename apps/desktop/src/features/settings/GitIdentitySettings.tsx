import { useEffect, useState } from "react";
import { getCommitIdentity, setCommitIdentity } from "../../api/settings";

/// Who every world's Git commits are attributed to. Defaults to
/// `mcgit <mcgit@localhost>` (unchanged from before this setting existed)
/// until the player sets their own — useful if they ever push a world's
/// history to a remote that expects a real identity.
export function GitIdentitySettings() {
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getCommitIdentity()
      .then((identity) => {
        setName(identity.name);
        setEmail(identity.email);
      })
      .catch((err) => setError(String(err)));
  }, []);

  async function handleSave(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setStatus(null);
    try {
      await setCommitIdentity(name.trim(), email.trim());
      // Re-fetch rather than trusting what was just typed — a blank field
      // clears back to the default instead of staying blank, so this shows
      // the value that's actually in effect (e.g. "mcgit"), not an empty
      // field that looks like "no identity is set".
      const identity = await getCommitIdentity();
      setName(identity.name);
      setEmail(identity.email);
      setStatus("Saved.");
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <section>
      <h2>Commit identity</h2>
      {error && <p className="banner banner-error">{error}</p>}
      {status && <p className="banner banner-status">{status}</p>}
      <form className="stacked-form" onSubmit={handleSave}>
        <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Name" />
        <input
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          placeholder="Email"
        />
        <button type="submit" className="btn-primary">
          Save
        </button>
      </form>
    </section>
  );
}
