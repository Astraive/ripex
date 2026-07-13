// ripex-lang-test: TSX fixtures — JSX, components, fragments, generics.
import { User } from './models/user.js';

export interface Props {
  title: string;
  count?: number;
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
    <User name="Alice" email="a@b.com" />
  </>
);
