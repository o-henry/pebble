import { useEffect, useState } from "react";
import downIcon from "../assets/icons/down.svg";
import {
  formatUpdateTime,
  mergeUpdateEntry,
  type UpdateFeedSnapshot
} from "../features/updates/updateFeed";
import { listenToUpdateFeed } from "../lib/events";
import { getUpdateFeed } from "../lib/invoke";
import { isTauriRuntime } from "../lib/runtime";
import { PublicSourceControl } from "./PublicSourceControl";
import { DiscoveryControl } from "./DiscoveryControl";

const EMPTY_FEED: UpdateFeedSnapshot = {
  entries: []
};

export function UpdateFeedPanel() {
  const [expanded, setExpanded] = useState(false);
  const [feed, setFeed] = useState(EMPTY_FEED);

  useEffect(() => {
    let active = true;
    let unlisten: () => void = () => undefined;
    getUpdateFeed()
      .then((snapshot) => active && setFeed(snapshot))
      .catch(() => undefined);
    void listenToUpdateFeed((entry) => {
      if (active) setFeed((current) => mergeUpdateEntry(current, entry));
    }).then((nextUnlisten) => {
      if (active) unlisten = nextUnlisten;
      else nextUnlisten();
    });
    return () => {
      active = false;
      unlisten();
    };
  }, []);

  return (
    <section className="update-feed" aria-label="UPDATES">
      <div className="update-feed__header">
        <span>UPDATES {feed.entries.length}</span>
        <button
          type="button"
          className="update-feed__toggle"
          aria-label={expanded ? "COLLAPSE UPDATES" : "EXPAND UPDATES"}
          aria-expanded={expanded}
          title={expanded ? "COLLAPSE UPDATES" : "EXPAND UPDATES"}
          onClick={() => setExpanded((current) => !current)}
        >
          <img
            src={downIcon}
            alt=""
            aria-hidden="true"
            className={expanded ? "is-expanded" : ""}
          />
        </button>
      </div>
      {expanded ? (
        <div className="update-feed__body">
          <span className="update-feed__path">
            DOWNLOADS/PEBBLE/PEBBLE-UPDATES.MD
          </span>
          {isTauriRuntime() ? <DiscoveryControl /> : null}
          {isTauriRuntime() ? <PublicSourceControl /> : null}
          {feed.entries.length === 0 ? (
            <p className="update-feed__empty">NO SAVED UPDATES YET</p>
          ) : (
            <ol>
              {feed.entries.map((entry) => (
                <li key={entry.id}>
                  <p>{entry.summary}</p>
                  <span>
                    {entry.kind} · {formatUpdateTime(entry.occurredAt)}
                    {entry.saved ? "" : " · NOT SAVED"}
                  </span>
                </li>
              ))}
            </ol>
          )}
        </div>
      ) : null}
    </section>
  );
}
