export const extractDateTimeFromEntryName = (str: string): Date | null => {
  const dateTimeRegex = /(\d{4}-\d{2}-\d{2})-(\d{2})(\d{2})(\d{2})/;
  const match = str.match(dateTimeRegex);

  if (match) {
    const [_, dateStr, hours, minutes, seconds] = match;
    const dateTimeStr = `${dateStr}T${hours}:${minutes}:${seconds}`;
    return new Date(dateTimeStr);
  }

  return null;
};

export const formatDate = (date: Date): string => {
  const day = date.getDate().toString().padStart(2, "0");
  const month = (date.getMonth() + 1).toString().padStart(2, "0"); // getMonth() is 0-indexed
  const year = date.getFullYear();
  const hours = date.getHours().toString().padStart(2, "0");
  const minutes = date.getMinutes().toString().padStart(2, "0");

  return `${day}-${month}-${year} ${hours}:${minutes}`;
};
