import { dashify } from "./strings";

export const getCss = (
  object: Partial<CSSStyleDeclaration> & Record<`--${string}`, string>
) =>
  Object.entries(object)
    .filter(([, value]) => value !== undefined)
    .map(([key, value]) => `${dashify(key)}:${value}`)
    .join(";");
