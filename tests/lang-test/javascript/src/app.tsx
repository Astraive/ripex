// ripex-lang-test: TSX fixtures — JSX, components, fragments, generics.
import { Status } from './types.js';

export interface Props {
  title: string;
  count?: number;
  status?: Status;
}

export function Header({ title, count = 0 }: Props) {
  return (
    <header>
      <h1>{title}</h1>
      <span>{count}</span>
    </header>
  );
}

export const App = (props: Props) => (
  <>
    <Header title={props.title} />
    <article data-email="a@b.com">Alice</article>
  </>
);
