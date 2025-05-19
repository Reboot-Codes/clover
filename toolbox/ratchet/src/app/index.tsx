import { Namespace, FlatNamespace, KeyPrefix } from "i18next";
import { FallbackNs } from "react-i18next";
import i18next, { headerName } from "../utils/i18n";

type $Tuple<T> = readonly [T?, ...T[]];
type $FirstNamespace<Ns extends Namespace> = Ns extends readonly any[]
  ? Ns[0]
  : Ns;

export async function getT<
  Ns extends FlatNamespace | $Tuple<FlatNamespace>,
  KPrefix extends KeyPrefix<
    FallbackNs<
      Ns extends FlatNamespace ? FlatNamespace : $FirstNamespace<FlatNamespace>
    >
  > = undefined
>(ns?: Ns, options: { keyPrefix?: KPrefix } = {}) {
  await i18next.changeLanguage();
  if (ns && !i18next.hasLoadedNamespace(ns as string | string[])) {
    await i18next.loadNamespaces(ns as string | string[]);
  }
  return {
    t: i18next.t,
    i18n: i18next,
  };
}
