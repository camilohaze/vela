import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

const FeatureList = [
  {
    title: 'Functional First',
    Svg: require('@site/static/img/undraw_docusaurus_mountain.svg').default,
    description: (
      <>
        Vela embraces functional programming with immutable data structures,
        pure functions, and powerful pattern matching. Write safer, more
        predictable code.
      </>
    ),
  },
  {
    title: 'Reactive by Design',
    Svg: require('@site/static/img/undraw_docusaurus_tree.svg').default,
    description: (
      <>
        Built-in reactive signals make state management effortless. Automatic
        dependency tracking and efficient updates keep your UI in sync.
      </>
    ),
  },
  {
    title: 'Actor Concurrency',
    Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
    description: (
      <>
        Message-passing concurrency with the actor model. Build scalable,
        fault-tolerant systems without shared mutable state.
      </>
    ),
  },
  {
    title: 'Multi-Platform',
    Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
    description: (
      <>
        Compile to WebAssembly, native binaries, or run on the Vela VM.
        Write once, deploy anywhere.
      </>
    ),
  },
  {
    title: 'Declarative UI',
    Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
    description: (
      <>
        Build beautiful, reactive user interfaces with composable widgets
        and declarative syntax. Inspired by Flutter and React.
      </>
    ),
  },
  {
    title: 'Type Safe',
    Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
    description: (
      <>
        Strong static typing with type inference. Catch errors at compile time,
        not runtime. Advanced generics and algebraic data types.
      </>
    ),
  },
];

function Feature({Svg, title, description}) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures() {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}