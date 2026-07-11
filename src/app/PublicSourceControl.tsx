import { useEffect, useMemo, useState, type FormEvent } from "react";
import {
  normalizedPublicSourceUrl,
  type PublicSourceStatus
} from "../features/updates/publicSource";
import { listenToPublicSourceStatus } from "../lib/events";
import {
  followPublicSource,
  getPublicSourceStatus,
  unfollowPublicSource
} from "../lib/invoke";
import { errorMessage } from "./usePebbleSession";

export function PublicSourceControl() {
  const [status, setStatus] = useState<PublicSourceStatus | null>(null);
  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const normalizedUrl = useMemo(() => normalizedPublicSourceUrl(url), [url]);

  useEffect(() => {
    let active = true;
    let unlisten: () => void = () => undefined;
    getPublicSourceStatus()
      .then((next) => {
        if (!active) return;
        setStatus(next);
        if (next.url) setUrl(next.url);
      })
      .catch(() => undefined);
    void listenToPublicSourceStatus((next) => {
      if (!active) return;
      setStatus(next);
      if (next.url) setUrl(next.url);
    }).then((nextUnlisten) => {
      if (active) unlisten = nextUnlisten;
      else nextUnlisten();
    });
    return () => {
      active = false;
      unlisten();
    };
  }, []);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    try {
      setBusy(true);
      setError(null);
      if (status?.enabled) {
        setStatus(await unfollowPublicSource());
      } else if (normalizedUrl) {
        setStatus(await followPublicSource(normalizedUrl));
      } else {
        setError("ENTER A PUBLIC HTTPS RSS, ATOM, OR WEB URL.");
      }
    } catch (reason) {
      setError(errorMessage(reason, "PUBLIC SOURCE COULD NOT BE UPDATED."));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="public-source">
      <form onSubmit={(event) => void submit(event)}>
        <input
          type="url"
          aria-label="PUBLIC SOURCE URL"
          value={url}
          placeholder="PUBLIC HTTPS RSS OR WEB URL"
          disabled={busy || status?.enabled}
          onChange={(event) => setUrl(event.currentTarget.value)}
        />
        <button
          type="submit"
          className={status?.enabled ? "secondary-action is-active" : "secondary-action"}
          disabled={busy || (!status?.enabled && normalizedUrl === null)}
        >
          {status?.enabled ? "STOP" : busy ? "CHECKING" : "FOLLOW"}
        </button>
      </form>
      <span>
        {status?.title
          ? `${status.intervalMinutes} MIN · ${status.title}`
          : "PUBLIC SOURCE · 15 MIN"}
      </span>
      {error ?? status?.error ? (
        <p role="alert">{error ?? status?.error}</p>
      ) : null}
    </div>
  );
}
