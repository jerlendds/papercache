import { For, createEffect, createSignal, onCleanup } from "solid-js";

import "./ChatPaperIntro.css";

type ChatHistoryBook = {
  id: string;
  title: string;
  subtitle: string;
  detail: string;
  theme: "red" | "blue" | "mint" | "marker";
};

type ChatHistoryCard = {
  role: "Question" | "Answer" | "Citation" | "Note";
  title: string;
  body: string;
};

const CHAT_HISTORY: ChatHistoryBook[] = [
  {
    id: "retrieval",
    title: "Retrieval Notes",
    subtitle: "local chunks",
    detail: "Grounded answers from indexed PDF passages.",
    theme: "red",
  },
  {
    id: "citations",
    title: "Citation Trail",
    subtitle: "paper graph",
    detail: "Questions about references, authors, and related work.",
    theme: "blue",
  },
  {
    id: "methods",
    title: "Methods Review",
    subtitle: "synthesis",
    detail: "Compare approaches across papers in the library.",
    theme: "mint",
  },
  {
    id: "reading",
    title: "Reading Queue",
    subtitle: "next papers",
    detail: "Summaries, open questions, and follow-up reading.",
    theme: "marker",
  },
];

const CHAT_HISTORY_CARDS: Record<string, ChatHistoryCard[]> =
  Object.fromEntries(
    CHAT_HISTORY.map((book) => [
      book.id,
      Array.from({ length: 16 }, (_, index) => {
        const roles: ChatHistoryCard["role"][] = [
          "Question",
          "Answer",
          "Citation",
          "Note",
        ];
        const role = roles[index % roles.length];
        return {
          role,
          title: `${book.title} ${index + 1}`,
          body:
            role === "Citation"
              ? "Placeholder citation card with paper title, page number, and the retrieved passage that grounded the response."
              : role === "Answer"
                ? `${book.detail} This answer card will hold the generated response and links back to source papers.`
                : role === "Note"
                  ? "Placeholder session note for a follow-up thread, unresolved question, or saved insight from the conversation."
                  : "Placeholder user question from this saved research conversation.",
        };
      }),
    ]),
  );

const HISTORY_CARD_HEIGHT = 212;
const HISTORY_CARD_GAP = 18;
const HISTORY_ROW_HEIGHT = HISTORY_CARD_HEIGHT + HISTORY_CARD_GAP;

export function ChatPaperIntro() {
  const [open, setOpen] = createSignal(false);
  const [scrollTop, setScrollTop] = createSignal(0);
  const [selectedBook, setSelectedBook] = createSignal<ChatHistoryBook>(
    CHAT_HISTORY[0],
  );

  createEffect(() => {
    if (!open()) {
      document.body.style.overflow = "";
      return;
    }

    document.body.style.overflow = "hidden";
  });

  const closePaper = () => setOpen(false);

  const openBook = (book: ChatHistoryBook) => {
    setSelectedBook(book);
    setScrollTop(0);
    setOpen(true);
  };

  const selectedCards = () => CHAT_HISTORY_CARDS[selectedBook().id] ?? [];

  const visibleCards = () => {
    const cards = selectedCards();
    const start = Math.max(0, Math.floor(scrollTop() / HISTORY_ROW_HEIGHT) - 2);
    const end = Math.min(cards.length, start + 8);
    return cards.slice(start, end).map((card, offset) => ({
      card,
      index: start + offset,
    }));
  };

  const onStageClick = (event: MouseEvent) => {
    if (event.target === event.currentTarget) closePaper();
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") closePaper();
  };

  createEffect(() => {
    if (!open()) return;
    window.addEventListener("keydown", onKeyDown);
    onCleanup(() => window.removeEventListener("keydown", onKeyDown));
  });

  onCleanup(() => {
    document.body.style.overflow = "";
  });

  return (
    <section class="chat-paper-intro" aria-label="Chat with your library">
      <section class="chat-demo-copy" aria-label="papercache intro">
        <p class="chat-eyebrow">local-first research library</p>
        <h1>Chat with papers where they live.</h1>
        <p class="chat-lead">
          papercache watches folders, ingests PDFs in the background, and builds
          a local searchable library without moving or reorganizing your files.
        </p>
        <p class="chat-hint">Click the book to open that chats history.</p>
      </section>

      <section class="chat-book-zone" aria-label="chat history shelf">
        <div class="chat-shelf-title">
          <span>Conversation history</span>
          <strong>Chat with your research</strong>
        </div>
        <div class="chat-history-shelf">
          <For each={CHAT_HISTORY}>
            {(book) => (
              <button
                type="button"
                class={`chat-book-button theme-${book.theme}`}
                aria-label={`Open ${book.title}`}
                onClick={() => openBook(book)}
              >
                <div class="bk-book">
                  <div class="bk-front">
                    <div class="bk-cover-back" />
                    <div class="bk-cover">
                      <h2>
                        <span>{book.subtitle}</span>
                        <span>{book.title}</span>
                      </h2>
                    </div>
                  </div>
                  <div class="bk-page">
                    <div class="bk-content">
                      <p>{book.detail}</p>
                    </div>
                  </div>
                  <div class="bk-back">
                    <p>{book.detail}</p>
                  </div>
                  <div class="bk-right" />
                  <div class="bk-left">
                    <h2>
                      <span>{book.subtitle}</span>
                      <span>{book.title}</span>
                    </h2>
                  </div>
                  <div class="bk-top" />
                  <div class="bk-bottom" />
                </div>
              </button>
            )}
          </For>
        </div>
      </section>

      <div
        classList={{ "chat-paper-overlay": true, open: open() }}
        aria-hidden={!open()}
        onClick={onStageClick}
      >
        <div class="chat-stage-label">saved chat · stacked sheets</div>
        <button
          type="button"
          class="chat-paper-close"
          aria-label="Close papercache introduction"
          onClick={closePaper}
        >
          ×
        </button>

        <div class="chat-history-modal">
          <header class="chat-history-modal-header">
            <span>{selectedBook().subtitle}</span>
            <h2>{selectedBook().title}</h2>
            <p>{selectedBook().detail}</p>
          </header>
          <div
            class="chat-history-scroll"
            onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
          >
            <div
              class="chat-history-spacer"
              style={{
                height: `${selectedCards().length * HISTORY_ROW_HEIGHT}px`,
              }}
            >
              <For each={visibleCards()}>
                {(item) => (
                  <article
                    class="chat-history-sheet"
                    style={{
                      transform: `translateY(${item.index * HISTORY_ROW_HEIGHT}px)`,
                      "z-index": `${selectedCards().length - item.index}`,
                    }}
                  >
                    <span>{item.card.role}</span>
                    <h3>{item.card.title}</h3>
                    <p>{item.card.body}</p>
                  </article>
                )}
              </For>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
