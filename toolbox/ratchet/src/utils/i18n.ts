import i18next from "i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import resourcesToBackend from "i18next-resources-to-backend";
// import LocizeBackend from 'i18next-locize-backend'
import { initReactI18next } from "react-i18next/initReactI18next";

export const fallbackLng = "en";
export const languages = [fallbackLng, "es", "de"];
export const defaultNS = "translation";
export const cookieName = "i18next";
export const headerName = "x-i18next-current-language";

const runsOnServerSide = typeof window === "undefined";

i18next
  .use(initReactI18next)
  .use(LanguageDetector)
  .use(
    resourcesToBackend(
      (language: string, namespace: string) =>
        import(`./locales/${language}/${namespace}.json`)
    )
  )
  // .use(runsOnServerSide ? LocizeBackend : resourcesToBackend((language: string, namespace: string) => import(`./locales/${language}/${namespace}.json`))) // locize backend could be used, but prefer to keep it in sync with server side
  .init({
    // debug: true,
    supportedLngs: languages,
    fallbackLng,
    lng: undefined, // let detect the language on client side
    fallbackNS: defaultNS,
    defaultNS,
    detection: {
      order: ["path", "htmlTag", "cookie", "navigator"],
    },
    preload: runsOnServerSide ? languages : [],
    // backend: {
    //   projectId: '01b2e5e8-6243-47d1-b36f-963dbb8bcae3'
    // }
  });

export default i18next;
