import { useEffect, useMemo, useState } from "react";
import {
  DISCOVERY_FILTERS,
  EMPTY_DISCOVERY_STATUS,
  filterDiscoveryItems,
  formatDiscoveryMeta,
  type DiscoveryFilter,
  type DiscoveryStatus
} from "../features/updates/discovery";
import { listenToDiscoveryStatus } from "../lib/events";
import {
  disableDiscovery,
  enableDiscovery,
  getDiscoveryStatus,
  openDiscoveryItem,
  refreshDiscovery
} from "../lib/invoke";

const DISCOVERY_PREFERENCE = "pebble.discovery.enabled";

export function DiscoveryControl() {
  const [status, setStatus] = useState<DiscoveryStatus>(EMPTY_DISCOVERY_STATUS);
  const [filter, setFilter] = useState<DiscoveryFilter>("all");
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const visibleItems = useMemo(
    () => filterDiscoveryItems(status.items, filter),
    [filter, status.items]
  );

  useEffect(() => {
    let active = true;
    let unlisten: () => void = () => undefined;
    getDiscoveryStatus()
      .then((next) => {
        if (!active) return;
        setStatus(next);
        if (!next.enabled && readPreference()) {
          setPending(true);
          void enableDiscovery()
            .then((enabled) => active && setStatus(enabled))
            .catch(() => active && setMessage("DISCOVERY COULD NOT START"))
            .finally(() => active && setPending(false));
        }
      })
      .catch(() => undefined);
    void listenToDiscoveryStatus((next) => active && setStatus(next)).then(
      (nextUnlisten) => {
        if (active) unlisten = nextUnlisten;
        else nextUnlisten();
      }
    );
    return () => {
      active = false;
      unlisten();
    };
  }, []);

  const run = (action: () => Promise<DiscoveryStatus>) => {
    setPending(true);
    setMessage(null);
    void action()
      .then(setStatus)
      .catch((error: unknown) => setMessage(readError(error)))
      .finally(() => setPending(false));
  };

  const start = () => {
    writePreference(true);
    run(enableDiscovery);
  };

  const stop = () => {
    writePreference(false);
    run(disableDiscovery);
  };

  return (
    <section className="discovery" aria-label="DISCOVER">
      <div className="discovery__heading">
        <span>DISCOVER</span>
        <span>
          {status.enabled
            ? `EVERY ${status.intervalMinutes} MIN`
            : "PUBLIC SOURCES · 0 AI TOKENS"}
        </span>
      </div>
      <div className="discovery__actions">
        {status.enabled ? (
          <>
            <button
              type="button"
              disabled={pending}
              onClick={() => run(refreshDiscovery)}
            >
              REFRESH
            </button>
            <button type="button" disabled={pending} onClick={stop}>
              STOP
            </button>
          </>
        ) : (
          <button type="button" disabled={pending} onClick={start}>
            {pending ? "STARTING" : "START"}
          </button>
        )}
      </div>
      <div
        className="discovery__filters"
        role="tablist"
        aria-label="DISCOVERY FILTER"
      >
        {DISCOVERY_FILTERS.map((option) => (
          <button
            key={option}
            type="button"
            role="tab"
            aria-selected={filter === option}
            className={filter === option ? "is-active" : ""}
            onClick={() => setFilter(option)}
          >
            {option}
          </button>
        ))}
      </div>
      {filter === "x" ? (
        <p className="discovery__notice">
          X TRENDS REQUIRE OFFICIAL API ACCESS. PEBBLE DOES NOT READ BROWSER
          COOKIES OR LOGIN SESSIONS.
        </p>
      ) : null}
      {message ?? status.error ? (
        <p className="discovery__error">{message ?? status.error}</p>
      ) : null}
      {status.warnings.map((warning) => (
        <p className="discovery__warning" key={warning}>{warning}</p>
      ))}
      {filter !== "x" && !status.enabled ? (
        <p className="discovery__notice">
          START ONCE TO CHECK BBC WORLD AND HACKER NEWS. NO COOKIES OR AI CALLS.
        </p>
      ) : null}
      {filter !== "x" &&
      status.enabled &&
      visibleItems.length === 0 &&
      !pending ? (
        <p className="discovery__notice">NO ITEMS AVAILABLE</p>
      ) : null}
      {visibleItems.length > 0 ? (
        <ol className="discovery__items">
          {visibleItems.map((item) => (
            <li key={item.id}>
              <button
                type="button"
                className="discovery__item"
                onClick={() =>
                  void openDiscoveryItem(item.id).catch(() =>
                    setMessage("ITEM COULD NOT BE OPENED")
                  )
                }
              >
                <span>{item.title}</span>
                <small>{formatDiscoveryMeta(item)}</small>
              </button>
            </li>
          ))}
        </ol>
      ) : null}
    </section>
  );
}

function readPreference(): boolean {
  try {
    return window.localStorage.getItem(DISCOVERY_PREFERENCE) === "true";
  } catch {
    return false;
  }
}

function writePreference(enabled: boolean) {
  try {
    window.localStorage.setItem(DISCOVERY_PREFERENCE, String(enabled));
  } catch {
    // Discovery still works for the current app session when storage is unavailable.
  }
}

function readError(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string") return message;
  }
  return "DISCOVERY COULD NOT BE UPDATED";
}
