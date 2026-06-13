export function PlaceholderPage(props: { title: string; body: string }) {
  return (
    <section class="page placeholder-page">
      <h1>{props.title}</h1>
      <p>{props.body}</p>
    </section>
  );
}
