export const dashify = (string: string) =>
  string.replace(/[A-Z]/g, (m) => "-" + m.toLowerCase());

export const camelToKebab = (str: string): string => {
  return str.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase();
};

export const addEllipsis = (text: string, size: number = 20) => {
  return text.length > size ? `${text.slice(0, size)}...` : text;
};
