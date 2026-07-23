import { useEffect, useState } from "react";
import downIcon from "../assets/icons/down.svg";
import {
  buildChangeStoryItems,
  changeStoryLabel,
  formatUpdateTime,
  isAttentionEntry,
  mergeUpdateEntry,
  updateSignalLabel,
  type ChangeStory,
  type UpdateEntry,
  type UpdateFeedSnapshot
} from "../features/updates/updateFeed";
import { listenToUpdateFeed } from "../lib/events";
import { getUpdateFeed } from "../lib/invoke";

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
      .then((snapshot) => {
        if (!active) return;
        setFeed(snapshot);
        if (snapshot.entries.some(isAttentionEntry)) setExpanded(true);
      })
      .catch(() => undefined);
    void listenToUpdateFeed((entry) => {
      if (!active) return;
      setFeed((current) => mergeUpdateEntry(current, entry));
      if (isAttentionEntry(entry)) setExpanded(true);
    }).then((nextUnlisten) => {
      if (active) unlisten = nextUnlisten;
      else nextUnlisten();
    });
    return () => {
      active = false;
      unlisten();
    };
  }, []);

  if (feed.entries.length === 0) {
    return <div className="update-feed update-feed--empty" aria-hidden="true" />;
  }

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
          <UpdateFeedList entries={feed.entries} />
        </div>
      ) : null}
    </section>
  );
}

export function UpdateFeedList({ entries }: { entries: UpdateEntry[] }) {
  return (
    <ol>
      {buildChangeStoryItems(entries).map((item) =>
        item.type === "story" ? (
          <ChangeStoryItem key={item.story.id} story={item.story} />
        ) : (
          <UpdateEntryItem key={item.entry.id} entry={item.entry} />
        )
      )}
    </ol>
  );
}

function ChangeStoryItem({ story }: { story: ChangeStory }) {
  const saved = story.entries.every((entry) => entry.saved);
  return (
    <li className="update-feed__story">
      <span className="update-feed__story-label">
        {changeStoryLabel(story)}
      </span>
      <div className="update-feed__timeline" role="list">
        {story.entries.map((entry) => (
          <div className="update-feed__event" role="listitem" key={entry.id}>
            <span className="update-feed__event-time">
              {formatUpdateTime(entry.occurredAt)}
            </span>
            <div className="update-feed__event-content">
              {entry.signal ? (
                <span className="update-feed__signal">
                  {updateSignalLabel(entry.signal)}
                </span>
              ) : null}
              <p>{entry.summary}</p>
            </div>
          </div>
        ))}
      </div>
      <span className="update-feed__time">
        {formatUpdateTime(story.startedAt)} → {formatUpdateTime(story.endedAt)}
        {saved ? "" : " · NOT SAVED"}
      </span>
    </li>
  );
}

function UpdateEntryItem({ entry }: { entry: UpdateEntry }) {
  return (
    <li>
      {entry.signal ? (
        <span className="update-feed__signal">
          {updateSignalLabel(entry.signal)}
        </span>
      ) : null}
      <p>{entry.summary}</p>
      <span className="update-feed__time">
        {entry.kind} · {formatUpdateTime(entry.occurredAt)}
        {entry.saved ? "" : " · NOT SAVED"}
      </span>
    </li>
  );
}
