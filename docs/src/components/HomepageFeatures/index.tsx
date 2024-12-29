import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Freedom of Form',
    Svg: require('@site/static/img/undraw_docusaurus_mountain.svg').default,
    description: (
      <>
        Clover helps you extend your body in any way you can imagine,
        from animal ears to cybernetic legs. The only limits are your imagination,
        and time. Clover was started to help solve what seemed like unsolvable dysphoria,
        and evolved into something much more helpful than anyone could think of.
      </>
    ),
  },
  {
    title: 'FOSS, Secure, and Private',
    Svg: require('@site/static/img/undraw_docusaurus_tree.svg').default,
    description: (
      <>
        Technology you rely on shouldn&apos;t be a black box, and certainly not a cardboard black box.
        Clover is 100% open source and is continously built from the ground up to be secure from the internal logic,
        to the detailed and fully simulatable permission system.
      </>
    ),
  },
  {
    title: 'Tinkering is Encouraged',
    Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
    description: (
      <>
        Clover is designed to be extended, modified, and remixed. All internal APIs are documented as they&apos;re
        written (... or... try to be), and libraries are exposed with multi-language bindings to let you write in whatever
        you&apos;re most comfortable in. All 3D models are published in open formats with assembly instructions alongside.
      </>
    ),
  },
];

function Feature({title, Svg, description}: FeatureItem) {
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

export default function HomepageFeatures(): JSX.Element {
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
