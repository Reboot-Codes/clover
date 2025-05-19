import { getT } from "../index";

export default async function Page() {
  const { t } = await getT();
  const version = require("../../../package.json").version;

  return (
    <>
      <h1>{t("welcome.hello")}</h1>
      <p>{t("welcome.version", { version })}</p>
    </>
  );
}
