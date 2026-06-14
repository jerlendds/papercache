import { For, createEffect, createSignal, onCleanup } from "solid-js";

import { Icon } from "./Icon";
import "./ChatPaperIntro.css";

type ChatHistoryBook = {
  id: string;
  title: string;
  subtitle: string;
  detail: string;
  theme: "red" | "blue" | "mint" | "marker";
};

type ChatHistoryCard = {
  role: "Question" | "Answer" | "Citation" | "System" | "Note";
  title: string;
  body: string;
  height?: number;
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
          "System",
          "Note",
        ];
        const role = roles[index % roles.length];
        const baseHeight = {
          Question: 270,
          Answer: 420,
          Citation: 330,
          System: 250,
          Note: 230,
        }[role];
        return {
          role,
          title: `${book.title} ${index + 1}`,
          height: baseHeight + (index % 3) * 34,
          body:
            role === "Citation"
              ? "Placeholder citation card with paper title, page number, and the retrieved passage that grounded the response."
              : role === "Answer"
                ? `${book.detail} This answer card will hold the generated response and links back to source papers.`
                : role === "System"
                  ? "Placeholder system event for retrieval settings, model context, indexing status, or tool activity during this chat."
                : role === "Note"
                  ? "Placeholder session note for a follow-up thread, unresolved question, or saved insight from the conversation."
                  : "Placeholder user question from this saved research conversation.",
        };
      }),
    ]),
  );

export function ChatPaperIntro() {
  const [open, setOpen] = createSignal(false);
  const [stackScroll, setStackScroll] = createSignal(0);
  const [selectedBook, setSelectedBook] = createSignal<ChatHistoryBook>(
    CHAT_HISTORY[0],
  );
  let stackScroller: HTMLDivElement | undefined;
  let stackSection: HTMLElement | undefined;

  const stackProgress = () => {
    const count = selectedCards().length;
    if (count === 0) return 0;
    return Math.min(1, Math.max(0, stackScroll() / count));
  };

  const closePaper = () => setOpen(false);

  const openBook = (book: ChatHistoryBook) => {
    setSelectedBook(book);
    setStackScroll(0);
    setOpen(true);
    requestAnimationFrame(() => {
      if (stackScroller) stackScroller.scrollTop = 0;
    });
  };

  const selectedCards = () => CHAT_HISTORY_CARDS[selectedBook().id] ?? [];

  const updateStackScroll = (element: HTMLDivElement) => {
    const stackTop = stackSection?.offsetTop ?? 0;
    const stackScrollable = Math.max(
      1,
      (stackSection?.offsetHeight ?? element.clientHeight) -
        element.clientHeight,
    );
    const progress = Math.min(
      1,
      Math.max(0, (element.scrollTop - stackTop) / stackScrollable),
    );
    setStackScroll(progress * selectedCards().length);
  };

  const scrollToStackProgress = (progress: number) => {
    if (!stackScroller || !stackSection) return;

    const stackTop = stackSection.offsetTop;
    const stackScrollable = Math.max(
      1,
      stackSection.offsetHeight - stackScroller.clientHeight,
    );
    const nextProgress = Math.min(1, Math.max(0, progress));
    stackScroller.scrollTop = stackTop + nextProgress * stackScrollable;
    setStackScroll(nextProgress * selectedCards().length);
  };

  const updateVirtualScrollbar = (
    event: PointerEvent & { currentTarget: HTMLDivElement },
  ) => {
    const rect = event.currentTarget.getBoundingClientRect();
    scrollToStackProgress((event.clientY - rect.top) / rect.height);
  };

  const onVirtualScrollbarPointerDown = (
    event: PointerEvent & { currentTarget: HTMLDivElement },
  ) => {
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    updateVirtualScrollbar(event);
  };

  const onVirtualScrollbarPointerMove = (
    event: PointerEvent & { currentTarget: HTMLDivElement },
  ) => {
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      updateVirtualScrollbar(event);
    }
  };

  const onVirtualScrollbarPointerUp = (
    event: PointerEvent & { currentTarget: HTMLDivElement },
  ) => {
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") closePaper();
  };

  createEffect(() => {
    if (!open()) return;
    window.addEventListener("keydown", onKeyDown);
    onCleanup(() => window.removeEventListener("keydown", onKeyDown));
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
        ref={stackScroller}
        onScroll={(event) => updateStackScroll(event.currentTarget)}
      >
        <button
          type="button"
          class="chat-paper-close"
          aria-label="Close papercache introduction"
          onClick={closePaper}
        >
          ×
        </button>

        <div class="chat-history-intro" aria-hidden="true">
          <p>{selectedBook().subtitle}</p>
          <h2>{selectedBook().title}</h2>
          <span>{selectedBook().detail}</span>
        </div>

        <main
          id="chat-stack-scroll"
          class="chat-stack-scroll"
          ref={stackSection}
          style={`--count: ${selectedCards().length}; --scroll: ${stackScroll().toFixed(4)}`}
        >
          <div class="chat-stack-stage">
            <div class="chat-stack-origin" aria-label="Saved chat history paper stack">
              <div class="chat-stack-pile" aria-hidden="true" />

              <For each={selectedCards()}>
                {(card, index) => (
                  <article
                    classList={{
                      "chat-history-sheet": true,
                      [`kind-${card.role.toLowerCase()}`]: true,
                      "is-gone": stackScroll() - index() >= 1,
                    }}
                    style={`--i: ${index()}; --sheet-height: ${
                      card.height ? `${card.height}px` : "var(--chat-sheet-h)"
                    }`}
                  >
                    <header class="chat-history-sheet-header">
                      <span>{card.role}</span>
                      <strong>
                        {String(index() + 1).padStart(2, "0")} /{" "}
                        {String(selectedCards().length).padStart(2, "0")}
                      </strong>
                    </header>
                    <div class="chat-history-sheet-body">
                      <h3>{card.title}</h3>
                      <p>{card.body}</p>
                    </div>
                  </article>
                )}
              </For>
            </div>
          </div>
        </main>

        <div class="chat-stack-status" aria-live="polite">
          {stackScroll() >= selectedCards().length - 0.02
            ? "Stack complete"
            : `Sheet ${Math.min(
                selectedCards().length,
                Math.floor(stackScroll()) + 1,
              )} / ${selectedCards().length}`}
        </div>

        <form class="chat-drawer-input" onSubmit={(event) => event.preventDefault()}>
          <Icon name="search" />
          <input
            aria-label="Ask about this chat history"
            placeholder="Ask a follow-up about this chat..."
          />
          <button type="submit">Ask</button>
        </form>
      </div>

      <div
        classList={{ "chat-virtual-scrollbar": true, open: open() }}
        role="scrollbar"
        aria-controls="chat-stack-scroll"
        aria-orientation="vertical"
        aria-valuemin="0"
        aria-valuemax={selectedCards().length}
        aria-valuenow={Math.min(
          selectedCards().length,
          Math.floor(stackScroll()) + 1,
        )}
        style={`--scroll-progress: ${stackProgress().toFixed(4)}`}
        onPointerDown={onVirtualScrollbarPointerDown}
        onPointerMove={onVirtualScrollbarPointerMove}
        onPointerUp={onVirtualScrollbarPointerUp}
        onPointerCancel={onVirtualScrollbarPointerUp}
      >
        <div class="chat-virtual-scrollbar-thumb" />
      </div>
    </section>
  );
}
