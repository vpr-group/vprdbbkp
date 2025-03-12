import { dashify } from "./strings";

export type CSSProperties = Partial<
  Omit<
    CSSStyleDeclaration,
    | "item"
    | "getPropertyPriority"
    | "getPropertyValue"
    | "removeProperty"
    | "setProperty"
    | "length"
    | "parentRule"
  >
>;

export const getCss = (
  object: CSSProperties & { [key: string]: string | number | undefined }
) =>
  Object.entries(object)
    .filter(([, value]) => value !== undefined)
    .map(([key, value]) => `${dashify(key)}:${value}`)
    .join(";");
